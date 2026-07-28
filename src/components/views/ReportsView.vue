<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { exportCsv, exportPdf, ledgerSummary, projectBalances, projectsList } from "../../lib/api";
import { notify } from "../../lib/feedback";
import { formatPHPFromCents } from "../../lib/money";
import type { LedgerSummary, Project, ProjectBalanceRow } from "../../lib/types";

const props = defineProps<{ sessionToken: string }>();

const projects = ref<Project[]>([]);
const filterFrom = ref(getYearStartString());
const filterTo = ref(getLocalDateString());
const filterProjectId = ref<string>("");
const reportType = ref("statement");

const summary = ref<LedgerSummary | null>(null);
const balances = ref<ProjectBalanceRow[]>([]);
const loading = ref(false);
const errorMessage = ref<string | null>(null);

const activeFilter = computed(() => ({
  from: filterFrom.value || null,
  to: filterTo.value || null,
  project_id: filterProjectId.value ? Number(filterProjectId.value) : null,
}));

const projectMap = computed(() => new Map(projects.value.map((project) => [project.id, project])));

const reportRows = computed(() =>
  balances.value
    .slice()
    .sort((a, b) => a.project_name.localeCompare(b.project_name))
    .map((row) => {
      const project = projectMap.value.get(row.project_id);
      const targetCents = project?.target_amount_cents ?? 0;
      const fundedPct = targetCents > 0 ? Math.min(100, Math.round((row.donations_cents / targetCents) * 100)) : 0;

      return {
        ...row,
        project_code: `PRJ-${String(row.project_id).padStart(3, "0")}`,
        target_cents: targetCents,
        funded_pct: fundedPct,
      };
    }),
);

const totalTargetCents = computed(() =>
  reportRows.value.reduce((sum, row) => sum + row.target_cents, 0),
);

const totalReceiptsCents = computed(() =>
  reportRows.value.reduce((sum, row) => sum + row.donations_cents, 0),
);

const totalDisbursementsCents = computed(() =>
  reportRows.value.reduce((sum, row) => sum + row.expenses_cents, 0),
);

const endingBalanceCents = computed(() =>
  reportRows.value.reduce((sum, row) => sum + row.balance_cents, 0),
);

const reportRangeLabel = computed(() => {
  const from = formatDateLong(filterFrom.value);
  const to = formatDateLong(filterTo.value);
  return `For the period ${from} to ${to} · in Philippine Peso`;
});

function getLocalDateString(date = new Date()) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function getYearStartString() {
  const now = new Date();
  return `${now.getFullYear()}-01-01`;
}

function formatDateLong(value: string) {
  const parsed = new Date(`${value}T00:00:00`);
  if (Number.isNaN(parsed.getTime())) return value;
  return new Intl.DateTimeFormat("en-GB", {
    day: "2-digit",
    month: "long",
    year: "numeric",
  }).format(parsed);
}

function formatPercent(numerator: number, denominator: number) {
  if (denominator <= 0) return "0%";
  return `${Math.min(100, Math.round((numerator / denominator) * 100))}%`;
}

function formatSignedMoney(cents: number) {
  const formatted = formatPHPFromCents(Math.abs(cents));
  return cents < 0 ? `(${formatted})` : formatted;
}

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    const [projectRows, summaryValue, balancesValue] = await Promise.all([
      projectsList(props.sessionToken),
      ledgerSummary(props.sessionToken, activeFilter.value),
      projectBalances(props.sessionToken, activeFilter.value),
    ]);
    projects.value = projectRows;
    summary.value = summaryValue;
    balances.value = balancesValue;
  } catch (error: any) {
    errorMessage.value = String(error);
  } finally {
    loading.value = false;
  }
}

async function exportSummaryPdf() {
  const dest = await save({
    defaultPath: `reports-${filterFrom.value || "all"}-${filterTo.value || "all"}.pdf`,
    filters: [{ name: "PDF", extensions: ["pdf"] }],
  });
  if (!dest) return;
  await exportPdf(props.sessionToken, {
    title: "Church Ledger - Reports & Export",
    filter: activeFilter.value,
    dest_path: dest,
  });
  notify("PDF summary exported.");
}

