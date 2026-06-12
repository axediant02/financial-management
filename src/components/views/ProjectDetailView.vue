<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { donationsCreate, donorsCreate, donorsList, exportCsv, exportPdf, projectReport } from "../../lib/api";
import { notify } from "../../lib/feedback";
import { centsFromPesos, formatPHPFromCents } from "../../lib/money";
import type { Donor, ProjectReport } from "../../lib/types";

type MatrixDate = {
  date: string;
  note: string;
};

type CellEditor = {
  rowKey: string;
  date: string;
};

const props = defineProps<{
  sessionToken: string;
  projectId: number;
}>();

const emit = defineEmits<{
  (e: "back"): void;
}>();

const filterFrom = ref<string>("");
const filterTo = ref<string>("");
const memberSearch = ref("");
const boardFilter = ref<"all" | "paid" | "missed" | "unpaid">("all");
const sortMode = ref<"name-asc" | "name-desc">("name-asc");
const matrixStorageKey = computed(() => `pft_project_matrix_state_${props.projectId}`);

const report = ref<ProjectReport | null>(null);
const loading = ref(true);
const errorMessage = ref<string | null>(null);
const lastSyncedAt = ref<string | null>(null);

const donors = ref<Donor[]>([]);
const plannedDates = ref<MatrixDate[]>([]);
const memberAliases = ref<Record<string, string>>({});
const warningLabel = ref("Missed");
const redHighlightTheme = ref(true);
const currencyMode = ref<"prefix" | "suffix">("prefix");
const newMatrixDate = ref("");
const newMatrixNote = ref("");
const newMemberName = ref("");

const cellEditor = ref<CellEditor | null>(null);
const cellAmount = ref("");
const cellNotes = ref("");

const showAddDonation = ref(false);
const addDate = ref<string>(new Date().toISOString().slice(0, 10));
const addAmount = ref("");
const addAnonymous = ref(false);
const addDonorName = ref("");
const addNotes = ref("");
const addSubmitting = ref(false);

const filter = computed(() => ({
  from: filterFrom.value || null,
  to: filterTo.value || null,
  project_id: props.projectId,
}));

type RowStatus = "paid" | "missed" | "unpaid" | "anonymous";

const rowStats = computed(() => {
  const stats = new Map<
    string,
    {
      filled: number;
      total: number;
      status: RowStatus;
    }
  >();

  for (const row of matrixRows.value) {
    const filled = projectDates.value.reduce((sum, entry) => sum + (matrixCellTotal(row.rowKey, entry.date) > 0 ? 1 : 0), 0);
    const total = matrixRowTotal(row.rowKey);
    let status: RowStatus = "unpaid";

    if (row.rowKey === "__anonymous__") {
      status = total > 0 ? "anonymous" : "unpaid";
    } else if (projectDates.value.length > 0 && filled === projectDates.value.length) {
      status = "paid";
    } else if (total > 0) {
      status = "missed";
    }

    stats.set(row.rowKey, { filled, total, status });
  }

  return stats;
});

const visibleRows = computed(() => {
  const query = memberSearch.value.trim().toLowerCase();
  const rows = matrixRows.value.filter((row) => {
    const stats = rowStats.value.get(row.rowKey);
    const total = stats?.total || 0;
    if (boardFilter.value === "paid" && total <= 0) return false;
    if (boardFilter.value === "missed" && stats?.status !== "missed") return false;
    if (boardFilter.value === "unpaid" && stats?.status !== "unpaid") return false;
    if (query) {
      const haystack = `${row.displayName} ${row.alias}`.toLowerCase();
      if (!haystack.includes(query)) return false;
    }
    return true;
  });

  rows.sort((a, b) => {
    const left = (memberAliases.value[a.rowKey]?.trim() || a.alias || a.displayName).toLowerCase();
    const right = (memberAliases.value[b.rowKey]?.trim() || b.alias || b.displayName).toLowerCase();
    return sortMode.value === "name-asc" ? left.localeCompare(right) : right.localeCompare(left);
  });

  return rows;
});

const filterCounts = computed(() => {
  let all = 0;
  let paid = 0;
  let missed = 0;
  let unpaid = 0;
  for (const row of matrixRows.value) {
    const stats = rowStats.value.get(row.rowKey);
    all += 1;
    if ((stats?.total || 0) > 0) paid += 1;
    if (stats?.status === "missed") missed += 1;
    if (stats?.status === "unpaid") unpaid += 1;
  }
  return { all, paid, missed, unpaid };
});

