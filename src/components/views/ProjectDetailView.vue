<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { ArrowLeft, FileText, Plus, Printer } from "lucide-vue-next";
import {
  donorsCreate,
  donorsList,
  donationsCreate,
  exportPdf,
  projectReport,
} from "../../lib/api";
import { notify } from "../../lib/feedback";
import { centsFromPesos, formatPHPFromCents } from "../../lib/money";
import type { Donor, ProjectReport } from "../../lib/types";
import Dialog from "../ui/Dialog.vue";

type RegisterRow = {
  id: string;
  name: string;
  cells: Record<string, number>;
  total: number;
};

type RegisterEntry = {
  id: number;
  date: string;
  donor: string;
  amount: number;
  reference: string;
};

type DisbursementEntry = {
  id: number;
  date: string;
  payee: string;
  voucher: string;
  amount: number;
};

const props = defineProps<{
  sessionToken: string;
  projectId: number;
  themeMode: "light" | "dark";
  backLabel: string;
}>();

const emit = defineEmits<{
  (e: "back"): void;
}>();

const loading = ref(true);
const errorMessage = ref<string | null>(null);
const lastSyncedAt = ref<string | null>(null);
const report = ref<ProjectReport | null>(null);
const donors = ref<Donor[]>([]);

const filterFrom = ref<string>("");
const filterTo = ref<string>("");

const showRegisterDialog = ref(false);
const savingContribution = ref(false);
const contributionDate = ref(localToday());
const contributionName = ref("");
const contributionAmount = ref("");
const contributionNotes = ref("");
const contributionAnonymous = ref(false);

const project = computed(() => report.value?.project || null);
const projectCode = computed(() => `PRJ-${String(props.projectId).padStart(3, "0")}`);
const projectRange = computed(() => {
  const start = project.value?.start_date ? formatPrettyDate(project.value.start_date) : "Jan 12, 2026";
  const end = project.value?.end_date ? formatPrettyDate(project.value.end_date) : "Sep 30, 2026";
  return `${start} - ${end}`;
});

const receivedCents = computed(() => report.value?.donations_cents || 0);
const disbursedCents = computed(() => report.value?.expenses_cents || 0);
const balanceCents = computed(() => report.value?.balance_cents || 0);
const targetCents = computed(() => report.value?.target_amount_cents || 0);
const projectStatus = computed(() => project.value?.status || "active");

const donationEntries = computed<RegisterEntry[]>(() =>
  (report.value?.donations || [])
    .slice()
    .sort((a, b) => a.donated_at.localeCompare(b.donated_at) || a.id - b.id)
    .map((item) => ({
      id: item.id,
      date: item.donated_at,
      donor: item.anonymous ? "Anonymous" : item.donor_name?.trim() || "Unnamed contributor",
      amount: item.amount_cents,
      reference: `OR-${String(item.id).padStart(5, "0")}`,
    })),
);

const disbursementEntries = computed<DisbursementEntry[]>(() =>
  (report.value?.expenses || [])
    .slice()
    .sort((a, b) => b.spent_at.localeCompare(a.spent_at) || b.id - a.id)
    .map((item) => ({
      id: item.id,
      date: item.spent_at,
      payee: item.payee?.trim() || item.category_name?.trim() || "Expense",
      voucher: `DV-${String(item.id).padStart(4, "0")}`,
      amount: item.amount_cents,
    })),
);

const registerDates = computed(() => {
  const dates = Array.from(new Set(donationEntries.value.map((entry) => entry.date))).sort((a, b) => a.localeCompare(b));
  return dates.slice(-5);
});

const registerRows = computed<RegisterRow[]>(() => {
  const rows = new Map<string, RegisterRow>();

  for (const entry of donationEntries.value) {
    const key = entry.donor.toLowerCase();
    const existing = rows.get(key);
    if (!existing) {
      rows.set(key, {
        id: key,
        name: entry.donor,
        cells: { [entry.date]: entry.amount },
        total: entry.amount,
      });
      continue;
    }

    existing.total += entry.amount;
    existing.cells[entry.date] = (existing.cells[entry.date] || 0) + entry.amount;
  }

  return Array.from(rows.values()).sort((a, b) => a.name.localeCompare(b.name));
});

const columnTotals = computed(() => {
  const totals = new Map<string, number>();
  for (const date of registerDates.value) {
    totals.set(date, 0);
  }
  for (const row of registerRows.value) {
    for (const date of registerDates.value) {
      totals.set(date, (totals.get(date) || 0) + (row.cells[date] || 0));
    }
  }
  return totals;
});

