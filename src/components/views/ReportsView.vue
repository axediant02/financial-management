<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { exportCsv, exportPdf, ledgerSummary, projectBalances, projectsList } from "../../lib/api";
import { formatPHPFromCents } from "../../lib/money";
import type { LedgerSummary, Project, ProjectBalanceRow } from "../../lib/types";

const props = defineProps<{ sessionToken: string }>();

const projects = ref<Project[]>([]);
const filterFrom = ref<string>("");
const filterTo = ref<string>("");
const filterProjectId = ref<string>("");

const summary = ref<LedgerSummary | null>(null);
const balances = ref<ProjectBalanceRow[]>([]);
const loading = ref(false);
const errorMessage = ref<string | null>(null);

const activeFilter = computed(() => ({
  from: filterFrom.value || null,
  to: filterTo.value || null,
  project_id: filterProjectId.value ? Number(filterProjectId.value) : null,
}));

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    projects.value = await projectsList(props.sessionToken);
    summary.value = await ledgerSummary(props.sessionToken, activeFilter.value);
    balances.value = await projectBalances(props.sessionToken, activeFilter.value);
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function exportDonationsCsv() {
  const dest = await save({
    defaultPath: `contributions-${filterFrom.value || "all"}-${filterTo.value || "all"}.csv`,
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!dest) return;
  await exportCsv(props.sessionToken, { kind: "donations", filter: activeFilter.value, dest_path: dest });
}

async function exportExpensesCsv() {
  const dest = await save({
    defaultPath: `expenses-${filterFrom.value || "all"}-${filterTo.value || "all"}.csv`,
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!dest) return;
  await exportCsv(props.sessionToken, { kind: "expenses", filter: activeFilter.value, dest_path: dest });
}

async function exportProjectsCsv() {
  const dest = await save({
    defaultPath: `projects.csv`,
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!dest) return;
  await exportCsv(props.sessionToken, { kind: "projects", filter: activeFilter.value, dest_path: dest });
}

async function exportSummaryPdf() {
  const dest = await save({
    defaultPath: `summary-${filterFrom.value || "all"}-${filterTo.value || "all"}.pdf`,
    filters: [{ name: "PDF", extensions: ["pdf"] }],
  });
  if (!dest) return;
  await exportPdf(props.sessionToken, { title: "Project Funds Tracker — Summary", filter: activeFilter.value, dest_path: dest });
}

onMounted(load);
</script>

<template>
  <div class="space-y-6">
    <div v-if="errorMessage" class="rounded-xl border border-rose-500/40 bg-rose-500/10 p-4 text-rose-200">
      {{ errorMessage }}
    </div>

    <div class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
      <div class="flex flex-col md:flex-row md:items-end md:justify-between gap-4">
        <div>
          <div class="font-semibold">Filters</div>
          <div class="text-sm text-slate-400">Use date range and/or project, then refresh</div>
        </div>
        <div class="flex flex-wrap items-end gap-2">
          <div>
            <div class="text-xs text-slate-400 mb-1">From</div>
            <input v-model="filterFrom" type="date" class="rounded-lg border border-slate-700 bg-slate-950/60 px-3 py-2 text-sm" />
          </div>
          <div>
            <div class="text-xs text-slate-400 mb-1">To</div>
            <input v-model="filterTo" type="date" class="rounded-lg border border-slate-700 bg-slate-950/60 px-3 py-2 text-sm" />
          </div>
          <div>
            <div class="text-xs text-slate-400 mb-1">Project</div>
            <select v-model="filterProjectId" class="rounded-lg border border-slate-700 bg-slate-950/60 px-3 py-2 text-sm">
              <option value="">All</option>
              <option v-for="p in projects" :key="p.id" :value="String(p.id)">{{ p.name }}</option>
            </select>
          </div>
          <button class="rounded-lg bg-slate-800 hover:bg-slate-700 px-3 py-2 text-sm font-semibold" @click="load">
            {{ loading ? "Refreshing…" : "Refresh" }}
          </button>
        </div>
      </div>

      <div class="mt-4 flex flex-wrap gap-2">
        <button class="rounded-lg bg-indigo-600 hover:bg-indigo-500 px-3 py-2 text-sm font-semibold" @click="exportSummaryPdf">
          Export PDF Summary
        </button>
        <button class="rounded-lg bg-slate-800 hover:bg-slate-700 px-3 py-2 text-sm font-semibold" @click="exportDonationsCsv">
          Export Contributions CSV
        </button>
        <button class="rounded-lg bg-slate-800 hover:bg-slate-700 px-3 py-2 text-sm font-semibold" @click="exportExpensesCsv">
          Export Expenses CSV
        </button>
        <button class="rounded-lg bg-slate-800 hover:bg-slate-700 px-3 py-2 text-sm font-semibold" @click="exportProjectsCsv">
          Export Projects CSV
        </button>
      </div>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <div class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
        <div class="text-xs uppercase tracking-wider text-slate-400">Contributions</div>
        <div class="mt-2 text-2xl font-bold">{{ formatPHPFromCents(summary?.total_donations_cents || 0) }}</div>
      </div>
      <div class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
        <div class="text-xs uppercase tracking-wider text-slate-400">Expenses</div>
        <div class="mt-2 text-2xl font-bold">{{ formatPHPFromCents(summary?.total_expenses_cents || 0) }}</div>
      </div>
      <div class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
        <div class="text-xs uppercase tracking-wider text-slate-400">Balance</div>
        <div class="mt-2 text-2xl font-bold">{{ formatPHPFromCents(summary?.balance_cents || 0) }}</div>
      </div>
    </div>

    <div class="rounded-2xl border border-slate-800 bg-slate-900/40 overflow-hidden">
      <div class="p-5">
        <div class="font-semibold">Project Balances</div>
        <div class="text-sm text-slate-400">Computed from contributions and expenses within the filter</div>
      </div>
      <div class="border-t border-slate-800">
        <table class="w-full text-sm">
          <thead class="bg-slate-950/40 text-slate-300">
            <tr>
              <th class="text-left p-3 font-medium">Project</th>
              <th class="text-right p-3 font-medium">Contributions</th>
              <th class="text-right p-3 font-medium">Expenses</th>
              <th class="text-right p-3 font-medium">Balance</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in balances" :key="row.project_id" class="border-t border-slate-800">
              <td class="p-3">{{ row.project_name }}</td>
              <td class="p-3 text-right">{{ formatPHPFromCents(row.donations_cents) }}</td>
              <td class="p-3 text-right">{{ formatPHPFromCents(row.expenses_cents) }}</td>
              <td class="p-3 text-right font-semibold">{{ formatPHPFromCents(row.balance_cents) }}</td>
            </tr>
            <tr v-if="balances.length === 0" class="border-t border-slate-800">
              <td class="p-3 text-slate-400" colspan="4">No projects found.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