function formatBoardMoney(cents: number) {
  const amount = (cents / 100).toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
  return currencyMode.value === "prefix" ? `PHP ${amount}` : `${amount} PHP`;
}

const progressPct = computed(() => {
  const target = report.value?.target_amount_cents || 0;
  const donated = report.value?.donations_cents || 0;
  if (target <= 0) return 0;
  return Math.min(100, Math.round((donated / target) * 100));
});

const projectTotal = computed(() => report.value?.donations_cents || 0);
const ledgerMinWidth = computed(() => 290 + Math.max(projectDates.value.length, 5) * 190 + 150);
const ledgerGridColumns = computed(() => `290px ${projectDates.value.map(() => "190px").join(" ")} 150px`);
const overallTotalLabel = computed(() => (report.value?.project.status === "completed" ? "Completed Total" : "Overall Total"));

function readStoredState() {
  try {
    const raw = localStorage.getItem(matrixStorageKey.value);
    if (!raw) return;
    const parsed = JSON.parse(raw) as Partial<{
      warningLabel: string;
      redHighlightTheme: boolean;
      currencyMode: "prefix" | "suffix";
      memberAliases: Record<string, string>;
      plannedDates: MatrixDate[];
    }>;
    warningLabel.value = typeof parsed.warningLabel === "string" && parsed.warningLabel.trim() ? parsed.warningLabel : warningLabel.value;
    redHighlightTheme.value = typeof parsed.redHighlightTheme === "boolean" ? parsed.redHighlightTheme : redHighlightTheme.value;
    currencyMode.value = parsed.currencyMode === "suffix" ? "suffix" : "prefix";
    memberAliases.value = parsed.memberAliases && typeof parsed.memberAliases === "object" ? parsed.memberAliases : {};
    plannedDates.value = Array.isArray(parsed.plannedDates)
      ? parsed.plannedDates
          .map((value) => ({
            date: typeof value?.date === "string" ? value.date : "",
            note: typeof value?.note === "string" ? value.note : "Weekly dues",
          }))
          .filter((value) => value.date.trim())
      : [];
  } catch {
    plannedDates.value = [];
  }
}

function persistStoredState() {
  localStorage.setItem(
    matrixStorageKey.value,
    JSON.stringify({
      warningLabel: warningLabel.value,
      redHighlightTheme: redHighlightTheme.value,
      currencyMode: currencyMode.value,
      memberAliases: memberAliases.value,
      plannedDates: plannedDates.value,
    }),
  );
}

function loadPlannedDates() {
  readStoredState();
}

function uniqueProjectDates() {
  const map = new Map<string, MatrixDate>();
  for (const item of report.value?.donations || []) {
    if (!map.has(item.donated_at)) {
      map.set(item.donated_at, {
        date: item.donated_at,
        note: item.notes?.trim() || "Recorded contribution",
      });
    }
  }
  for (const planned of plannedDates.value) {
    if (!map.has(planned.date)) {
      map.set(planned.date, planned);
    } else if (planned.note.trim()) {
      const current = map.get(planned.date)!;
      if (current.note === "Recorded contribution") {
        current.note = planned.note.trim();
      }
    }
  }
  return Array.from(map.values()).sort((a, b) => a.date.localeCompare(b.date));
}

const projectDates = computed(() => uniqueProjectDates());

const matrixRows = computed(() => {
  const rows = donors.value
    .slice()
    .sort((a, b) => a.name.localeCompare(b.name))
    .map((donor) => ({
      rowKey: `donor:${donor.id}`,
      displayName: donor.name,
      alias: memberAliases.value[`donor:${donor.id}`]?.trim() || donor.name,
      locked: false,
    }));

  if ((report.value?.donations || []).some((item) => item.anonymous)) {
    rows.push({
      rowKey: "__anonymous__",
      displayName: "Anonymous",
      alias: "Anonymous",
      locked: true,
    });
  }

  return rows;
});