const registerTotal = computed(() => donationEntries.value.reduce((sum, entry) => sum + entry.amount, 0));
function localToday() {
  return new Date().toISOString().slice(0, 10);
}

function formatPrettyDate(value: string) {
  const parsed = new Date(`${value}T00:00:00`);
  if (Number.isNaN(parsed.getTime())) return value;
  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(parsed);
}

function formatShortDate(value: string) {
  const parsed = new Date(`${value}T00:00:00`);
  if (Number.isNaN(parsed.getTime())) return value;
  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "2-digit",
    year: "numeric",
  }).format(parsed);
}

function formatSignedMoney(cents: number) {
  const formatted = formatPHPFromCents(Math.abs(cents));
  return cents < 0 ? `(${formatted})` : formatted;
}

function formatPercent(numerator: number, denominator: number) {
  if (denominator <= 0) return "0%";
  return `${Math.min(100, Math.round((numerator / denominator) * 100))}%`;
}

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    const [donorRows, reportValue] = await Promise.all([
      donorsList(props.sessionToken),
      projectReport(props.sessionToken, props.projectId, {
        from: filterFrom.value || null,
        to: filterTo.value || null,
        project_id: props.projectId,
      }),
    ]);
    donors.value = donorRows;
    report.value = reportValue;
    lastSyncedAt.value = new Date().toLocaleString();
  } catch (error: any) {
    errorMessage.value = String(error);
  } finally {
    loading.value = false;
  }
}

async function openPrint() {
  window.print();
}

async function exportProjectPdf() {
  const dest = await save({
    defaultPath: `project-${props.projectId}-summary.pdf`,
    filters: [{ name: "PDF", extensions: ["pdf"] }],
  });
  if (!dest) return;

  await exportPdf(props.sessionToken, {
    title: `Project Funds Tracker - ${project.value?.name || "Project"} Summary`,
    filter: {
      from: filterFrom.value || null,
      to: filterTo.value || null,
      project_id: props.projectId,
    },
    dest_path: dest,
  });
  notify("Project PDF exported.");
}

function openRegisterDialog() {
  contributionDate.value = localToday();
  contributionName.value = "";
  contributionAmount.value = "";
  contributionNotes.value = "";
  contributionAnonymous.value = false;
  showRegisterDialog.value = true;
}

async function submitContribution() {
  errorMessage.value = null;
  savingContribution.value = true;
  try {
    const amountCents = centsFromPesos(contributionAmount.value);
    if (amountCents <= 0) {
      notify("Contribution amount must be greater than 0.");
      return;
    }

    if (!contributionAnonymous.value && !contributionName.value.trim()) {
      notify("Enter a contributor name first.");
      return;
    }

    if (!confirm("Save this contribution?")) return;

    let donorId: number | null = null;
    if (!contributionAnonymous.value) {
      const donorName = contributionName.value.trim();
      const existing = donors.value.find((donor) => donor.name.trim().toLowerCase() === donorName.toLowerCase());
      if (existing) {
        donorId = existing.id;
      } else {
        const created = await donorsCreate(props.sessionToken, { name: donorName, notes: null });
        donorId = created.id;
        donors.value = [
          ...donors.value,
          { id: created.id, name: donorName, notes: null, created_at: new Date().toISOString() },
        ];
      }
    }

    await donationsCreate(props.sessionToken, {
      donated_at: contributionDate.value,
      amount_cents: amountCents,
      donor_id: contributionAnonymous.value ? null : donorId,
      anonymous: contributionAnonymous.value,
      notes: contributionNotes.value.trim() || null,
      project_id: props.projectId,
    });

    await load();
    showRegisterDialog.value = false;
    notify("Contribution saved.");
  } catch (error: any) {
    errorMessage.value = String(error);
  } finally {
    savingContribution.value = false;
  }
}

onMounted(load);
</script>

