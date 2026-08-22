<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { donationsList, expensesList, exportCsv, ledgerSummary, projectBalances, projectsList } from "../../lib/api";
import { formatPHPFromCents } from "../../lib/money";
import type { Donation, Expense, LedgerSummary, Project, ProjectBalanceRow } from "../../lib/types";

type JournalEntry = {
  id: string;
  date: string;
  particulars: string;
  ref: string;
  projectName: string;
  creditCents: number;
  debitCents: number;
};

const props = defineProps<{ sessionToken: string }>();
const emit = defineEmits<{
  (e: "create-project"): void;
  (e: "open-project", id: number): void;
  (e: "open-projects"): void;
  (e: "open-donations"): void;
  (e: "open-expenses"): void;
  (e: "open-create-record"): void;
}>();

const loading = ref(true);
const summary = ref<LedgerSummary | null>(null);
const balances = ref<ProjectBalanceRow[]>([]);
const projects = ref<Project[]>([]);
const donations = ref<Donation[]>([]);
const expenses = ref<Expense[]>([]);
const errorMessage = ref<string | null>(null);

const projectMap = computed(() => new Map(projects.value.map((project) => [project.id, project])));

const openProjectsCount = computed(() => projects.value.filter((project) => project.status !== "completed").length);
const receiptsPosted = computed(() => donations.value.length);
const vouchersIssued = computed(() => expenses.value.length);
const journalEntries = computed<JournalEntry[]>(() => {
  const donationRows: JournalEntry[] = donations.value.map((item) => {
    const project = item.project_id ? projectMap.value.get(item.project_id) : null;
    const particulars = item.anonymous ? "Anonymous" : item.donor_id ? `Donor #${item.donor_id}` : "Contribution";
    return {
      id: `donation-${item.id}`,
      date: item.donated_at,
      particulars,
      ref: `OR-${String(item.id).padStart(5, "0")}`,
      projectName: project?.name || (item.project_id ? `#${item.project_id}` : "-"),
      creditCents: item.amount_cents,
      debitCents: 0,
    };
  });

  const expenseRows: JournalEntry[] = expenses.value.map((item) => {
    const project = item.project_id ? projectMap.value.get(item.project_id) : null;
    return {
      id: `expense-${item.id}`,
      date: item.spent_at,
      particulars: item.payee?.trim() || "Expense",
      ref: `DV-${String(item.id).padStart(5, "0")}`,
      projectName: project?.name || (item.project_id ? `#${item.project_id}` : "-"),
      creditCents: 0,
      debitCents: item.amount_cents,
    };
  });

  return [...donationRows, ...expenseRows]
    .sort((a, b) => b.date.localeCompare(a.date) || b.id.localeCompare(a.id))
    .slice(0, 8);
});

const balanceRows = computed(() =>
  balances.value.map((row) => {
    const project = projectMap.value.get(row.project_id);
    return {
      ...row,
      projectName: project?.name || row.project_name,
      targetCents: project?.target_amount_cents ?? 0,
      status: project?.status || "active",
    };
  }),
);

function formatPercent(numerator: number, denominator: number) {
  if (denominator <= 0) return "0%";
  return `${Math.min(100, Math.round((numerator / denominator) * 100))}%`;
}

function projectCode(projectId: number) {
  return `PRJ-${String(projectId).padStart(3, "0")}`;
}

function progressWidth(row: { donations_cents: number; targetCents: number }) {
  if (row.targetCents <= 0) return 0;
  return Math.min(100, Math.round((row.donations_cents / row.targetCents) * 100));
}

