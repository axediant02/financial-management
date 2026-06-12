<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import {
  donationsCreate,
  donationsDelete,
  donationsList,
  donorsCreate,
  donorsList,
  exportCsv,
  projectsList,
} from "../../lib/api";
import { notify } from "../../lib/feedback";
import type { Donation, Donor, Project } from "../../lib/types";
import ContributionHistoryTable from "./ContributionHistoryTable.vue";

type PlannedDate = {
  date: string;
  note: string;
};

type CellEditor = {
  rowKey: string;
  date: string;
};

const props = defineProps<{ sessionToken: string }>();

const STORAGE_KEY = "pft_contribution_board_state";
const THEME_KEY = "pft_theme_mode";
const defaultWarningLabel = "Missed";
const defaultCurrencyMode: "prefix" | "suffix" = "prefix";
const defaultThemeMode: "light" | "dark" = "light";

const loading = ref(true);
const syncing = ref(false);
const errorMessage = ref<string | null>(null);
const lastSyncedAt = ref<string | null>(null);
const hydrated = ref(false);

const items = ref<Donation[]>([]);
const donors = ref<Donor[]>([]);
const projects = ref<Project[]>([]);

const scopeProjectId = ref<string>("");
const warningLabel = ref(defaultWarningLabel);
const redHighlightTheme = ref(true);
const currencyMode = ref<"prefix" | "suffix">(defaultCurrencyMode);
const themeMode = ref<"light" | "dark">(defaultThemeMode);
const memberAliases = ref<Record<string, string>>({});
const plannedDates = ref<PlannedDate[]>([]);

const newMemberName = ref("");
const newDateValue = ref("");
const newDateNote = ref("");

const cellEditor = ref<CellEditor | null>(null);
const cellAmount = ref("");
const cellNotes = ref("");
const cellProjectId = ref<string>("");

const donorsById = computed(() => new Map(donors.value.map((d) => [d.id, d])));
const projectsById = computed(() => new Map(projects.value.map((p) => [p.id, p])));

function readStoredState() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    const themeRaw = localStorage.getItem(THEME_KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw) as Partial<{
      scopeProjectId: string;
      warningLabel: string;
      redHighlightTheme: boolean;
      currencyMode: "prefix" | "suffix";
      themeMode: "light" | "dark";
      memberAliases: Record<string, string>;
      plannedDates: PlannedDate[];
    }>;
    scopeProjectId.value = typeof parsed.scopeProjectId === "string" ? parsed.scopeProjectId : "";
    warningLabel.value = typeof parsed.warningLabel === "string" && parsed.warningLabel.trim()
      ? parsed.warningLabel
      : defaultWarningLabel;
    redHighlightTheme.value = typeof parsed.redHighlightTheme === "boolean" ? parsed.redHighlightTheme : true;
    currencyMode.value = parsed.currencyMode === "suffix" ? "suffix" : defaultCurrencyMode;
    themeMode.value = themeRaw === "dark" || parsed.themeMode === "dark" ? "dark" : defaultThemeMode;
    memberAliases.value = parsed.memberAliases && typeof parsed.memberAliases === "object" ? parsed.memberAliases : {};
    plannedDates.value = Array.isArray(parsed.plannedDates) ? parsed.plannedDates : [];
  } catch {
    // Ignore malformed local state.
  }
}

function persistStoredState() {
  localStorage.setItem(
    STORAGE_KEY,
    JSON.stringify({
      scopeProjectId: scopeProjectId.value,
      warningLabel: warningLabel.value,
      redHighlightTheme: redHighlightTheme.value,
      currencyMode: currencyMode.value,
      themeMode: themeMode.value,
      memberAliases: memberAliases.value,
      plannedDates: plannedDates.value,
    }),
  );
  localStorage.setItem(THEME_KEY, themeMode.value);
}

function applyTheme() {
  document.documentElement.dataset.theme = themeMode.value;
}

function toggleTheme() {
  themeMode.value = themeMode.value === "light" ? "dark" : "light";
}

const boardShellClass = computed(() =>
  themeMode.value === "dark"
    ? "space-y-6 text-slate-100"
    : "space-y-6 text-slate-900",
);