<template>
  <div class="space-y-5 text-[var(--ledger-text)]">
    <section class="ledger-panel overflow-hidden rounded-[26px]">
      <div class="flex flex-col gap-5 border-b border-[color:var(--ledger-line)] px-6 py-5 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <nav class="mb-2 flex items-center gap-2 text-[11px] uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
            <button
              type="button"
              class="transition hover:text-[var(--ledger-text)]"
              @click="emit('back')"
            >
              {{ backLabel }}
            </button>
            <span>/</span>
            <span class="text-[var(--ledger-text)]">Project detail</span>
          </nav>
          <p class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">
            BOOK OF ACCOUNTS · FY 2026
          </p>
          <h2 class="ledger-heading mt-2 text-4xl text-[var(--ledger-text)]">
            {{ project?.name || "Project" }}
          </h2>
          <p class="mt-2 text-sm text-[var(--ledger-muted)]">
            {{ projectCode }} · {{ projectRange }}
          </p>
        </div>

        <div class="flex flex-wrap gap-2 print:hidden">
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-[12px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-4 py-3 text-sm font-semibold text-[var(--ledger-text)] transition hover:bg-[rgba(255,255,255,0.95)]"
            @click="emit('back')"
          >
            <ArrowLeft class="h-4 w-4" />
            <span>Register</span>
          </button>
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-[12px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-4 py-3 text-sm font-semibold text-[var(--ledger-text)] transition hover:bg-[rgba(255,255,255,0.95)]"
            @click="openPrint"
          >
            <Printer class="h-4 w-4" />
            <span>Print</span>
          </button>
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-[12px] border border-[var(--ledger-red)] bg-[var(--ledger-red)] px-4 py-3 text-sm font-semibold text-white transition hover:bg-[#a73d24]"
            @click="exportProjectPdf"
          >
            <FileText class="h-4 w-4" />
            <span>Export PDF</span>
          </button>
        </div>
      </div>

      <div class="px-4 py-4">
        <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
          <article class="ledger-card rounded-[4px] p-5">
            <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">Target</div>
            <div class="mt-4 text-[28px] font-semibold tracking-tight text-[var(--ledger-text)]">
              {{ formatPHPFromCents(targetCents) }}
            </div>
            <div class="mt-3 text-sm uppercase tracking-[0.22em] text-[var(--ledger-muted)]">
              {{ projectStatus }}
            </div>
          </article>

          <article class="ledger-card rounded-[4px] p-5">
            <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">Received</div>
            <div class="mt-4 text-[28px] font-semibold tracking-tight text-[var(--ledger-green)]">
              {{ formatPHPFromCents(receivedCents) }}
            </div>
            <div class="mt-3 text-sm text-[var(--ledger-muted)]">
              {{ formatPercent(receivedCents, targetCents) }} of target
            </div>
          </article>

          <article class="ledger-card rounded-[4px] p-5">
            <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">Disbursed</div>
            <div class="mt-4 text-[28px] font-semibold tracking-tight text-[var(--ledger-red)]">
              {{ formatPHPFromCents(disbursedCents) }}
            </div>
            <div class="mt-3 text-sm text-[var(--ledger-muted)]">
              {{ report?.expenses?.length || 0 }} vouchers
            </div>
          </article>

          <article class="ledger-card rounded-[4px] p-5">
            <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">Balance on Hand</div>
            <div class="mt-4 text-[28px] font-semibold tracking-tight text-[var(--ledger-text)]">
              {{ formatSignedMoney(balanceCents) }}
            </div>
            <div class="mt-3 text-sm text-[var(--ledger-muted)]">
              Available for release
            </div>
          </article>
        </div>

        <section class="ledger-panel mt-4 overflow-hidden rounded-[4px]">
          <div class="flex flex-col gap-4 border-b border-[color:var(--ledger-line)] px-4 py-4 lg:flex-row lg:items-end lg:justify-between">
            <div>
              <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">
                FORM 2-A · CONTRIBUTION REGISTER
              </div>
              <h3 class="ledger-heading mt-2 text-2xl text-[var(--ledger-text)]">
                Record of Contributions by Member
              </h3>
              <p class="mt-1 text-sm text-[var(--ledger-muted)]">
                {{ project?.name || "Project" }} - amounts in Philippine Peso
              </p>
            </div>

            <div class="flex items-center gap-3">
              <span class="rounded-full border border-[color:#9ec8a9] bg-[rgba(241,250,243,0.95)] px-3 py-1 text-xs font-semibold uppercase tracking-[0.2em] text-[var(--ledger-green)]">
                Active
              </span>
              <button
                type="button"
                class="inline-flex items-center gap-2 rounded-[12px] border border-[var(--ledger-navy)] bg-[var(--ledger-navy)] px-4 py-3 text-sm font-semibold text-white transition hover:bg-[var(--ledger-navy-2)]"
                @click="openRegisterDialog"
              >
                <Plus class="h-4 w-4" />
                <span>Post entry</span>
              </button>
            </div>
          </div>

          <div v-if="errorMessage" class="mx-4 mt-4 rounded-[4px] border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
            {{ errorMessage }}
          </div>

          <div v-else-if="loading" class="px-4 py-8 text-sm text-[var(--ledger-muted)]">
            Loading project register...
          </div>

          <div v-else class="overflow-x-auto">
            <table class="w-full min-w-[980px] border-separate border-spacing-0">
              <thead>
                <tr class="bg-[rgba(240,229,203,0.85)] text-[11px] uppercase tracking-[0.28em] text-[var(--ledger-text)]">
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Contributor</th>
                  <th
                    v-for="date in registerDates"
                    :key="date"
                    class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-right font-semibold whitespace-nowrap"
                  >
                    {{ formatShortDate(date) }}
                  </th>
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-right font-semibold">Total</th>
                </tr>
              </thead>

              <tbody>
                <tr
                  v-for="row in registerRows"
                  :key="row.id"
                  class="bg-[rgba(251,247,235,0.92)] transition hover:bg-[rgba(247,241,224,0.95)]"
                >
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 font-medium text-[var(--ledger-text)]">
                    {{ row.name }}
                  </td>
                  <td
                    v-for="date in registerDates"
                    :key="`${row.id}:${date}`"
                    class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-right font-mono text-[var(--ledger-green)]"
                  >
                    <span v-if="row.cells[date]">{{ formatPHPFromCents(row.cells[date]) }}</span>
                    <span v-else class="text-[var(--ledger-muted)]">-</span>
                  </td>
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-right font-semibold tabular-nums text-[var(--ledger-text)]">
                    {{ formatPHPFromCents(row.total) }}
                  </td>
                </tr>

                <tr v-if="registerRows.length === 0">
                  <td :colspan="registerDates.length + 2" class="px-4 py-10 text-center text-sm text-[var(--ledger-muted)]">
                    No contributions recorded yet.
                  </td>
                </tr>
              </tbody>

              <tfoot>
                <tr class="bg-[rgba(244,237,220,0.9)]">
                  <th class="border-t border-[color:var(--ledger-line)] px-4 py-4 text-left text-[11px] uppercase tracking-[0.28em] text-[var(--ledger-text)]">
                    Column Total
                  </th>
                  <td
                    v-for="date in registerDates"
                    :key="`total:${date}`"
                    class="border-t border-[color:var(--ledger-line)] px-4 py-4 text-right font-semibold tabular-nums text-[var(--ledger-text)]"
                  >
                    {{ formatPHPFromCents(columnTotals.get(date) || 0) }}
                  </td>
                  <td class="border-t border-[color:var(--ledger-line)] px-4 py-4 text-right text-lg font-semibold tabular-nums text-[var(--ledger-text)]">
                    {{ formatPHPFromCents(registerTotal) }}
                  </td>
                </tr>
              </tfoot>
            </table>
          </div>

          <div class="grid gap-4 border-t border-[color:var(--ledger-line)] px-4 py-10 md:grid-cols-3">
            <div class="border-t border-[color:var(--ledger-line)] pt-8">
              <div class="text-sm font-semibold uppercase tracking-[0.28em] text-[var(--ledger-text)]">
                Prepared By
              </div>
            </div>
            <div class="border-t border-[color:var(--ledger-line)] pt-8">
              <div class="text-sm font-semibold uppercase tracking-[0.28em] text-[var(--ledger-text)]">
                Audited By
              </div>
            </div>
            <div class="border-t border-[color:var(--ledger-line)] pt-8">
              <div class="text-sm font-semibold uppercase tracking-[0.28em] text-[var(--ledger-text)]">
                Noted By
              </div>
            </div>
          </div>
        </section>

        <div class="grid gap-4 xl:grid-cols-2">
          <section class="ledger-panel overflow-hidden rounded-[4px]">
            <div class="border-b border-[color:var(--ledger-line)] px-4 py-4">
              <h3 class="ledger-heading text-2xl text-[var(--ledger-text)]">
                Contributions Detail
              </h3>
            </div>

            <div class="overflow-x-auto">
              <table class="w-full min-w-[720px] border-separate border-spacing-0">
                <thead>
                  <tr class="bg-[rgba(240,229,203,0.85)] text-[11px] uppercase tracking-[0.28em] text-[var(--ledger-text)]">
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Date</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Donor</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">OR / Ref</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-right font-semibold">Amount</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="entry in donationEntries.slice().reverse().slice(0, 8)"
                    :key="entry.id"
                    class="bg-[rgba(251,247,235,0.92)]"
                  >
                    <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 whitespace-nowrap text-sm text-[var(--ledger-text)]">
                      {{ formatShortDate(entry.date) }}
                    </td>
                    <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-sm font-medium text-[var(--ledger-text)]">
                      {{ entry.donor }}
                    </td>
                    <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-sm text-[var(--ledger-muted)]">
                      {{ entry.reference }}
                    </td>
                    <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-right text-sm font-semibold text-[var(--ledger-green)]">
                      {{ formatPHPFromCents(entry.amount) }}
                    </td>
                  </tr>

                  <tr v-if="donationEntries.length === 0">
                    <td colspan="4" class="px-4 py-10 text-center text-sm text-[var(--ledger-muted)]">
                      No contributions found.
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </section>

          <section class="ledger-panel overflow-hidden rounded-[4px]">
            <div class="border-b border-[color:var(--ledger-line)] px-4 py-4">
              <h3 class="ledger-heading text-2xl text-[var(--ledger-text)]">
                Disbursements Detail
              </h3>
            </div>

            <div class="overflow-x-auto">
              <table class="w-full min-w-[720px] border-separate border-spacing-0">
                <thead>
                  <tr class="bg-[rgba(240,229,203,0.85)] text-[11px] uppercase tracking-[0.28em] text-[var(--ledger-text)]">
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Date</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Payee</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Voucher</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-right font-semibold">Amount</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="entry in disbursementEntries.slice(0, 8)"
                    :key="entry.id"
                    class="bg-[rgba(251,247,235,0.92)]"
                  >
                    <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 whitespace-nowrap text-sm text-[var(--ledger-text)]">
                      {{ formatShortDate(entry.date) }}
                    </td>
                    <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-sm font-medium text-[var(--ledger-text)]">
                      {{ entry.payee }}
                    </td>
                    <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-sm text-[var(--ledger-muted)]">
                      {{ entry.voucher }}
                    </td>
                    <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-right text-sm font-semibold text-[var(--ledger-red)]">
                      {{ formatPHPFromCents(entry.amount) }}
                    </td>
                  </tr>

                  <tr v-if="disbursementEntries.length === 0">
                    <td colspan="4" class="px-4 py-10 text-center text-sm text-[var(--ledger-muted)]">
                      No disbursements found.
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </section>
        </div>
      </div>

      <div class="border-t border-[color:var(--ledger-line)] px-6 py-3 text-xs text-[var(--ledger-muted)]">
        Auto-sync: {{ lastSyncedAt || "Pending" }} · Balance {{ formatSignedMoney(balanceCents) }}
      </div>
    </section>

    <Dialog
      v-model:open="showRegisterDialog"
      :theme-mode="themeMode"
      title="Post Entry"
      description="Add a contribution to this project register."
    >
      <div class="grid gap-4">
        <label class="grid gap-2">
          <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">Date</span>
          <input
            v-model="contributionDate"
            type="date"
            class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition focus:border-[color:var(--ledger-gold)]"
          />
        </label>

        <label class="grid gap-2">
          <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">Contributor</span>
          <input
            v-model="contributionName"
            :disabled="contributionAnonymous"
            list="project-donors"
            placeholder="Name of contributor"
            class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)] disabled:opacity-60"
          />
          <datalist id="project-donors">
            <option v-for="donor in donors" :key="donor.id" :value="donor.name" />
          </datalist>
        </label>

        <label class="flex items-center gap-2 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 py-3">
          <input v-model="contributionAnonymous" type="checkbox" class="accent-[var(--ledger-navy)]" />
          <span class="text-sm text-[var(--ledger-text)]">Anonymous</span>
        </label>

        <label class="grid gap-2">
          <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">Amount (PHP)</span>
          <input
            v-model="contributionAmount"
            inputmode="decimal"
            placeholder="0.00"
            class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-right text-sm font-medium text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)]"
          />
        </label>

        <label class="grid gap-2">
          <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">Notes</span>
          <textarea
            v-model="contributionNotes"
            rows="3"
            placeholder="Optional memo"
            class="min-h-24 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 py-3 text-sm text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)]"
          ></textarea>
        </label>

        <div class="flex items-center justify-end gap-2 pt-2">
          <button
            type="button"
            class="rounded-[4px] border px-3 py-2 text-sm font-semibold"
            :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100 hover:bg-slate-700' : 'border-slate-300 bg-white text-slate-900 hover:bg-slate-50'"
            @click="showRegisterDialog = false"
          >
            Cancel
          </button>
          <button
            type="button"
            class="rounded-[4px] border px-3 py-2 text-sm font-semibold"
            :class="themeMode === 'dark' ? 'border-blue-500 bg-blue-500 text-white hover:bg-blue-400' : 'border-blue-600 bg-blue-600 text-white hover:bg-blue-500'"
            :disabled="savingContribution"
            @click="submitContribution"
          >
            {{ savingContribution ? "Posting..." : "Save Entry" }}
          </button>
        </div>
      </div>
    </Dialog>
  </div>
</template>