const matrixLookup = computed(() => {
  const lookup = new Map<string, Map<string, number>>();
  for (const row of matrixRows.value) {
    lookup.set(row.rowKey, new Map());
  }

  for (const item of report.value?.donations || []) {
    let rowKey = "__anonymous__";
    if (!item.anonymous && item.donor_name) {
      const donor = donors.value.find((entry) => entry.name.trim().toLowerCase() === item.donor_name?.trim().toLowerCase());
      if (donor) rowKey = `donor:${donor.id}`;
    }
    const rowMap = lookup.get(rowKey);
    if (!rowMap) continue;
    rowMap.set(item.donated_at, (rowMap.get(item.donated_at) || 0) + item.amount_cents);
  }

  return lookup;
});

const dailyTotals = computed(() =>
  projectDates.value.map((entry) => ({
    date: entry.date,
    total: (report.value?.donations || [])
      .filter((item) => item.donated_at === entry.date)
      .reduce((sum, item) => sum + item.amount_cents, 0),
  })),
);

const activeMembers = computed(() => matrixRows.value.filter((row) => row.rowKey !== "__anonymous__" && matrixRowTotal(row.rowKey) > 0).length);
const paidMembers = computed(() => donors.value.length);

const filledCells = computed(() => {
  let count = 0;
  for (const row of matrixRows.value) {
    for (const date of projectDates.value) {
      if (matrixCellTotal(row.rowKey, date) > 0) count += 1;
    }
  }
  return count;
});

const fidelityPct = computed(() => {
  const totalCells = matrixRows.value.length * projectDates.value.length;
  if (totalCells <= 0) return 0;
  return Math.round((filledCells.value / totalCells) * 100);
});

const topSupporter = computed(() => {
  const top = report.value?.top_donors?.[0];
  return {
    name: top?.donor_name || "No support yet",
    total: top?.total_cents || 0,
  };
});

function matrixCellTotal(rowKey: string, date: string) {
  return matrixLookup.value.get(rowKey)?.get(date) || 0;
}

function matrixRowTotal(rowKey: string) {
  return projectDates.value.reduce((sum, entry) => sum + matrixCellTotal(rowKey, entry.date), 0);
}

function rowStatus(rowKey: string): RowStatus {
  return rowStats.value.get(rowKey)?.status || "unpaid";
}

function rowStatusLabel(rowKey: string) {
  const stats = rowStats.value.get(rowKey);
  if (!stats) return "Unpaid";
  switch (stats.status) {
    case "paid":
      return `Paid all`;
    case "missed":
      return `Missed: ${stats.filled} / ${projectDates.value.length}`;
    case "anonymous":
      return "Anonymous";
    case "unpaid":
      return `Unpaid: 0 / ${projectDates.value.length}`;
  }
}

function dateNote(date: string) {
  return plannedDates.value.find((entry) => entry.date === date)?.note || "Recorded contribution";
}

function formatReadableDate(date: string) {
  const parsed = new Date(`${date}T00:00:00`);
  if (Number.isNaN(parsed.getTime())) return date;
  return new Intl.DateTimeFormat("en-US", {
    month: "long",
    day: "numeric",
    year: "numeric",
  }).format(parsed);
}

function openCell(rowKey: string, date: string) {
  cellEditor.value = { rowKey, date };
  cellAmount.value = "";
  cellNotes.value = "";
}

function closeCell() {
  cellEditor.value = null;
  cellAmount.value = "";
  cellNotes.value = "";
}

async function saveCell() {
  if (!cellEditor.value) return;
  const amount = Number(String(cellAmount.value).replace(/,/g, ""));
  if (!Number.isFinite(amount) || amount <= 0) {
    errorMessage.value = "Enter a valid amount greater than zero.";
    return;
  }
  if (!confirm("Save this contribution cell?")) return;
  try {
    const donorId = cellEditor.value.rowKey.startsWith("donor:") ? Number(cellEditor.value.rowKey.split(":")[1]) : null;
    await donationsCreate(props.sessionToken, {
      donated_at: cellEditor.value.date,
      amount_cents: Math.round(amount * 100),
      donor_id: donorId,
      anonymous: cellEditor.value.rowKey === "__anonymous__",
      notes: cellNotes.value.trim() || null,
      project_id: props.projectId,
    });
    closeCell();
    await load();
    notify("Contribution saved.");
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    donors.value = await donorsList(props.sessionToken);
    report.value = await projectReport(props.sessionToken, props.projectId, filter.value);
    loadPlannedDates();
    lastSyncedAt.value = new Date().toLocaleString();
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    loading.value = false;
  }
}