function formatDateLabel(date: string) {
  const parsed = new Date(`${date}T00:00:00`);
  if (Number.isNaN(parsed.getTime())) return date;
  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(parsed);
}

function formatBoardMoney(cents: number) {
  const amount = (cents / 100).toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
  return currencyMode.value === "prefix" ? `PHP ${amount}` : `${amount} PHP`;
}

function formatCompactMoney(cents: number) {
  return formatBoardMoney(cents);
}

function uniqueDates() {
  const map = new Map<string, PlannedDate>();

  for (const item of items.value) {
    if (!map.has(item.donated_at)) {
      map.set(item.donated_at, {
        date: item.donated_at,
        note: item.notes?.trim() || "Recorded contribution",
      });
    } else {
      const current = map.get(item.donated_at)!;
      if (!current.note || current.note === "Recorded contribution") {
        current.note = item.notes?.trim() || current.note;
      }
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

const matrixDates = computed(() => uniqueDates());

const rowDefs = computed(() => {
  const memberRows = donors.value
    .slice()
    .sort((a, b) => a.name.localeCompare(b.name))
    .map((donor) => ({
      rowKey: `donor:${donor.id}`,
      donorId: donor.id,
      defaultName: donor.name,
      displayName: memberAliases.value[`donor:${donor.id}`]?.trim() || donor.name,
      locked: false,
    }));

  return [
    ...memberRows,
    {
      rowKey: "anon",
      donorId: null as number | null,
      defaultName: "Anonymous",
      displayName: "Anonymous",
      locked: true,
    },
  ];
});

const cellLookup = computed(() => {
  const lookup = new Map<string, Map<string, Donation[]>>();
  for (const row of rowDefs.value) {
    lookup.set(row.rowKey, new Map());
  }

  for (const item of items.value) {
    const rowKey = item.anonymous || item.donor_id == null ? "anon" : `donor:${item.donor_id}`;
    const rowMap = lookup.get(rowKey);
    if (!rowMap) continue;
    const bucket = rowMap.get(item.donated_at) || [];
    bucket.push(item);
    rowMap.set(item.donated_at, bucket);
  }

  return lookup;
});

function cellItems(rowKey: string, date: string) {
  return cellLookup.value.get(rowKey)?.get(date) || [];
}

function cellTotal(rowKey: string, date: string) {
  return cellItems(rowKey, date).reduce((sum, item) => sum + item.amount_cents, 0);
}

function rowTotal(rowKey: string) {
  return matrixDates.value.reduce((sum, entry) => sum + cellTotal(rowKey, entry.date), 0);
}

const totalCollected = computed(() => items.value.reduce((sum, item) => sum + item.amount_cents, 0));
const targetTotal = computed(() =>
  projects.value
    .filter((project) => !scopeProjectId.value || String(project.id) === scopeProjectId.value)
    .reduce((sum, project) => sum + project.target_amount_cents, 0),
);

const dateTotals = computed(() =>
  matrixDates.value.map((entry) => ({
    date: entry.date,
    total: items.value
      .filter((item) => item.donated_at === entry.date)
      .reduce((sum, item) => sum + item.amount_cents, 0),
  })),
);

const completionPct = computed(() => {
  if (targetTotal.value <= 0) return 0;
  return Math.min(100, Math.round((totalCollected.value / targetTotal.value) * 100));
});

const activeMembers = computed(() =>
  donors.value.filter((donor) => rowTotal(`donor:${donor.id}`) > 0).length,
);

const paidMembers = computed(() => donors.value.length);

const filledCells = computed(() => {
  let count = 0;
  for (const row of rowDefs.value) {
    for (const entry of matrixDates.value) {
      if (cellTotal(row.rowKey, entry.date) > 0) count += 1;
    }
  }
  return count;
});

const fidelityPct = computed(() => {
  const totalCells = rowDefs.value.length * matrixDates.value.length;
  if (totalCells <= 0) return 0;
  return Math.round((filledCells.value / totalCells) * 100);
});

const topSupporter = computed(() => {
  let bestName = "No records";
  let bestTotal = 0;
  for (const donor of donors.value) {
    const total = rowTotal(`donor:${donor.id}`);
    if (total > bestTotal) {
      bestTotal = total;
      bestName = memberAliases.value[`donor:${donor.id}`]?.trim() || donor.name;
    }
  }
  return { name: bestName, total: bestTotal };
});

const scopeProject = computed(() =>
  scopeProjectId.value ? projectsById.value.get(Number(scopeProjectId.value)) || null : null,
);

const quickScopeLabel = computed(() => {
  if (scopeProject.value) return scopeProject.value.name;
  return "All projects";
});

function getLocalDateString(date = new Date()) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

const todayDate = computed(() => getLocalDateString());

const contributionHistoryDates = computed(() =>
  dateTotals.value.map((entry) => ({
    date: entry.date,
    total: entry.total,
  })),
);

const contributionHistoryRows = computed(() =>
  rowDefs.value.map((row) => ({
    id: row.rowKey,
    name: row.displayName,
    total: rowTotal(row.rowKey),
    cells: matrixDates.value.map((entry) => ({
      date: entry.date,
      amount: cellTotal(row.rowKey, entry.date),
    })),
  })),
);

const currentDayTotal = computed(() => {
  const todayEntry = dateTotals.value.find((entry) => entry.date === todayDate.value);
  return todayEntry?.total || 0;
});

async function load() {
  loading.value = true;
  syncing.value = true;
  errorMessage.value = null;
  try {
    donors.value = await donorsList(props.sessionToken);
    projects.value = await projectsList(props.sessionToken);
    items.value = await donationsList(props.sessionToken, {
      from: null,
      to: null,
      project_id: scopeProjectId.value ? Number(scopeProjectId.value) : null,
    });
    lastSyncedAt.value = new Date().toLocaleString();
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    loading.value = false;
    syncing.value = false;
  }
}

async function exportCurrentCsv() {
  const dest = await save({
    defaultPath: "contribution-board.csv",
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!dest) return;
  await exportCsv(props.sessionToken, {
    kind: "donations",
    filter: {
      from: null,
      to: null,
      project_id: scopeProjectId.value ? Number(scopeProjectId.value) : null,
    },
    dest_path: dest,
  });
  notify("Contribution CSV exported.");
}

async function addMember() {
  errorMessage.value = null;
  const name = newMemberName.value.trim();
  if (!name) return;
  if (!confirm(`Add member "${name}"?`)) return;
  try {
    await donorsCreate(props.sessionToken, { name, notes: null });
    newMemberName.value = "";
    await load();
    notify(`Member "${name}" added.`);
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

function addPlannedDate() {
  const date = newDateValue.value.trim();
  if (!date) return;
  const note = newDateNote.value.trim();
  if (!confirm(`Add planned date ${date}${note ? ` with note "${note}"` : ""}?`)) return;
  const next = plannedDates.value.filter((item) => item.date !== date);
  next.push({ date, note: note || "Planned contribution day" });
  plannedDates.value = next.sort((a, b) => a.date.localeCompare(b.date));
  newDateValue.value = "";
  newDateNote.value = "";
  notify(`Planned date ${date} added.`);
}

function openCell(rowKey: string, date: string) {
  cellEditor.value = { rowKey, date };
  cellAmount.value = "";
  cellNotes.value = "";
  cellProjectId.value = scopeProjectId.value;
}

function closeCell() {
  cellEditor.value = null;
  cellAmount.value = "";
  cellNotes.value = "";
}

async function saveCell() {
  if (!cellEditor.value) return;
  errorMessage.value = null;
  const row = cellEditor.value.rowKey;
  const amount = Number(String(cellAmount.value).replace(/,/g, ""));
  if (!Number.isFinite(amount) || amount <= 0) {
    errorMessage.value = "Enter a valid amount greater than zero.";
    return;
  }
  if (!confirm("Save this contribution record?")) return;
  try {
    const donorId = row.startsWith("donor:") ? Number(row.split(":")[1]) : null;
    await donationsCreate(props.sessionToken, {
      donated_at: cellEditor.value.date,
      amount_cents: Math.round(amount * 100),
      donor_id: donorId,
      anonymous: row === "anon",
      notes: cellNotes.value.trim() || null,
      project_id: cellProjectId.value ? Number(cellProjectId.value) : null,
    });
    closeCell();
    await load();
    notify("Contribution saved.");
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

async function removeEntry(id: number) {
  if (!confirm("Delete this contribution record?")) return;
  try {
    await donationsDelete(props.sessionToken, id);
    await load();
    notify("Contribution deleted.");
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

function setAlias(rowKey: string, value: string) {
  memberAliases.value = {
    ...memberAliases.value,
    [rowKey]: value,
  };
}

function resetGrid() {
  if (!confirm("Reset the grid settings? This clears cached board preferences only.")) return;
  memberAliases.value = {};
  plannedDates.value = [];
  cellEditor.value = null;
  cellAmount.value = "";
  cellNotes.value = "";
  newMemberName.value = "";
  newDateValue.value = "";
  newDateNote.value = "";
  errorMessage.value = null;
  persistStoredState();
  notify("Grid settings reset.");
}

function restoreDemoData() {
  if (!confirm("Restore demo data and reset board settings?")) return;
  scopeProjectId.value = "";
  warningLabel.value = defaultWarningLabel;
  redHighlightTheme.value = true;
  currencyMode.value = defaultCurrencyMode;
  memberAliases.value = {};
  plannedDates.value = [];
  cellEditor.value = null;
  cellAmount.value = "";
  cellNotes.value = "";
  newMemberName.value = "";
  newDateValue.value = "";
  newDateNote.value = "";
  errorMessage.value = null;
  persistStoredState();
  void load();
  notify("Demo board restored.");
}

watch(
  [scopeProjectId, warningLabel, redHighlightTheme, currencyMode, memberAliases, plannedDates],
  persistStoredState,
  { deep: true },
);

watch(themeMode, () => {
  applyTheme();
  persistStoredState();
  window.dispatchEvent(new CustomEvent("pft:theme-change", { detail: themeMode.value }));
});

watch(scopeProjectId, () => {
  if (hydrated.value) {
    void load();
  }
});

onMounted(() => {
  readStoredState();
  applyTheme();
  window.addEventListener("pft:theme-change", (event: Event) => {
    const detail = (event as CustomEvent).detail;
    if (detail === "light" || detail === "dark") {
      themeMode.value = detail;
    }
  });
  hydrated.value = true;
  void load();
});
</script>

<template>
  <div :class="boardShellClass">
    <section class="overflow-hidden rounded-[2px] border shadow-sm" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-950 text-slate-100' : 'border-slate-300 bg-slate-950 text-slate-100'">
      <div class="border-b-4 border-blue-600 bg-slate-900 px-6 py-5">
        <div class="flex flex-col gap-5 xl:flex-row xl:items-end xl:justify-between">
          <div>
            <p class="text-[11px] uppercase tracking-[0.4em] text-slate-400">Group Contribution Board</p>
            <h2 class="mt-2 max-w-4xl text-3xl font-semibold tracking-tight text-white md:text-4xl">
              {{ quickScopeLabel }} contribution register
            </h2>
            <p class="mt-2 max-w-3xl text-sm text-slate-300">
              Formal record-book layout with live totals, inline cell entry, and cached board settings.
            </p>
          </div>

          <div class="grid gap-3 text-right sm:grid-cols-2 xl:min-w-[28rem]">
            <div class="rounded-[2px] border px-4 py-3" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-950' : 'border-slate-300 bg-slate-950'">
              <div class="text-[11px] uppercase tracking-[0.3em] text-slate-400">Total Collected</div>
              <div class="mt-2 text-2xl font-semibold text-cyan-400">{{ formatCompactMoney(totalCollected) }}</div>
            </div>
            <div class="rounded-[2px] border px-4 py-3" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-950' : 'border-slate-300 bg-slate-950'">
              <div class="text-[11px] uppercase tracking-[0.3em] text-slate-400">Target Completion</div>
              <div class="mt-2 text-2xl font-semibold text-emerald-400">{{ completionPct }}%</div>
            </div>
          </div>
        </div>
      </div>

      <div class="flex flex-col gap-3 border-b px-4 py-3 lg:flex-row lg:items-center lg:justify-between" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900 text-slate-100' : 'border-slate-300 bg-white text-slate-900'">
        <div class="flex flex-wrap items-center gap-2">
          <button
            class="rounded-[2px] border px-3 py-2 text-sm font-semibold" :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100 hover:bg-slate-700' : 'border-slate-300 bg-white text-slate-900 hover:bg-slate-50'"
            @click="exportCurrentCsv"
          >
            Export CSV
          </button>
          <button
            class="rounded-[2px] border px-3 py-2 text-sm font-semibold" :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100 hover:bg-slate-700' : 'border-slate-300 bg-white text-slate-900 hover:bg-slate-50'"
            @click="restoreDemoData"
          >
            Restore Demo Data
          </button>
          <button
            class="rounded-[2px] border px-3 py-2 text-sm font-semibold" :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100 hover:bg-slate-700' : 'border-slate-300 bg-white text-slate-900 hover:bg-slate-50'"
            @click="resetGrid"
          >
            Reset Grid
          </button>
        </div>

        <div class="flex flex-wrap items-center gap-3 text-sm">
          <div class="rounded-[2px] border px-3 py-2" :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100' : 'border-slate-300 bg-slate-50 text-slate-900'">
            Auto-sync: <span class="font-semibold">{{ syncing ? "Syncing..." : (lastSyncedAt || "Pending") }}</span>
          </div>
          <div class="rounded-[2px] border px-3 py-2" :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100' : 'border-slate-300 bg-slate-50 text-slate-900'">
            Scope: <span class="font-semibold">{{ quickScopeLabel }}</span>
          </div>
          <select
            v-model="scopeProjectId"
            class="rounded-[2px] border px-3 py-2 text-sm font-medium" :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100' : 'border-slate-300 bg-white text-slate-900'"
          >
            <option value="">All projects</option>
            <option v-for="p in projects" :key="p.id" :value="String(p.id)">{{ p.name }}</option>
          </select>
        </div>
      </div>

      <div class="grid gap-4 px-4 py-4 md:grid-cols-2 xl:grid-cols-5" :class="themeMode === 'dark' ? 'bg-slate-950' : 'bg-slate-100'">
        <div class="rounded-[2px] border p-4" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900' : 'border-slate-300 bg-white'">
          <div class="flex items-start justify-between gap-3">
            <div>
              <div class="text-[11px] uppercase tracking-[0.3em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Total Collected</div>
              <div class="mt-2 text-2xl font-semibold text-emerald-600">{{ formatCompactMoney(totalCollected) }}</div>
            </div>
            <div class="rounded-[2px] border border-emerald-200 bg-emerald-50 px-2 py-1 text-xs font-semibold text-emerald-700">
              Live
            </div>
          </div>
        </div>

        <div class="rounded-[2px] border p-4" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900' : 'border-slate-300 bg-white'">
          <div class="text-[11px] uppercase tracking-[0.3em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Group Target</div>
          <div class="mt-2 text-2xl font-semibold text-blue-700">{{ formatCompactMoney(targetTotal) }}</div>
          <div class="mt-2 inline-flex rounded-[2px] border border-emerald-200 bg-emerald-50 px-2 py-1 text-xs font-semibold text-emerald-700">
            {{ completionPct }}% complete
          </div>
        </div>

        <div class="rounded-[2px] border p-4" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900' : 'border-slate-300 bg-white'">
          <div class="text-[11px] uppercase tracking-[0.3em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Members Paid</div>
          <div class="mt-2 text-2xl font-semibold" :class="themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">{{ activeMembers }} / {{ paidMembers }}</div>
          <div class="mt-2 text-xs" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Participation fraction</div>
        </div>

        <div class="rounded-[2px] border p-4" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900' : 'border-slate-300 bg-white'">
          <div class="text-[11px] uppercase tracking-[0.3em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Fidelity Index</div>
          <div class="mt-2 text-2xl font-semibold text-indigo-700">{{ fidelityPct }}%</div>
          <div class="mt-2 text-xs" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Filled cells vs. board size</div>
        </div>

        <div class="rounded-[2px] border p-4" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900' : 'border-slate-300 bg-white'">
          <div class="text-[11px] uppercase tracking-[0.3em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Top Supporter</div>
          <div class="mt-2 text-2xl font-semibold text-amber-700">{{ topSupporter.name }}</div>
          <div class="mt-2 inline-flex items-center gap-2 rounded-[2px] border border-amber-200 bg-amber-50 px-2 py-1 text-xs font-semibold text-amber-800">
            <svg class="h-3.5 w-3.5 text-amber-700" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
              <path d="M3 7l5 4 4-6 4 6 5-4-2 11H5L3 7zm3 11h12v2H6z" />
            </svg>
            <span>{{ formatCompactMoney(topSupporter.total) }}</span>
          </div>
        </div>
      </div>
    </section>

    <div v-if="errorMessage" class="rounded-[2px] border border-rose-300 bg-rose-50 p-4 text-rose-700">
      {{ errorMessage }}
    </div>

    <ContributionHistoryTable
      :rows="contributionHistoryRows"
      :dates="contributionHistoryDates"
      :overall-total="totalCollected"
      :current-day-total="currentDayTotal"
      :current-day-label="todayDate"
      :theme-mode="themeMode"
      :format-money="formatCompactMoney"
      :format-date="formatDateLabel"
    />

    <section class="rounded-[2px] border px-4 py-4 shadow-sm" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900' : 'border-slate-300 bg-white'">
      <div class="flex flex-col gap-2 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <div class="text-lg font-semibold" :class="themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">Class-Record Contribution Matrix</div>
          <div class="text-sm" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">
            The detailed matrix now lives inside each project so the records shown are only for that project.
          </div>
        </div>
        <div class="text-sm" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">
          Open a project from the Projects tab to review its record book.
        </div>
      </div>
    </section>

    <section class="grid gap-4 lg:grid-cols-2">
      <div class="rounded-[2px] border p-4 shadow-sm" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900' : 'border-slate-300 bg-white'">
        <div class="text-lg font-semibold" :class="themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">Membership Appender</div>
        <div class="mt-1 text-sm" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Insert a new member row into the board.</div>
        <div class="mt-4 flex gap-2">
          <input
            v-model="newMemberName"
            placeholder="New member name"
            class="min-w-0 flex-1 rounded-[2px] border px-3 py-2 text-sm outline-none focus:border-blue-500"
            :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100' : 'border-slate-300 bg-white text-slate-900'"
          />
          <button class="rounded-[2px] border border-blue-600 bg-blue-600 px-4 py-2 text-sm font-semibold text-white" @click="addMember">
            Add Member
          </button>
        </div>
      </div>

      <div class="rounded-[2px] border p-4 shadow-sm" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900' : 'border-slate-300 bg-white'">
        <div class="text-lg font-semibold" :class="themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">Date Column Scheduler</div>
        <div class="mt-1 text-sm" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Preload a future date column and attach a note.</div>
        <div class="mt-4 grid gap-2 md:grid-cols-3">
          <input
            v-model="newDateValue"
            type="date"
            class="rounded-[2px] border px-3 py-2 text-sm outline-none focus:border-blue-500"
            :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100' : 'border-slate-300 bg-white text-slate-900'"
          />
          <input
            v-model="newDateNote"
            placeholder="Column note"
            class="rounded-[2px] border px-3 py-2 text-sm outline-none focus:border-blue-500 md:col-span-1"
            :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100' : 'border-slate-300 bg-white text-slate-900'"
          />
          <button class="rounded-[2px] border border-slate-900 bg-slate-900 px-4 py-2 text-sm font-semibold text-white" @click="addPlannedDate">
            Add Date
          </button>
        </div>
      </div>
    </section>

    <section class="rounded-[2px] border px-4 py-4 text-slate-100 shadow-sm" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900' : 'border-slate-300 bg-slate-900'">
      <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <div class="text-sm font-semibold uppercase tracking-[0.2em] text-slate-300">Customizer Footer</div>
          <div class="mt-1 text-sm text-slate-400">
            Cached locally in the browser so the board keeps its display preferences.
          </div>
        </div>

        <div class="flex flex-wrap items-center gap-3 text-sm">
          <label class="flex items-center gap-2 rounded-[2px] border px-3 py-2" :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800' : 'border-slate-300 bg-slate-100'">
            <span class="text-slate-300">Warning label</span>
            <input
              v-model="warningLabel"
              class="w-32 rounded-[2px] border px-2 py-1 text-sm outline-none"
              :class="themeMode === 'dark' ? 'border-slate-500 bg-slate-900 text-white' : 'border-slate-300 bg-white text-slate-900'"
            />
          </label>

          <label class="flex items-center gap-2 rounded-[2px] border px-3 py-2" :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800' : 'border-slate-300 bg-slate-100'">
            <input v-model="redHighlightTheme" type="checkbox" class="accent-red-500" />
            <span class="text-slate-300">Red highlight theme</span>
          </label>

          <label class="flex items-center gap-2 rounded-[2px] border px-3 py-2" :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800' : 'border-slate-300 bg-slate-100'">
            <span class="text-slate-300">Currency</span>
            <select v-model="currencyMode" class="rounded-[2px] border px-2 py-1 outline-none" :class="themeMode === 'dark' ? 'border-slate-500 bg-slate-900 text-white' : 'border-slate-300 bg-white text-slate-900'">
              <option value="prefix">Prefix</option>
              <option value="suffix">Suffix</option>
            </select>
          </label>

          <button
            type="button"
            class="rounded-[2px] border px-3 py-2 font-semibold"
            :class="themeMode === 'dark' ? 'border-slate-600 bg-white text-slate-900' : 'border-slate-300 bg-slate-900 text-white'"
            @click="toggleTheme"
          >
            {{ themeMode === 'light' ? 'Dark mode' : 'Light mode' }}
          </button>
        </div>
      </div>
    </section>

    <section class="rounded-[2px] border p-4 shadow-sm" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900' : 'border-slate-300 bg-white'">
      <div class="flex items-center justify-between gap-3">
        <div>
          <div class="text-lg font-semibold" :class="themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">Recent Contribution Log</div>
          <div class="text-sm" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Latest 12 records, kept for quick review and deletion.</div>
        </div>
        <div class="text-sm" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">
          Last sync: <span class="font-semibold" :class="themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">{{ lastSyncedAt || "Pending" }}</span>
        </div>
      </div>

      <div class="mt-4 overflow-hidden rounded-[2px] border" :class="themeMode === 'dark' ? 'border-slate-700' : 'border-slate-300'">
        <table class="w-full text-sm">
          <thead :class="themeMode === 'dark' ? 'bg-slate-800 text-slate-200' : 'bg-slate-100 text-slate-700'">
            <tr>
              <th class="px-3 py-2 text-left uppercase tracking-[0.2em]">Date</th>
              <th class="px-3 py-2 text-left uppercase tracking-[0.2em]">Member</th>
              <th class="px-3 py-2 text-left uppercase tracking-[0.2em]">Project</th>
              <th class="px-3 py-2 text-right uppercase tracking-[0.2em]">Amount</th>
              <th class="px-3 py-2 text-right uppercase tracking-[0.2em]">Action</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in items.slice().sort((a, b) => b.donated_at.localeCompare(a.donated_at)).slice(0, 12)" :key="item.id" class="border-t" :class="themeMode === 'dark' ? 'border-slate-700' : 'border-slate-200'">
              <td class="px-3 py-2" :class="themeMode === 'dark' ? 'text-slate-300' : 'text-slate-700'">{{ item.donated_at }}</td>
              <td class="px-3 py-2" :class="themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
                {{ item.anonymous ? "Anonymous" : (item.donor_id ? (memberAliases[`donor:${item.donor_id}`] || donorsById.get(item.donor_id)?.name || `#${item.donor_id}`) : "-") }}
              </td>
              <td class="px-3 py-2" :class="themeMode === 'dark' ? 'text-slate-300' : 'text-slate-700'">
                {{ item.project_id ? projectsById.get(item.project_id)?.name || `#${item.project_id}` : "-" }}
              </td>
              <td class="px-3 py-2 text-right font-semibold" :class="themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">{{ formatCompactMoney(item.amount_cents) }}</td>
              <td class="px-3 py-2 text-right">
                <button class="rounded-[2px] border border-rose-300 bg-rose-50 px-3 py-1 text-xs font-semibold text-rose-700" @click="removeEntry(item.id)">
                  Delete
                </button>
              </td>
            </tr>
            <tr v-if="!loading && items.length === 0">
              <td colspan="5" class="px-3 py-4" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">No contributions found.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </div>
</template>