async function exportSummaryCsv() {
  const dest = await save({
    defaultPath: `reports-${filterFrom.value || "all"}-${filterTo.value || "all"}.csv`,
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!dest) return;
  await exportCsv(props.sessionToken, {
    kind: "projects",
    filter: activeFilter.value,
    dest_path: dest,
  });
  notify("CSV exported.");
}

function printReport() {
  window.print();
}

function generateReport() {
  load();
}

onMounted(load);
</script>

<template>
  <div class="space-y-5 text-[var(--ledger-text)]">
    <section class="ledger-panel overflow-hidden rounded-[26px]">
      <div class="flex flex-col gap-5 border-b border-[color:var(--ledger-line)] px-6 py-5 lg:flex-row lg:items-start lg:justify-between print:border-b-0">
        <div>
          <p class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">
            BOOK OF ACCOUNTS · FY 2026
          </p>
          <h2 class="ledger-heading mt-2 text-4xl text-[var(--ledger-text)]">
            Reports &amp; Export
          </h2>
          <p class="mt-3 max-w-2xl text-sm text-[var(--ledger-muted)]">
            Statement of receipts and disbursements for the selected period.
          </p>
        </div>

        <div class="flex flex-wrap gap-2 print:hidden">
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-[12px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-4 py-3 text-sm font-semibold text-[var(--ledger-text)] transition hover:bg-[rgba(255,255,255,0.95)]"
            @click="printReport"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <path d="M6 9V2h12v7" />
              <path d="M6 18H5a3 3 0 0 1-3-3v-3a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v3a3 3 0 0 1-3 3h-1" />
              <path d="M6 14h12v8H6z" />
            </svg>
            <span>Print</span>
          </button>
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-[12px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-4 py-3 text-sm font-semibold text-[var(--ledger-text)] transition hover:bg-[rgba(255,255,255,0.95)]"
            @click="exportSummaryCsv"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <path d="M14 3v4a1 1 0 0 0 1 1h4" />
              <path d="M17 21H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h7l5 5v11a2 2 0 0 1-2 2Z" />
              <path d="M9 13h6" />
              <path d="M9 17h6" />
            </svg>
            <span>CSV</span>
          </button>
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-[12px] border border-[var(--ledger-red)] bg-[var(--ledger-red)] px-4 py-3 text-sm font-semibold text-white transition hover:bg-[#a73d24]"
            @click="exportSummaryPdf"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <path d="M14 3v4a1 1 0 0 0 1 1h4" />
              <path d="M6 2h8l6 6v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2Z" />
              <path d="M9 13h6" />
              <path d="M9 17h6" />
            </svg>
            <span>PDF</span>
          </button>
        </div>
      </div>

      <div class="px-4 py-4 print:px-0 print:pt-0">
        <section class="ledger-card rounded-[4px] p-4 print:hidden">
          <div class="grid gap-3 xl:grid-cols-[1.1fr_1fr_1fr_1fr_auto]">
            <label class="grid gap-2">
              <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                Report Type
              </span>
              <select
                v-model="reportType"
                class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition focus:border-[color:var(--ledger-gold)]"
              >
                <option value="statement">Statement of Receipts &amp; Disbursements</option>
                <option value="summary">Summary by Project</option>
                <option value="balance">Balance Sheet View</option>
              </select>
            </label>

            <label class="grid gap-2">
              <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                From
              </span>
              <input
                v-model="filterFrom"
                type="date"
                class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition focus:border-[color:var(--ledger-gold)]"
              />
            </label>

            <label class="grid gap-2">
              <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                To
              </span>
              <input
                v-model="filterTo"
                type="date"
                class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition focus:border-[color:var(--ledger-gold)]"
              />
            </label>

            <label class="grid gap-2">
              <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                Project
              </span>
              <select
                v-model="filterProjectId"
                class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition focus:border-[color:var(--ledger-gold)]"
              >
                <option value="">All projects</option>
                <option v-for="project in projects" :key="project.id" :value="String(project.id)">
                  {{ project.name }}
                </option>
              </select>
            </label>

            <button
              type="button"
              class="mt-auto inline-flex h-12 items-center justify-center rounded-[4px] bg-[var(--ledger-navy)] px-5 text-sm font-semibold text-white transition hover:bg-[var(--ledger-navy-2)]"
              @click="generateReport"
            >
              {{ loading ? "Generating..." : "Generate" }}
            </button>
          </div>
        </section>

        <div v-if="errorMessage" class="mt-4 rounded-[4px] border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
          {{ errorMessage }}
        </div>

        <template v-if="!loading">
          <section class="mt-4 grid gap-4 md:grid-cols-3 print:mt-0">
            <article class="ledger-card rounded-[4px] p-5">
              <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">Total Receipts</div>
              <div class="mt-4 text-2xl font-semibold tracking-tight text-[var(--ledger-green)]">
                {{ formatPHPFromCents(summary?.total_donations_cents || 0) }}
              </div>
            </article>

            <article class="ledger-card rounded-[4px] p-5">
              <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">Total Disbursements</div>
              <div class="mt-4 text-2xl font-semibold tracking-tight text-[var(--ledger-red)]">
                {{ formatPHPFromCents(summary?.total_expenses_cents || 0) }}
              </div>
            </article>

            <article class="ledger-card rounded-[4px] p-5">
              <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">Ending Balance</div>
              <div class="mt-4 text-2xl font-semibold tracking-tight text-[var(--ledger-text)]">
                {{ formatSignedMoney(summary?.balance_cents || 0) }}
              </div>
              <div class="mt-2 text-sm text-[var(--ledger-muted)]">
                {{ reportRows.length }} entries
              </div>
            </article>
          </section>

          <section class="ledger-panel mt-4 overflow-hidden rounded-[4px]">
            <div class="border-b border-[color:var(--ledger-line)] px-4 py-6 text-center">
              <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">
                GRACE COMMUNITY CHURCH · FINANCE OFFICE
              </div>
              <h3 class="ledger-heading mt-2 text-2xl text-[var(--ledger-text)]">
                Statement of Receipts and Disbursements
              </h3>
              <p class="mt-1 text-xs text-[var(--ledger-muted)]">
                {{ reportRangeLabel }}
              </p>
            </div>

            <div class="overflow-x-auto">
              <table class="w-full min-w-[900px] border-separate border-spacing-0">
                <thead>
                  <tr class="bg-[rgba(240,229,203,0.85)] text-[11px] uppercase tracking-[0.28em] text-[var(--ledger-text)]">
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Project</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-right font-semibold">Target</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-right font-semibold">Receipts</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-right font-semibold">Disbursements</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-right font-semibold">Balance</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-right font-semibold">% Funded</th>
                  </tr>
                </thead>

                <tbody>
                  <tr
                    v-for="row in reportRows"
                    :key="row.project_id"
                    class="border-b border-[color:rgba(215,196,154,0.7)] bg-[rgba(251,247,235,0.92)] transition hover:bg-[rgba(247,241,224,0.95)]"
                  >
                    <td class="px-4 py-4">
                      <div class="text-sm text-[var(--ledger-text)]">{{ row.project_name }}</div>
                      <div class="mt-1 text-[11px] uppercase tracking-[0.22em] text-[var(--ledger-muted)]">
                        {{ row.project_code }}
                      </div>
                    </td>
                    <td class="px-4 py-4 text-right text-sm tabular-nums text-[var(--ledger-text)]">
                      {{ formatPHPFromCents(row.target_cents) }}
                    </td>
                    <td class="px-4 py-4 text-right text-sm tabular-nums text-[var(--ledger-green)]">
                      {{ formatPHPFromCents(row.donations_cents) }}
                    </td>
                    <td class="px-4 py-4 text-right text-sm tabular-nums text-[var(--ledger-red)]">
                      {{ formatPHPFromCents(row.expenses_cents) }}
                    </td>
                    <td class="px-4 py-4 text-right text-sm tabular-nums text-[var(--ledger-text)]">
                      {{ formatSignedMoney(row.balance_cents) }}
                    </td>
                    <td class="px-4 py-4 text-right text-sm tabular-nums text-[var(--ledger-text)]">
                      {{ formatPercent(row.donations_cents, row.target_cents) }}
                    </td>
                  </tr>

                  <tr v-if="reportRows.length === 0">
                    <td colspan="6" class="px-4 py-10 text-center text-sm text-[var(--ledger-muted)]">
                      No projects found for the selected period.
                    </td>
                  </tr>
                </tbody>

                <tfoot>
                  <tr class="bg-[rgba(244,237,220,0.9)]">
                    <th
                      class="border-t border-[color:var(--ledger-line)] px-4 py-4 text-left text-[11px] uppercase tracking-[0.28em] text-[var(--ledger-text)]"
                      colspan="1"
                    >
                      Grand Total (incl. General Fund)
                    </th>
                    <td class="border-t border-[color:var(--ledger-line)] px-4 py-4 text-right text-sm tabular-nums text-[var(--ledger-text)]">
                      {{ formatPHPFromCents(totalTargetCents) }}
                    </td>
                    <td class="border-t border-[color:var(--ledger-line)] px-4 py-4 text-right text-sm tabular-nums text-[var(--ledger-green)]">
                      {{ formatPHPFromCents(totalReceiptsCents) }}
                    </td>
                    <td class="border-t border-[color:var(--ledger-line)] px-4 py-4 text-right text-sm tabular-nums text-[var(--ledger-red)]">
                      {{ formatPHPFromCents(totalDisbursementsCents) }}
                    </td>
                    <td class="border-t border-[color:var(--ledger-line)] px-4 py-4 text-right text-sm font-semibold tabular-nums text-[var(--ledger-text)]">
                      {{ formatSignedMoney(endingBalanceCents) }}
                    </td>
                    <td class="border-t border-[color:var(--ledger-line)] px-4 py-4 text-right text-sm tabular-nums text-[var(--ledger-text)]">
                      {{ formatPercent(totalReceiptsCents, totalTargetCents) }}
                    </td>
                  </tr>
                </tfoot>
              </table>
            </div>
          </section>

          <section class="mt-4 grid gap-4 md:grid-cols-3 print:mt-4">
            <div class="border-t border-[color:var(--ledger-line)] pt-8">
              <div class="text-sm font-semibold uppercase tracking-[0.28em] text-[var(--ledger-text)]">
                Prepared By - Treasurer
              </div>
            </div>
            <div class="border-t border-[color:var(--ledger-line)] pt-8">
              <div class="text-sm font-semibold uppercase tracking-[0.28em] text-[var(--ledger-text)]">
                Audited By - Finance Committee
              </div>
            </div>
            <div class="border-t border-[color:var(--ledger-line)] pt-8">
              <div class="text-sm font-semibold uppercase tracking-[0.28em] text-[var(--ledger-text)]">
                Approved By - Pastor
              </div>
            </div>
          </section>
        </template>

        <div v-else class="mt-4 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-4 py-6 text-sm text-[var(--ledger-muted)]">
          Generating report...
        </div>
      </div>
    </section>
  </div>
</template>