function openAddDonation() {
  showAddDonation.value = true;
  addDate.value = new Date().toISOString().slice(0, 10);
  addAmount.value = "";
  addAnonymous.value = false;
  addDonorName.value = "";
  addNotes.value = "";
}

async function addMemberRow() {
  errorMessage.value = null;
  const name = newMemberName.value.trim();
  if (!name) return;
  if (!confirm(`Add member "${name}" to this project matrix?`)) return;
  try {
    const existing = donors.value.find((donor) => donor.name.trim().toLowerCase() === name.toLowerCase());
    if (!existing) {
      await donorsCreate(props.sessionToken, { name, notes: null });
    }
    newMemberName.value = "";
    await load();
    notify(`Member "${name}" added.`);
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

function addDateColumn() {
  const date = newMatrixDate.value.trim();
  if (!date) return;
  if (!confirm(`Add date column ${date}?`)) return;
  if (!plannedDates.value.some((entry) => entry.date === date)) {
    plannedDates.value = [...plannedDates.value, { date, note: newMatrixNote.value.trim() || "Weekly dues" }].sort((a, b) => a.date.localeCompare(b.date));
    persistStoredState();
  }
  newMatrixDate.value = "";
  newMatrixNote.value = "";
  notify(`Date column ${date} added.`);
}

function resetGrid() {
  if (!confirm("Reset matrix settings for this project?")) return;
  warningLabel.value = "Missed";
  redHighlightTheme.value = true;
  currencyMode.value = "prefix";
  memberAliases.value = {};
  plannedDates.value = [];
  newMatrixDate.value = "";
  newMatrixNote.value = "";
  closeCell();
  persistStoredState();
  notify("Grid reset.");
}

function restoreDemoData() {
  if (!confirm("Restore demo board settings for this project?")) return;
  warningLabel.value = "Missed";
  redHighlightTheme.value = true;
  currencyMode.value = "prefix";
  memberAliases.value = {};
  plannedDates.value = [];
  newMatrixDate.value = "";
  newMatrixNote.value = "";
  closeCell();
  persistStoredState();
  notify("Demo settings restored.");
}

function setAlias(rowKey: string, value: string) {
  memberAliases.value = {
    ...memberAliases.value,
    [rowKey]: value,
  };
}

watch([warningLabel, redHighlightTheme, currencyMode, memberAliases, plannedDates], persistStoredState, { deep: true });

async function submitAddDonation() {
  errorMessage.value = null;
  addSubmitting.value = true;
  try {
    const amountCents = centsFromPesos(addAmount.value);
    if (!confirm("Save this contribution?")) return;
    let donorId: number | null = null;
    const donorName = addDonorName.value.trim();
    if (!addAnonymous.value && donorName) {
      const existing = donors.value.find((d) => d.name.trim().toLowerCase() === donorName.toLowerCase());
      if (existing) {
        donorId = existing.id;
      } else {
        const created = await donorsCreate(props.sessionToken, { name: donorName, notes: null });
        donorId = created.id;
      }
    }
    await donationsCreate(props.sessionToken, {
      donated_at: addDate.value,
      amount_cents: amountCents,
      donor_id: addAnonymous.value ? null : donorId,
      anonymous: addAnonymous.value,
      notes: addNotes.value || null,
      project_id: props.projectId,
    });
    showAddDonation.value = false;
    await load();
    notify("Contribution saved.");
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    addSubmitting.value = false;
  }
}

async function exportProjectSummaryPdf() {
  const dest = await save({
    defaultPath: `project-${props.projectId}-summary-${filterFrom.value || "all"}-${filterTo.value || "all"}.pdf`,
    filters: [{ name: "PDF", extensions: ["pdf"] }],
  });
  if (!dest) return;
  await exportPdf(props.sessionToken, {
    title: `Project Funds Tracker - ${report.value?.project.name || "Project"} Summary`,
    filter: filter.value,
    dest_path: dest,
  });
  notify("Project PDF exported.");
}

async function exportProjectDonationsCsv() {
  const dest = await save({
    defaultPath: `project-${props.projectId}-contributions-${filterFrom.value || "all"}-${filterTo.value || "all"}.csv`,
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!dest) return;
  await exportCsv(props.sessionToken, { kind: "donations", filter: filter.value, dest_path: dest });
  notify("Project contributions CSV exported.");
}

onMounted(load);
</script>

<template>
  <div class="space-y-6">
    <section class="overflow-hidden rounded-[2px] border border-slate-300 bg-slate-950 text-slate-100 shadow-sm">
      <div class="border-b-4 border-blue-600 bg-slate-900 px-6 py-5">
        <div class="flex flex-col gap-5 xl:flex-row xl:items-end xl:justify-between">
          <div>
            <p class="text-[11px] uppercase tracking-[0.4em] text-slate-400">Group Contribution Board</p>
            <h2 class="mt-2 max-w-4xl text-3xl font-semibold tracking-tight text-white md:text-4xl">
              {{ report?.project.name || "Loading..." }} contribution register
            </h2>
            <p class="mt-2 max-w-3xl text-sm text-slate-300">
              Formal class record book for one project with dated history, live totals, and editable member rows.
            </p>
          </div>

          <div class="grid gap-3 text-right sm:grid-cols-2 xl:min-w-[28rem]">
            <div class="rounded-[2px] border border-slate-300 bg-slate-950 px-4 py-3">
              <div class="text-[11px] uppercase tracking-[0.3em] text-slate-400">Total Collected</div>
              <div class="mt-2 text-2xl font-semibold text-cyan-400">{{ formatBoardMoney(projectTotal) }}</div>
            </div>
            <div class="rounded-[2px] border border-slate-300 bg-slate-950 px-4 py-3">
              <div class="text-[11px] uppercase tracking-[0.3em] text-slate-400">Target Completion</div>
              <div class="mt-2 text-2xl font-semibold text-emerald-400">{{ progressPct }}%</div>
            </div>
          </div>
        </div>
      </div>

      <div class="flex flex-col gap-3 border-b border-slate-300 bg-white px-4 py-3 text-slate-900 lg:flex-row lg:items-center lg:justify-between">
        <div class="flex flex-wrap items-center gap-2">
          <button class="rounded-[2px] border border-slate-300 bg-white px-3 py-2 text-sm font-semibold hover:bg-slate-50" @click="emit('back')">
            Back
          </button>
          <button class="rounded-[2px] border border-slate-300 bg-white px-3 py-2 text-sm font-semibold hover:bg-slate-50" @click="exportProjectDonationsCsv">
            Export CSV
          </button>
          <button class="rounded-[2px] border border-slate-300 bg-white px-3 py-2 text-sm font-semibold hover:bg-slate-50" @click="exportProjectSummaryPdf">
            Export PDF
          </button>
          <button class="rounded-[2px] border border-slate-300 bg-white px-3 py-2 text-sm font-semibold hover:bg-slate-50" @click="restoreDemoData">
            Restore Demo Data
          </button>
          <button class="rounded-[2px] border border-slate-300 bg-white px-3 py-2 text-sm font-semibold hover:bg-slate-50" @click="resetGrid">
            Reset Grid
          </button>
          <button class="rounded-[2px] border border-slate-300 bg-white px-3 py-2 text-sm font-semibold hover:bg-slate-50" @click="openAddDonation">
            + Add Contribution
          </button>
        </div>

        <div class="flex flex-wrap items-center gap-3 text-sm">
          <div class="rounded-[2px] border border-slate-300 bg-slate-50 px-3 py-2">
            Auto-sync: <span class="font-semibold">{{ lastSyncedAt || "Pending" }}</span>
          </div>
          <div class="rounded-[2px] border border-slate-300 bg-slate-50 px-3 py-2">
            Status: <span class="font-semibold">{{ report?.project.status || "-" }}</span>
          </div>
        </div>
      </div>
    </section>

    <div v-if="errorMessage" class="rounded-[2px] border border-rose-300 bg-rose-50 p-4 text-rose-700">
      {{ errorMessage }}
    </div>

    <div v-else-if="loading" class="rounded-[2px] border border-slate-300 bg-white p-4 text-slate-500">
      Loading...
    </div>

    <div v-else-if="report" class="space-y-6">
      <section class="grid gap-4 px-1 md:grid-cols-2 xl:grid-cols-5">
        <div class="rounded-[2px] border border-slate-300 bg-white p-4">
          <div class="flex items-start justify-between gap-3">
            <div>
              <div class="text-[11px] uppercase tracking-[0.3em] text-slate-500">Total Collected</div>
              <div class="mt-2 text-2xl font-semibold text-emerald-600">{{ formatBoardMoney(projectTotal) }}</div>
            </div>
            <div class="rounded-[2px] border border-emerald-200 bg-emerald-50 px-2 py-1 text-xs font-semibold text-emerald-700">
              Live
            </div>
          </div>
        </div>
        <div class="rounded-[2px] border border-slate-300 bg-white p-4">
          <div class="text-[11px] uppercase tracking-[0.3em] text-slate-500">Group Target</div>
          <div class="mt-2 text-2xl font-semibold text-blue-700">{{ formatBoardMoney(report.target_amount_cents) }}</div>
          <div class="mt-2 inline-flex rounded-[2px] border border-emerald-200 bg-emerald-50 px-2 py-1 text-xs font-semibold text-emerald-700">
            {{ progressPct }}% complete
          </div>
        </div>
        <div class="rounded-[2px] border border-slate-300 bg-white p-4">
          <div class="text-[11px] uppercase tracking-[0.3em] text-slate-500">Members Paid</div>
          <div class="mt-2 text-2xl font-semibold text-slate-900">{{ activeMembers }} / {{ paidMembers }}</div>
          <div class="mt-2 text-xs text-slate-500">Participation fraction</div>
        </div>
        <div class="rounded-[2px] border border-slate-300 bg-white p-4">
          <div class="text-[11px] uppercase tracking-[0.3em] text-slate-500">Fidelity Index</div>
          <div class="mt-2 text-2xl font-semibold text-indigo-700">{{ fidelityPct }}%</div>
          <div class="mt-2 text-xs text-slate-500">Filled cells vs. board size</div>
        </div>
        <div class="rounded-[2px] border border-slate-300 bg-white p-4">
          <div class="text-[11px] uppercase tracking-[0.3em] text-slate-500">Top Supporter</div>
          <div class="mt-2 text-2xl font-semibold text-amber-700">{{ topSupporter.name }}</div>
          <div class="mt-2 inline-flex items-center gap-2 rounded-[2px] border border-amber-200 bg-amber-50 px-2 py-1 text-xs font-semibold text-amber-800">
            <svg class="h-3.5 w-3.5 text-amber-700" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
              <path d="M3 7l5 4 4-6 4 6 5-4-2 11H5L3 7zm3 11h12v2H6z" />
            </svg>
            <span>{{ formatBoardMoney(topSupporter.total) }}</span>
          </div>
        </div>
      </section>

 

      <section class="grid gap-4 lg:grid-cols-2">
        <div class="rounded-[2px] border border-slate-300 bg-white p-4 shadow-sm">
          <div class="text-lg font-semibold text-slate-900">Membership Appender</div>
          <div class="mt-1 text-sm text-slate-500">Insert a new member row into the board.</div>
          <div class="mt-4 flex gap-2">
            <input
              v-model="newMemberName"
              placeholder="New member name"
              class="min-w-0 flex-1 rounded-[2px] border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 outline-none focus:border-blue-500"
            />
            <button class="rounded-[2px] border border-blue-600 bg-blue-600 px-4 py-2 text-sm font-semibold text-white" @click="addMemberRow">
              Add Member
            </button>
          </div>
        </div>

        <div class="rounded-[2px] border border-slate-300 bg-white p-4 shadow-sm">
          <div class="text-lg font-semibold text-slate-900">Date Column Scheduler</div>
          <div class="mt-1 text-sm text-slate-500">Preload a future date column and attach a note.</div>
          <div class="mt-4 grid gap-2 md:grid-cols-3">
            <input
              v-model="newMatrixDate"
              type="date"
              class="rounded-[2px] border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 outline-none focus:border-blue-500"
            />
            <input
              v-model="newMatrixNote"
              placeholder="Column note"
              class="rounded-[2px] border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 outline-none focus:border-blue-500 md:col-span-1"
            />
            <button class="rounded-[2px] border border-slate-900 bg-slate-900 px-4 py-2 text-sm font-semibold text-white" @click="addDateColumn">
              Add Date
            </button>
          </div>
        </div>
      </section>

      <section class="rounded-[2px] border border-slate-300 bg-slate-900 px-4 py-4 text-slate-100 shadow-sm">
        <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <div class="text-sm font-semibold uppercase tracking-[0.2em] text-slate-300">Customizer Footer</div>
            <div class="mt-1 text-sm text-slate-400">
              Cached locally in the browser so the board keeps its display preferences.
            </div>
          </div>

          <div class="flex flex-wrap items-center gap-3 text-sm">
            <label class="flex items-center gap-2 rounded-[2px] border border-slate-600 bg-slate-800 px-3 py-2">
              <span class="text-slate-300">Warning label</span>
              <input
                v-model="warningLabel"
                class="w-32 rounded-[2px] border border-slate-500 bg-slate-900 px-2 py-1 text-sm text-white outline-none"
              />
            </label>

            <label class="flex items-center gap-2 rounded-[2px] border border-slate-600 bg-slate-800 px-3 py-2">
              <input v-model="redHighlightTheme" type="checkbox" class="accent-red-500" />
              <span class="text-slate-300">Red highlight theme</span>
            </label>

            <label class="flex items-center gap-2 rounded-[2px] border border-slate-600 bg-slate-800 px-3 py-2">
              <span class="text-slate-300">Currency</span>
              <select v-model="currencyMode" class="rounded-[2px] border border-slate-500 bg-slate-900 px-2 py-1 text-white outline-none">
                <option value="prefix">Prefix</option>
                <option value="suffix">Suffix</option>
              </select>
            </label>

            <button
              type="button"
              class="rounded-[2px] border border-slate-600 bg-white px-3 py-2 font-semibold text-slate-900"
              @click="load"
            >
              Refresh
            </button>
          </div>
        </div>
      </section>
    </div>

    <div v-if="showAddDonation" class="fixed inset-0 z-50">
      <div class="absolute inset-0 bg-black/60" @click="showAddDonation = false"></div>
      <div class="absolute inset-0 flex items-center justify-center p-4">
        <div class="w-full max-w-xl rounded-2xl border border-slate-800 bg-slate-950 p-6 shadow-2xl">
          <div class="flex items-center justify-between">
            <div>
              <div class="text-lg font-bold">Add Contribution</div>
              <div class="text-sm text-slate-400">This contribution will be linked to this project.</div>
            </div>
            <button class="rounded-lg bg-slate-800 hover:bg-slate-700 px-3 py-2 text-sm font-semibold" @click="showAddDonation = false">
              Close
            </button>
          </div>

          <div class="mt-5 grid grid-cols-1 md:grid-cols-2 gap-4 text-base">
            <div>
              <div class="text-xs text-slate-400 mb-1">Date</div>
              <input v-model="addDate" type="date" class="w-full rounded-xl border border-slate-700 bg-slate-900/40 px-3 py-2" />
            </div>
            <div>
              <div class="text-xs text-slate-400 mb-1">Amount (PHP)</div>
              <input v-model="addAmount" inputmode="decimal" placeholder="0.00" class="w-full rounded-xl border border-slate-700 bg-slate-900/40 px-3 py-2" />
            </div>
            <div>
              <div class="text-xs text-slate-400 mb-1">Anonymous</div>
              <label class="flex items-center gap-2 rounded-xl border border-slate-700 bg-slate-900/40 px-3 py-2">
                <input v-model="addAnonymous" type="checkbox" class="accent-indigo-500" />
                <span class="text-sm text-slate-200">Yes</span>
              </label>
            </div>
            <div>
              <div class="text-xs text-slate-400 mb-1">Name</div>
              <input
                v-model="addDonorName"
                :disabled="addAnonymous"
                placeholder="optional"
                class="w-full rounded-xl border border-slate-700 bg-slate-900/40 px-3 py-2 disabled:opacity-60"
              />
              <div class="mt-1 text-xs text-slate-500">If this name doesn't exist yet, it will be created.</div>
            </div>
            <div class="md:col-span-2">
              <div class="text-xs text-slate-400 mb-1">Notes</div>
              <input v-model="addNotes" placeholder="optional" class="w-full rounded-xl border border-slate-700 bg-slate-900/40 px-3 py-2" />
            </div>
          </div>

          <div class="mt-6 flex items-center justify-end gap-2">
            <button class="rounded-xl bg-slate-800 hover:bg-slate-700 px-4 py-2 font-semibold" @click="showAddDonation = false">
              Cancel
            </button>
            <button
              class="rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-60 px-4 py-2 font-semibold"
              :disabled="addSubmitting"
              @click="submitAddDonation"
            >
              {{ addSubmitting ? "Saving..." : "Save Contribution" }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