async function exportLedgerCsv() {
  const dest = await save({
    defaultPath: "ledger-overview.csv",
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!dest) return;
  await exportCsv(props.sessionToken, {
    kind: "donations",
    filter: { from: null, to: null, project_id: null },
    dest_path: dest,
  });
}

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    const [summaryValue, balancesValue, projectsValue, donationsValue, expensesValue] = await Promise.all([
      ledgerSummary(props.sessionToken, { from: null, to: null, project_id: null }),
      projectBalances(props.sessionToken, { from: null, to: null, project_id: null }),
      projectsList(props.sessionToken),
      donationsList(props.sessionToken, { from: null, to: null, project_id: null }),
      expensesList(props.sessionToken, { from: null, to: null, project_id: null }),
    ]);
    summary.value = summaryValue;
    balances.value = balancesValue;
    projects.value = projectsValue;
    donations.value = donationsValue;
    expenses.value = expensesValue;
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(load);
</script>

<template>
  <div class="space-y-5">
    <section class="ledger-panel rounded-[26px] px-6 py-6 md:px-8">
      <div class="flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between">
        <div class="max-w-3xl">
          <div class="ledger-eyebrow text-[11px] text-[#8d6f2f]">Book of Accounts - FY 2026</div>
          <h2 class="ledger-heading mt-2 text-4xl font-normal text-[#1f3558] md:text-5xl">
            Ledger Overview
          </h2>
          <p class="mt-3 max-w-2xl text-[15px] leading-7 text-[#6a6b5d]">
            Consolidated position of all project and general funds.
          </p>
        </div>

        <div class="flex flex-wrap gap-3">
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-xl border border-[#d7c49a] bg-[#fbf7eb] px-4 py-3 text-sm font-semibold text-[#243858] transition hover:bg-[#f4ecd7]"
            @click="exportLedgerCsv"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <path d="M12 3v12" />
              <path d="m8 11 4 4 4-4" />
              <path d="M4 21h16" />
            </svg>
            Export CSV
          </button>
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-xl border border-[#243858] bg-[#243858] px-4 py-3 text-sm font-semibold text-white transition hover:bg-[#1f2f4a]"
            @click="emit('open-create-record')"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <path d="M12 5v14" />
              <path d="M5 12h14" />
            </svg>
            New Record
          </button>
        </div>
      </div>
    </section>

    <div v-if="errorMessage" class="rounded-2xl border border-[#e3b2a3] bg-[#fff4ef] px-4 py-3 text-[#9d3f27]">
      {{ errorMessage }}
    </div>

    <div v-if="loading" class="rounded-2xl border border-[#d7c49a] bg-[#fbf7eb] px-4 py-6 text-[#6a6b5d]">
      Loading...
    </div>

    <template v-else>
      <section class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-4">
        <article class="ledger-card rounded-[18px] px-5 py-5">
          <div class="ledger-eyebrow text-[11px] text-[#34527f]">Total Contributions</div>
          <div class="mt-4 text-[32px] font-semibold tracking-tight text-[var(--ledger-green)]">
            {{ formatPHPFromCents(summary?.total_donations_cents || 0) }}
          </div>
          <div class="mt-3 text-sm text-[#6a6b5d]">{{ receiptsPosted }} receipts posted</div>
        </article>

        <article class="ledger-card rounded-[18px] px-5 py-5">
          <div class="ledger-eyebrow text-[11px] text-[#34527f]">Total Disbursements</div>
          <div class="mt-4 text-[32px] font-semibold tracking-tight text-[var(--ledger-red)]">
            {{ formatPHPFromCents(summary?.total_expenses_cents || 0) }}
          </div>
          <div class="mt-3 text-sm text-[#6a6b5d]">{{ vouchersIssued }} vouchers issued</div>
        </article>

        <article class="ledger-card rounded-[18px] px-5 py-5">
          <div class="ledger-eyebrow text-[11px] text-[#34527f]">Fund Balance</div>
          <div class="mt-4 text-[32px] font-semibold tracking-tight text-[#1f3558]">
            {{ formatPHPFromCents(summary?.balance_cents || 0) }}
          </div>
          <div class="mt-3 text-sm text-[#6a6b5d]">Cash on hand + bank</div>
        </article>

        <article class="ledger-card rounded-[18px] px-5 py-5">
          <div class="ledger-eyebrow text-[11px] text-[#34527f]">Open Projects</div>
          <div class="mt-4 text-[32px] font-semibold tracking-tight text-[#1f3558]">
            {{ openProjectsCount }}
          </div>
          <div class="mt-3 text-sm text-[#6a6b5d]">{{ openProjectsCount }} projects on file</div>
        </article>
      </section>

      <section class="grid gap-4 xl:grid-cols-[1.45fr_1fr]">
        <article class="ledger-panel overflow-hidden rounded-[22px]">
          <div class="flex items-center justify-between border-b ledger-rule px-5 py-4">
            <div>
              <h3 class="ledger-heading text-2xl font-normal text-[#1f3558]">General Journal - Recent Entries</h3>
            </div>
            <div class="ledger-eyebrow text-[11px] text-[#34527f]">Page 14</div>
          </div>

          <div class="overflow-x-auto">
            <table class="w-full min-w-[760px] text-sm">
              <thead>
                <tr class="border-b ledger-rule bg-[#f3ebd7] text-[#3f5d89]">
                  <th class="px-5 py-3 text-left font-medium ledger-eyebrow text-[11px]">Date</th>
                  <th class="px-5 py-3 text-left font-medium ledger-eyebrow text-[11px]">Particulars</th>
                  <th class="px-5 py-3 text-left font-medium ledger-eyebrow text-[11px]">Ref.</th>
                  <th class="px-5 py-3 text-left font-medium ledger-eyebrow text-[11px]">Project</th>
                  <th class="px-5 py-3 text-right font-medium ledger-eyebrow text-[11px]">Credit</th>
                  <th class="px-5 py-3 text-right font-medium ledger-eyebrow text-[11px]">Debit</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="entry in journalEntries"
                  :key="entry.id"
                  class="border-b border-[#eadfbd] bg-[#fbf7eb] transition hover:bg-[#f7f1e0]"
                >
                  <td class="px-5 py-4 whitespace-nowrap text-[#1f3558]">{{ entry.date }}</td>
                  <td class="px-5 py-4 text-[#1f3558]">{{ entry.particulars }}</td>
                  <td class="px-5 py-4 whitespace-nowrap text-[#6a6b5d]">{{ entry.ref }}</td>
                  <td class="px-5 py-4 text-[#1f3558]">{{ entry.projectName }}</td>
                  <td class="px-5 py-4 text-right tabular-nums text-[var(--ledger-green)]">
                    <span v-if="entry.creditCents > 0">{{ formatPHPFromCents(entry.creditCents) }}</span>
                    <span v-else>-</span>
                  </td>
                  <td class="px-5 py-4 text-right tabular-nums text-[var(--ledger-red)]">
                    <span v-if="entry.debitCents > 0">{{ formatPHPFromCents(entry.debitCents) }}</span>
                    <span v-else>-</span>
                  </td>
                </tr>
                <tr v-if="journalEntries.length === 0">
                  <td colspan="6" class="px-5 py-8 text-center text-[#6a6b5d]">No entries yet.</td>
                </tr>
              </tbody>
            </table>
          </div>
        </article>

        <article class="ledger-panel overflow-hidden rounded-[22px]">
          <div class="flex items-center justify-between border-b ledger-rule px-5 py-4">
            <div>
              <h3 class="ledger-heading text-2xl font-normal text-[#1f3558]">Project Balances</h3>
            </div>
            <button
              type="button"
              class="text-sm text-[#34527f] underline decoration-[#c8b17e] underline-offset-4 transition hover:text-[#1f3558]"
              @click="emit('open-projects')"
            >
              View all
            </button>
          </div>

          <div class="divide-y divide-[#eadfbd]">
            <button
              v-for="row in balanceRows.slice(0, 3)"
              :key="row.project_id"
              type="button"
              class="block w-full text-left bg-[#fbf7eb] px-5 py-5 transition hover:bg-[#f7f1e0]"
              @click="emit('open-project', row.project_id)"
            >
              <div class="flex items-start justify-between gap-3">
                <div>
                  <div class="text-[22px] font-semibold text-[#1f3558]">
                    {{ row.projectName }}
                  </div>
                  <div class="mt-1 text-xs uppercase tracking-[0.22em] text-[#7f7a67]">
                    {{ projectCode(row.project_id) }}
                  </div>
                </div>
                <span
                  class="rounded-full border px-3 py-1 text-xs font-semibold uppercase tracking-[0.2em]"
                  :class="row.status === 'completed'
                    ? 'border-[#d7c49a] bg-[#f5ecda] text-[#8d6f2f]'
                    : 'border-[#9ec8a9] bg-[#f1faf3] text-[#2b7a49]'"
                >
                  {{ row.status }}
                </span>
              </div>

              <div class="mt-5 h-2 rounded-full bg-[#e6dcc2]">
                <div
                  class="h-2 rounded-full bg-[#1f6b43]"
                  :style="{ width: `${progressWidth(row)}%` }"
                ></div>
              </div>

              <div class="mt-3 flex items-end justify-between gap-3 text-sm">
                <div class="font-mono text-[#1f6b43]">{{ formatPHPFromCents(row.donations_cents) }}</div>
                <div class="text-center text-[#6a6b5d]">of {{ formatPHPFromCents(row.targetCents) }}</div>
                <div class="font-semibold text-[#1f3558]">{{ formatPercent(row.donations_cents, row.targetCents) }}</div>
              </div>
            </button>
          </div>

          <div class="border-t ledger-rule bg-[#fbf7eb] px-5 py-4">
            <div class="flex items-center justify-between gap-3">
              <div class="ledger-eyebrow text-[11px] text-[#34527f]">Quick Entry</div>
              <div class="flex gap-2">
                <button
                  type="button"
                  class="rounded-xl border border-[#d7c49a] bg-[#fbf7eb] px-4 py-2 text-sm font-semibold text-[#243858] transition hover:bg-[#f4ecd7]"
                  @click="emit('open-create-record')"
                >
                  New Record
                </button>
                <button
                  type="button"
                  class="rounded-xl border border-[#d7c49a] bg-[#fbf7eb] px-4 py-2 text-sm font-semibold text-[#243858] transition hover:bg-[#f4ecd7]"
                  @click="emit('open-expenses')"
                >
                  Expense
                </button>
              </div>
            </div>
          </div>
        </article>
      </section>
    </template>

    <button
      type="button"
      class="fixed bottom-6 right-6 flex h-14 w-14 items-center justify-center rounded-2xl bg-[#243858] text-white shadow-2xl shadow-[#243858]/25 transition hover:bg-[#1d2c45]"
      title="New record"
      @click="emit('open-create-record')"
    >
      <svg class="h-8 w-8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.25" aria-hidden="true">
        <path d="M12 5v14" />
        <path d="M5 12h14" />
      </svg>
    </button>
  </div>
</template>
