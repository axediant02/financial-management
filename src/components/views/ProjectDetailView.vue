<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { donationsCreate, donorsCreate, donorsList, exportCsv, exportPdf, projectReport } from "../../lib/api";
import { notify } from "../../lib/feedback";
import { centsFromPesos } from "../../lib/money";
import type { Donor, ProjectReport } from "../../lib/types";
import ContributionHistoryTable from "./ContributionHistoryTable.vue";
import Dialog from "../ui/Dialog.vue";
import CalendarPicker from "../ui/CalendarPicker.vue";

type MatrixDate = {
  date: string;
  note: string;
};

type ProjectHistoryDate = {
  date: string;
  total: number;
};

type ProjectHistoryCell = {
  date: string;
  amount: number;
};

type ProjectHistoryRow = {
  id: string;
  name: string;
  total: number;
  cells: ProjectHistoryCell[];
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
const matrixStorageKey = computed(() => `pft_project_matrix_state_${props.projectId}`);
const THEME_KEY = "pft_theme_mode";

const report = ref<ProjectReport | null>(null);
const loading = ref(true);
const errorMessage = ref<string | null>(null);
const lastSyncedAt = ref<string | null>(null);
const themeMode = ref<"light" | "dark">(localStorage.getItem(THEME_KEY) === "dark" ? "dark" : "light");

const donors = ref<Donor[]>([]);
const plannedDates = ref<MatrixDate[]>([]);
const plannedMembers = ref<string[]>([]);
const memberAliases = ref<Record<string, string>>({});
const warningLabel = ref("Missed");
const redHighlightTheme = ref(true);
const currencyMode = ref<"prefix" | "suffix">("prefix");
const newMatrixDate = ref("");
const newMatrixNote = ref("");
const newMemberName = ref("");
const showMemberDialog = ref(false);
const showDateDialog = ref(false);

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

function formatBoardMoney(cents: number) {
  const amount = (cents / 100).toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
  return currencyMode.value === "prefix" ? `PHP ${amount}` : `${amount} PHP`;
}

function formatDateLabel(value: string) {
  const parsed = new Date(`${value}T00:00:00`);
  if (Number.isNaN(parsed.getTime())) return value;
  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(parsed);
}

function applyTheme() {
  document.documentElement.dataset.theme = themeMode.value;
  localStorage.setItem(THEME_KEY, themeMode.value);
}

const progressPct = computed(() => {
  const target = report.value?.target_amount_cents || 0;
  const donated = report.value?.donations_cents || 0;
  if (target <= 0) return 0;
  return Math.min(100, Math.round((donated / target) * 100));
});

const projectTotal = computed(() => report.value?.donations_cents || 0);

const todayDate = computed(() => new Date().toISOString().slice(0, 10));

const projectHistoryDates = computed<ProjectHistoryDate[]>(() => {
  const totals = new Map<string, number>();
  for (const item of report.value?.donations || []) {
    totals.set(item.donated_at, (totals.get(item.donated_at) || 0) + item.amount_cents);
  }
  for (const planned of plannedDates.value) {
    if (!totals.has(planned.date)) {
      totals.set(planned.date, 0);
    }
  }
  return Array.from(totals.entries())
    .map(([date, total]) => ({ date, total }))
    .sort((a, b) => a.date.localeCompare(b.date));
});

const projectCurrentDayTotal = computed(() => {
  const current = projectHistoryDates.value.find((entry) => entry.date === todayDate.value);
  return current?.total || 0;
});

const projectRows = computed<ProjectHistoryRow[]>(() => {
  const rowMap = new Map<string, ProjectHistoryRow>();
  const seenNames = new Set<string>();

  for (const item of report.value?.donations || []) {
    const name = item.anonymous ? "Anonymous" : item.donor_name?.trim() || "Unnamed contributor";
    const key = item.anonymous ? "anon" : `name:${name.toLowerCase()}`;
    seenNames.add(name.toLowerCase());
    const existing = rowMap.get(key);
    if (!existing) {
      rowMap.set(key, {
        id: key,
        name,
        total: item.amount_cents,
        cells: [{ date: item.donated_at, amount: item.amount_cents }],
      });
      continue;
    }

    existing.total += item.amount_cents;
    const cell = existing.cells.find((entry) => entry.date === item.donated_at);
    if (cell) {
      cell.amount += item.amount_cents;
    } else {
      existing.cells.push({ date: item.donated_at, amount: item.amount_cents });
    }
  }

  for (const plannedName of plannedMembers.value) {
    const key = plannedName.toLowerCase();
    if (seenNames.has(key)) continue;
    rowMap.set(`planned:${key}`, {
      id: `planned:${key}`,
      name: plannedName,
      total: 0,
      cells: [],
    });
  }

  return Array.from(rowMap.values())
    .map((row) => ({
      ...row,
      cells: projectHistoryDates.value.map((date) => ({
        date: date.date,
        amount: row.cells.find((cell) => cell.date === date.date)?.amount ?? 0,
      })),
    }))
    .sort((a, b) => a.name.localeCompare(b.name));
});

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
      plannedMembers: string[];
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
    plannedMembers.value = Array.isArray(parsed.plannedMembers)
      ? parsed.plannedMembers.map((value) => (typeof value === "string" ? value.trim() : "")).filter((value) => value.length > 0)
      : [];
  } catch {
    plannedDates.value = [];
    plannedMembers.value = [];
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
      plannedMembers: plannedMembers.value,
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

  const seen = new Set(rows.map((row) => row.displayName.trim().toLowerCase()));
  for (const plannedName of plannedMembers.value) {
    const normalized = plannedName.trim().toLowerCase();
    if (!normalized || seen.has(normalized)) continue;
    rows.push({
      rowKey: `planned:${normalized}`,
      displayName: plannedName,
      alias: plannedName,
      locked: false,
    });
    seen.add(normalized);
  }

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

const contributionMemberOptions = computed(() =>
  matrixRows.value
    .filter((row) => row.rowKey !== "__anonymous__")
    .map((row) => row.displayName)
    .filter((value, index, array) => array.indexOf(value) === index),
);

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

const activeMembers = computed(() => matrixRows.value.filter((row) => row.rowKey !== "__anonymous__" && matrixRowTotal(row.rowKey) > 0).length);
const paidMembers = computed(() => donors.value.length);

const filledCells = computed(() => {
  let count = 0;
  for (const row of matrixRows.value) {
    for (const date of projectDates.value) {
      if (matrixCellTotal(row.rowKey, date.date) > 0) count += 1;
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

function rebuildProjectTotals(donations: ProjectReport["donations"]) {
  const donationTotal = donations.reduce((sum, entry) => sum + entry.amount_cents, 0);
  const topMap = new Map<string, number>();

  for (const entry of donations) {
    if (entry.anonymous) continue;
    const name = entry.donor_name?.trim();
    if (!name) continue;
    topMap.set(name, (topMap.get(name) || 0) + entry.amount_cents);
  }

  const top_donors = Array.from(topMap.entries())
    .map(([donor_name, total_cents]) => ({ donor_name, total_cents }))
    .sort((a, b) => b.total_cents - a.total_cents || a.donor_name.localeCompare(b.donor_name))
    .slice(0, 5);

  if (!report.value) return;
  report.value = {
    ...report.value,
    donations,
    donations_cents: donationTotal,
    balance_cents: donationTotal - report.value.expenses_cents,
    remaining_to_target_cents: Math.max(0, report.value.target_amount_cents - donationTotal),
    top_donors,
  };
}

function appendProjectDonation(entry: {
  donated_at: string;
  amount_cents: number;
  donor_name?: string | null;
  anonymous: boolean;
  notes?: string | null;
}) {
  if (!report.value) return;
  rebuildProjectTotals([
    ...report.value.donations,
    {
      id: Date.now(),
      donated_at: entry.donated_at,
      amount_cents: entry.amount_cents,
      donor_name: entry.donor_name ?? null,
      anonymous: entry.anonymous,
      notes: entry.notes ?? null,
    },
  ]);
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

function openContributionDialog(memberName = "", date = todayDate.value) {
  showAddDonation.value = true;
  addDate.value = date;
  addAmount.value = "";
  addAnonymous.value = false;
  addDonorName.value = memberName;
  addNotes.value = "";
}

async function addMemberRow() {
  errorMessage.value = null;
  const name = newMemberName.value.trim();
  if (!name) {
    notify("Enter a member name first.");
    return;
  }
  if (!confirm(`Add member "${name}" to this project matrix?`)) return;
  try {
    const existing = donors.value.find((donor) => donor.name.trim().toLowerCase() === name.toLowerCase());
    if (!existing) {
      await donorsCreate(props.sessionToken, { name, notes: null });
    }
    if (!plannedMembers.value.some((member) => member.toLowerCase() === name.toLowerCase())) {
      plannedMembers.value = [...plannedMembers.value, name];
      persistStoredState();
    }
    newMemberName.value = "";
    await load();
    showMemberDialog.value = false;
    notify(`Member "${name}" added.`);
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

function openMemberDialog() {
  newMemberName.value = "";
  showMemberDialog.value = true;
}

function addDateColumn() {
  const date = newMatrixDate.value.trim();
  if (!date) {
    notify("Pick a date first.");
    return;
  }
  if (!confirm(`Add date column ${date}?`)) return;
  if (!plannedDates.value.some((entry) => entry.date === date)) {
    plannedDates.value = [...plannedDates.value, { date, note: newMatrixNote.value.trim() || "Weekly dues" }].sort((a, b) => a.date.localeCompare(b.date));
    persistStoredState();
  } else {
    notify(`Date column ${date} is already in the table.`);
    showDateDialog.value = false;
    return;
  }
  newMatrixDate.value = "";
  newMatrixNote.value = "";
  showDateDialog.value = false;
  notify(`Date column ${date} added.`);
}

function openDateDialog() {
  newMatrixDate.value = new Date().toISOString().slice(0, 10);
  newMatrixNote.value = "";
  showDateDialog.value = true;
}

function resetGrid() {
  if (!confirm("Reset matrix settings for this project?")) return;
  warningLabel.value = "Missed";
  redHighlightTheme.value = true;
  currencyMode.value = "prefix";
  memberAliases.value = {};
  plannedDates.value = [];
  plannedMembers.value = [];
  newMatrixDate.value = "";
  newMatrixNote.value = "";
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
  plannedMembers.value = [];
  newMatrixDate.value = "";
  newMatrixNote.value = "";
  persistStoredState();
  notify("Demo settings restored.");
}

watch([warningLabel, redHighlightTheme, currencyMode, memberAliases, plannedDates, plannedMembers], persistStoredState, { deep: true });

watch(themeMode, () => {
  applyTheme();
  window.dispatchEvent(new CustomEvent("pft:theme-change", { detail: themeMode.value }));
});

async function submitAddDonation() {
  errorMessage.value = null;
  addSubmitting.value = true;
  try {
    const amountCents = centsFromPesos(addAmount.value);
    if (amountCents <= 0) {
      notify("Donation amount must be greater than 0.");
      return;
    }
    const donorName = addDonorName.value.trim();
    if (!addAnonymous.value && !donorName) {
      notify("Select a member name first.");
      return;
    }
    if (!confirm("Save this contribution?")) return;
    let donorId: number | null = null;
    if (!addAnonymous.value && donorName) {
      const existing = donors.value.find((d) => d.name.trim().toLowerCase() === donorName.toLowerCase());
      if (existing) {
        donorId = existing.id;
      } else {
        const created = await donorsCreate(props.sessionToken, { name: donorName, notes: null });
        donorId = created.id;
        donors.value = [
          ...donors.value,
          {
            id: created.id,
            name: donorName,
            notes: null,
            created_at: new Date().toISOString(),
          },
        ];
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
    appendProjectDonation({
      donated_at: addDate.value,
      amount_cents: amountCents,
      donor_name: addAnonymous.value ? "Anonymous" : donorName || null,
      anonymous: addAnonymous.value,
      notes: addNotes.value || null,
    });
    showAddDonation.value = false;
    notify("Contribution saved.");
  } catch (e: any) {
    const message = String(e);
    if (message.includes("donation amount must be > 0")) {
      notify("Donation amount must be greater than 0.");
      return;
    }
    errorMessage.value = message;
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
onMounted(() => {
  applyTheme();
  window.addEventListener("pft:theme-change", (event: Event) => {
    const detail = (event as CustomEvent).detail;
    if (detail === "light" || detail === "dark") {
      themeMode.value = detail;
    }
  });
});
</script>

<template>
  <div class="space-y-6" :class="themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
    <section class="overflow-hidden rounded-[2px] border shadow-sm" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-950 text-slate-100' : 'border-slate-200 bg-white text-slate-900'">
      <div class="border-b-4 border-blue-600 px-6 py-5" :class="themeMode === 'dark' ? 'bg-slate-900' : 'bg-slate-50'">
        <div class="flex flex-col gap-5 xl:flex-row xl:items-end xl:justify-between">
          <div>
            <p class="text-[11px] uppercase tracking-[0.4em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Group Contribution Board</p>
            <h2 class="mt-2 max-w-4xl text-3xl font-semibold tracking-tight md:text-4xl" :class="themeMode === 'dark' ? 'text-white' : 'text-slate-900'">
              {{ report?.project.name || "Loading..." }} contribution register
            </h2>
            <p class="mt-2 max-w-3xl text-sm" :class="themeMode === 'dark' ? 'text-slate-300' : 'text-slate-600'">
              Formal class record book for one project with dated history, live totals, and editable member rows.
            </p>
          </div>

          <div class="grid gap-3 text-right sm:grid-cols-2 xl:min-w-[28rem]">
            <div class="rounded-[2px] border px-4 py-3" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-950' : 'border-slate-200 bg-white'">
              <div class="text-[11px] uppercase tracking-[0.3em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Total Collected</div>
              <div class="mt-2 text-2xl font-semibold" :class="themeMode === 'dark' ? 'text-cyan-400' : 'text-emerald-700'">{{ formatBoardMoney(projectTotal) }}</div>
            </div>
            <div class="rounded-[2px] border px-4 py-3" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-950' : 'border-slate-200 bg-white'">
              <div class="text-[11px] uppercase tracking-[0.3em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Target Completion</div>
              <div class="mt-2 text-2xl font-semibold" :class="themeMode === 'dark' ? 'text-emerald-400' : 'text-emerald-700'">{{ progressPct }}%</div>
            </div>
          </div>
        </div>
      </div>

      <div class="flex flex-col gap-3 border-b px-4 py-3 lg:flex-row lg:items-center lg:justify-between" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900 text-slate-100' : 'border-slate-200 bg-white text-slate-900'">
        <div class="flex flex-wrap items-center gap-2">
          <button class="rounded-[2px] border px-3 py-2 text-sm font-semibold" :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 hover:bg-slate-700' : 'border-slate-300 bg-white hover:bg-slate-50'" @click="emit('back')">
            Back
          </button>
          <button class="rounded-[2px] border px-3 py-2 text-sm font-semibold" :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 hover:bg-slate-700' : 'border-slate-300 bg-white hover:bg-slate-50'" @click="exportProjectDonationsCsv">
            Export CSV
          </button>
          <button class="rounded-[2px] border px-3 py-2 text-sm font-semibold" :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 hover:bg-slate-700' : 'border-slate-300 bg-white hover:bg-slate-50'" @click="exportProjectSummaryPdf">
            Export PDF
          </button>
          <button class="rounded-[2px] border px-3 py-2 text-sm font-semibold" :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 hover:bg-slate-700' : 'border-slate-300 bg-white hover:bg-slate-50'" @click="restoreDemoData">
            Restore Demo Data
          </button>
          <button class="rounded-[2px] border px-3 py-2 text-sm font-semibold" :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 hover:bg-slate-700' : 'border-slate-300 bg-white hover:bg-slate-50'" @click="resetGrid">
            Reset Grid
          </button>
        </div>

        <div class="flex flex-wrap items-center gap-3 text-sm">
          <div class="rounded-[2px] border px-3 py-2" :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100' : 'border-slate-300 bg-slate-50 text-slate-900'">
            Auto-sync: <span class="font-semibold">{{ lastSyncedAt || "Pending" }}</span>
          </div>
          <div class="rounded-[2px] border px-3 py-2" :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100' : 'border-slate-300 bg-slate-50 text-slate-900'">
            Status: <span class="font-semibold">{{ report?.project.status || "-" }}</span>
          </div>
        </div>
      </div>
    </section>

    <div v-if="errorMessage" class="rounded-[2px] border p-4" :class="themeMode === 'dark' ? 'border-rose-500/40 bg-rose-500/10 text-rose-200' : 'border-rose-300 bg-rose-50 text-rose-700'">
      {{ errorMessage }}
    </div>

    <div v-else-if="loading" class="rounded-[2px] border p-4" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900 text-slate-400' : 'border-slate-300 bg-white text-slate-500'">
      Loading...
    </div>

    <div v-else-if="report" class="space-y-6">
      <section class="grid gap-4 px-1 md:grid-cols-2 xl:grid-cols-5">
        <div class="rounded-[2px] border p-4" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900' : 'border-slate-200 bg-white'">
          <div class="flex items-start justify-between gap-3">
            <div>
              <div class="text-[11px] uppercase tracking-[0.3em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Total Collected</div>
              <div class="mt-2 text-2xl font-semibold" :class="themeMode === 'dark' ? 'text-emerald-400' : 'text-emerald-600'">{{ formatBoardMoney(projectTotal) }}</div>
            </div>
            <div class="rounded-[2px] border px-2 py-1 text-xs font-semibold" :class="themeMode === 'dark' ? 'border-emerald-500/20 bg-emerald-500/10 text-emerald-300' : 'border-emerald-200 bg-emerald-50 text-emerald-700'">
              Live
            </div>
          </div>
        </div>
        <div class="rounded-[2px] border p-4" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900' : 'border-slate-200 bg-white'">
          <div class="text-[11px] uppercase tracking-[0.3em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Group Target</div>
          <div class="mt-2 text-2xl font-semibold" :class="themeMode === 'dark' ? 'text-blue-300' : 'text-blue-700'">{{ formatBoardMoney(report.target_amount_cents) }}</div>
          <div class="mt-2 inline-flex rounded-[2px] border px-2 py-1 text-xs font-semibold" :class="themeMode === 'dark' ? 'border-emerald-500/20 bg-emerald-500/10 text-emerald-300' : 'border-emerald-200 bg-emerald-50 text-emerald-700'">
            {{ progressPct }}% complete
          </div>
        </div>
        <div class="rounded-[2px] border p-4" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900' : 'border-slate-200 bg-white'">
          <div class="text-[11px] uppercase tracking-[0.3em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Members Paid</div>
          <div class="mt-2 text-2xl font-semibold" :class="themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">{{ activeMembers }} / {{ paidMembers }}</div>
          <div class="mt-2 text-xs" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Participation fraction</div>
        </div>
        <div class="rounded-[2px] border p-4" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900' : 'border-slate-200 bg-white'">
          <div class="text-[11px] uppercase tracking-[0.3em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Fidelity Index</div>
          <div class="mt-2 text-2xl font-semibold" :class="themeMode === 'dark' ? 'text-indigo-300' : 'text-indigo-700'">{{ fidelityPct }}%</div>
          <div class="mt-2 text-xs" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Filled cells vs. board size</div>
        </div>
        <div class="rounded-[2px] border p-4" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900' : 'border-slate-200 bg-white'">
          <div class="text-[11px] uppercase tracking-[0.3em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Top Supporter</div>
          <div class="mt-2 text-2xl font-semibold" :class="themeMode === 'dark' ? 'text-amber-300' : 'text-amber-700'">{{ topSupporter.name }}</div>
          <div class="mt-2 inline-flex items-center gap-2 rounded-[2px] border px-2 py-1 text-xs font-semibold" :class="themeMode === 'dark' ? 'border-amber-500/20 bg-amber-500/10 text-amber-200' : 'border-amber-200 bg-amber-50 text-amber-800'">
            <svg class="h-3.5 w-3.5" :class="themeMode === 'dark' ? 'text-amber-300' : 'text-amber-700'" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
              <path d="M3 7l5 4 4-6 4 6 5-4-2 11H5L3 7zm3 11h12v2H6z" />
            </svg>
            <span>{{ formatBoardMoney(topSupporter.total) }}</span>
          </div>
        </div>
      </section>

      <ContributionHistoryTable
        :rows="projectRows"
        :dates="projectHistoryDates"
        :overall-total="projectTotal"
        :current-day-total="projectCurrentDayTotal"
        :current-day-label="todayDate"
        :theme-mode="themeMode"
      :format-money="formatBoardMoney"
      :format-date="formatDateLabel"
      @add-contribution="openContributionDialog"
      @add-member="openMemberDialog"
      @add-date="openDateDialog"
    />

      <Dialog
        v-model:open="showDateDialog"
        :theme-mode="themeMode"
        title="Add Date Column"
        description="Add a new date slot to the matrix. Existing project entries stay in place."
      >
        <div class="grid gap-4">
          <CalendarPicker v-model="newMatrixDate" :theme-mode="themeMode" />

          <div>
            <label class="mb-1 block text-xs uppercase tracking-[0.2em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">
              Note
            </label>
            <input
              v-model="newMatrixNote"
              placeholder="Column note"
              class="w-full rounded-[2px] border px-3 py-2 text-sm outline-none"
              :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-900 text-slate-100 placeholder:text-slate-500' : 'border-slate-300 bg-white text-slate-900 placeholder:text-slate-400'"
            />
          </div>

          <div class="flex items-center justify-end gap-2 pt-2">
            <button
              type="button"
              class="rounded-[2px] border px-3 py-2 text-sm font-semibold"
              :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100 hover:bg-slate-700' : 'border-slate-300 bg-white text-slate-900 hover:bg-slate-50'"
              @click="showDateDialog = false"
            >
              Cancel
            </button>
            <button
              type="button"
              class="rounded-[2px] border px-3 py-2 text-sm font-semibold"
              :class="themeMode === 'dark' ? 'border-blue-500 bg-blue-500 text-white hover:bg-blue-400' : 'border-blue-600 bg-blue-600 text-white hover:bg-blue-500'"
              @click="addDateColumn"
            >
              Save Date
            </button>
          </div>
        </div>
      </Dialog>

      <Dialog
        v-model:open="showMemberDialog"
        :theme-mode="themeMode"
        title="Add Member"
        description="Add a new contributor row to this project matrix."
      >
        <div class="grid gap-4">
          <div>
            <label class="mb-1 block text-xs uppercase tracking-[0.2em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">
              Member name
            </label>
            <input
              v-model="newMemberName"
              placeholder="New member name"
              class="w-full rounded-[2px] border px-3 py-2 text-sm outline-none"
              :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-900 text-slate-100 placeholder:text-slate-500' : 'border-slate-300 bg-white text-slate-900 placeholder:text-slate-400'"
            />
          </div>

          <div class="flex items-center justify-end gap-2 pt-2">
            <button
              type="button"
              class="rounded-[2px] border px-3 py-2 text-sm font-semibold"
              :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100 hover:bg-slate-700' : 'border-slate-300 bg-white text-slate-900 hover:bg-slate-50'"
              @click="showMemberDialog = false"
            >
              Cancel
            </button>
            <button
              type="button"
              class="rounded-[2px] border px-3 py-2 text-sm font-semibold"
              :class="themeMode === 'dark' ? 'border-blue-500 bg-blue-500 text-white hover:bg-blue-400' : 'border-blue-600 bg-blue-600 text-white hover:bg-blue-500'"
              @click="addMemberRow"
            >
              Save Member
            </button>
          </div>
        </div>
      </Dialog>

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
              <div class="text-sm text-slate-400">Pick the date and member to place the amount into the project matrix.</div>
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
              <div class="text-xs text-slate-400 mb-1">Member</div>
              <input
                v-model="addDonorName"
                :disabled="addAnonymous"
                list="project-member-options"
                placeholder="Select or type a name"
                class="w-full rounded-xl border border-slate-700 bg-slate-900/40 px-3 py-2 disabled:opacity-60"
              />
              <datalist id="project-member-options">
                <option v-for="member in contributionMemberOptions" :key="member" :value="member" />
              </datalist>
            </div>
            <div>
              <div class="text-xs text-slate-400 mb-1">Anonymous</div>
              <label class="flex items-center gap-2 rounded-xl border border-slate-700 bg-slate-900/40 px-3 py-2">
                <input v-model="addAnonymous" type="checkbox" class="accent-indigo-500" />
                <span class="text-sm text-slate-200">Yes</span>
              </label>
            </div>
            <div>
              <div class="text-xs text-slate-400 mb-1">Amount (PHP)</div>
              <input v-model="addAmount" inputmode="decimal" placeholder="0.00" class="w-full rounded-xl border border-slate-700 bg-slate-900/40 px-3 py-2" />
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
