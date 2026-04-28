<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { donationsCreate, donorsCreate, donorsList, exportCsv, exportPdf, projectReport } from "../../lib/api";
import { formatPHPFromCents } from "../../lib/money";
import { centsFromPesos } from "../../lib/money";
import type { Donor, ProjectReport } from "../../lib/types";

const props = defineProps<{
  sessionToken: string;
  projectId: number;
}>();

const emit = defineEmits<{
  (e: "back"): void;
}>();

const filterFrom = ref<string>("");
const filterTo = ref<string>("");

const report = ref<ProjectReport | null>(null);
const loading = ref(true);
const errorMessage = ref<string | null>(null);

const donors = ref<Donor[]>([]);

const showAddDonation = ref(false);
const addDate = ref<string>(new Date().toISOString().slice(0, 10));
const addAmount = ref<string>("");
const addAnonymous = ref<boolean>(false);
const addDonorName = ref<string>("");
const addNotes = ref<string>("");
const addSubmitting = ref(false);

const filter = computed(() => ({
  from: filterFrom.value || null,
  to: filterTo.value || null,
  project_id: props.projectId,
}));

const progressPct = computed(() => {
  const target = report.value?.target_amount_cents || 0;
  const donated = report.value?.donations_cents || 0;
  if (target <= 0) return 0;
  return Math.min(100, Math.round((donated / target) * 100));
});

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    donors.value = await donorsList(props.sessionToken);
    report.value = await projectReport(props.sessionToken, props.projectId, filter.value);
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

async function submitAddDonation() {
  errorMessage.value = null;
  addSubmitting.value = true;
  try {
    const amountCents = centsFromPesos(addAmount.value);
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
    title: `Project Funds Tracker — ${report.value?.project.name || "Project"} Summary`,
    filter: filter.value,
    dest_path: dest,
  });
}

async function exportProjectDonationsCsv() {
  const dest = await save({
    defaultPath: `project-${props.projectId}-contributions-${filterFrom.value || "all"}-${filterTo.value || "all"}.csv`,
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!dest) return;
  await exportCsv(props.sessionToken, { kind: "donations", filter: filter.value, dest_path: dest });
}

async function exportProjectExpensesCsv() {
  const dest = await save({
    defaultPath: `project-${props.projectId}-expenses-${filterFrom.value || "all"}-${filterTo.value || "all"}.csv`,
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!dest) return;
  await exportCsv(props.sessionToken, { kind: "expenses", filter: filter.value, dest_path: dest });
}

onMounted(load);
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between gap-3">
      <div class="flex items-center gap-3">
        <button class="rounded-xl bg-slate-800 hover:bg-slate-700 px-3 py-2 text-sm font-semibold" @click="emit('back')">
          ← Back
        </button>
        <div>
          <div class="text-xs text-slate-400">Project</div>
          <div class="text-2xl font-bold">{{ report?.project.name || "Loading…" }}</div>
        </div>
      </div>
      <div class="flex gap-2">
        <button class="rounded-xl bg-indigo-600 hover:bg-indigo-500 px-4 py-2 text-sm font-semibold" @click="openAddDonation">
          + Add Contribution
        </button>
        <button class="rounded-xl bg-slate-800 hover:bg-slate-700 px-4 py-2 text-sm font-semibold" @click="load">
          Refresh
        </button>
      </div>
    </div>

    <div v-if="errorMessage" class="rounded-xl border border-rose-500/40 bg-rose-500/10 p-4 text-rose-200">
      {{ errorMessage }}
    </div>

    <div v-else-if="loading" class="text-slate-300">Loading…</div>

    <div v-else-if="report" class="space-y-6">
      <div class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
        <div class="flex flex-col md:flex-row md:items-end md:justify-between gap-4">
          <div>
            <div class="text-sm text-slate-400">
              Status: <span class="text-slate-200 font-semibold">{{ report.project.status }}</span>
            </div>
            <div class="mt-2 text-sm text-slate-400">
              Dates: <span class="text-slate-200">{{ report.project.start_date || "-" }} → {{ report.project.end_date || "-" }}</span>
            </div>
            <div v-if="report.project.description" class="mt-2 text-sm text-slate-300">
              {{ report.project.description }}
            </div>
          </div>

          <div class="flex flex-wrap gap-2 text-base">
            <button class="rounded-lg bg-indigo-600 hover:bg-indigo-500 px-3 py-2 text-sm font-semibold" @click="exportProjectSummaryPdf">
              Export PDF
            </button>
            <button class="rounded-lg bg-slate-800 hover:bg-slate-700 px-3 py-2 text-sm font-semibold" @click="exportProjectDonationsCsv">
              Contributions CSV
            </button>
            <button class="rounded-lg bg-slate-800 hover:bg-slate-700 px-3 py-2 text-sm font-semibold" @click="exportProjectExpensesCsv">
              Expenses CSV
            </button>
          </div>
        </div>

        <div class="mt-5 grid grid-cols-1 md:grid-cols-5 gap-4 text-base">
          <div class="rounded-2xl border border-slate-800 bg-slate-950/30 p-4">
            <div class="text-xs uppercase tracking-wider text-slate-400">Target</div>
            <div class="mt-2 text-xl font-bold">{{ formatPHPFromCents(report.target_amount_cents) }}</div>
          </div>
          <div class="rounded-2xl border border-slate-800 bg-slate-950/30 p-4">
            <div class="text-xs uppercase tracking-wider text-slate-400">Accumulated</div>
            <div class="mt-2 text-xl font-bold">{{ formatPHPFromCents(report.donations_cents) }}</div>
          </div>
          <div class="rounded-2xl border border-slate-800 bg-slate-950/30 p-4">
            <div class="text-xs uppercase tracking-wider text-slate-400">Expenses</div>
            <div class="mt-2 text-xl font-bold">{{ formatPHPFromCents(report.expenses_cents) }}</div>
          </div>
          <div class="rounded-2xl border border-slate-800 bg-slate-950/30 p-4">
            <div class="text-xs uppercase tracking-wider text-slate-400">Balance</div>
            <div class="mt-2 text-xl font-bold">{{ formatPHPFromCents(report.balance_cents) }}</div>
          </div>
          <div class="rounded-2xl border border-slate-800 bg-slate-950/30 p-4">
            <div class="text-xs uppercase tracking-wider text-slate-400">Remaining to Target</div>
            <div class="mt-2 text-xl font-bold">{{ formatPHPFromCents(report.remaining_to_target_cents) }}</div>
          </div>
        </div>

        <div class="mt-5">
          <div class="flex items-center justify-between mb-2">
            <div class="text-sm text-slate-300">Progress</div>
            <div class="text-sm text-slate-300 font-semibold">{{ progressPct }}%</div>
          </div>
          <div class="h-3 rounded-full bg-slate-800 overflow-hidden">
            <div class="h-full bg-indigo-600" :style="{ width: progressPct + '%' }"></div>
          </div>
        </div>
      </div>

      <div class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
        <div class="flex flex-wrap items-end gap-2 justify-between">
          <div>
            <div class="font-semibold">Filter</div>
            <div class="text-sm text-slate-400">Apply date range for this project</div>
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
            <button class="rounded-lg bg-slate-800 hover:bg-slate-700 px-3 py-2 text-sm font-semibold" @click="load">Apply</button>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 text-base">
        <div class="lg:col-span-2 rounded-2xl border border-slate-800 bg-slate-900/40 overflow-hidden">
          <div class="p-5">
            <div class="font-semibold">Contributions</div>
            <div class="text-sm text-slate-400">Who gave for this project (anonymous names hidden)</div>
          </div>
          <div class="border-t border-slate-800">
            <table class="w-full text-base table-fixed">
              <thead class="bg-slate-950/40 text-slate-300">
                <tr>
                  <th class="text-left p-3 font-medium w-32">Date</th>
                  <th class="text-left p-3 font-medium w-48">Name</th>
                  <th class="text-right p-3 font-medium w-40">Amount</th>
                  <th class="text-left p-3 font-medium">Notes</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="d in report.donations" :key="d.id" class="border-t border-slate-800">
                  <td class="p-3 truncate" :title="d.donated_at">{{ d.donated_at }}</td>
                  <td class="p-3">
                    <span v-if="d.anonymous" class="rounded-full bg-slate-800 px-2 py-1 text-xs">Anonymous</span>
                    <span v-else>{{ d.donor_name || "-" }}</span>
                  </td>
                  <td class="p-3 text-right font-semibold">{{ formatPHPFromCents(d.amount_cents) }}</td>
                  <td class="p-3 text-slate-300 truncate" :title="d.notes || ''">{{ d.notes || "" }}</td>
                </tr>
                <tr v-if="report.donations.length === 0" class="border-t border-slate-800">
                  <td class="p-3 text-slate-400" colspan="4">No contributions for this project yet.</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <div class="rounded-2xl border border-slate-800 bg-slate-900/40 overflow-hidden">
          <div class="p-5">
            <div class="font-semibold">Top Names</div>
            <div class="text-sm text-slate-400">Non-anonymous totals (top 10)</div>
          </div>
          <div class="border-t border-slate-800">
            <table class="w-full text-base table-fixed">
              <thead class="bg-slate-950/40 text-slate-300">
                <tr>
                  <th class="text-left p-3 font-medium w-1/2">Name</th>
                  <th class="text-right p-3 font-medium w-1/2">Total</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="row in report.top_donors" :key="row.donor_name" class="border-t border-slate-800">
                  <td class="p-3">{{ row.donor_name }}</td>
                  <td class="p-3 text-right font-semibold">{{ formatPHPFromCents(row.total_cents) }}</td>
                </tr>
                <tr v-if="report.top_donors.length === 0" class="border-t border-slate-800">
                  <td class="p-3 text-slate-400" colspan="2">No names yet.</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <div class="rounded-2xl border border-slate-800 bg-slate-900/40 overflow-hidden">
        <div class="p-5">
          <div class="font-semibold">Expenses</div>
          <div class="text-sm text-slate-400">Spending linked to this project</div>
        </div>
        <div class="border-t border-slate-800">
          <table class="w-full text-base table-fixed">
            <thead class="bg-slate-950/40 text-slate-300">
              <tr>
                <th class="text-left p-3 font-medium w-32">Date</th>
                <th class="text-left p-3 font-medium w-44">Category</th>
                <th class="text-right p-3 font-medium w-40">Amount</th>
                <th class="text-left p-3 font-medium w-44">Payee</th>
                <th class="text-left p-3 font-medium">Notes</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="e in report.expenses" :key="e.id" class="border-t border-slate-800">
                <td class="p-3 truncate" :title="e.spent_at">{{ e.spent_at }}</td>
                <td class="p-3 truncate" :title="e.category_name || '-'">{{ e.category_name || "-" }}</td>
                <td class="p-3 text-right font-semibold">{{ formatPHPFromCents(e.amount_cents) }}</td>
                <td class="p-3 text-slate-300 truncate" :title="e.payee || ''">{{ e.payee || "" }}</td>
                <td class="p-3 text-slate-300 truncate" :title="e.notes || ''">{{ e.notes || "" }}</td>
              </tr>
              <tr v-if="report.expenses.length === 0" class="border-t border-slate-800">
                <td class="p-3 text-slate-400" colspan="5">No expenses for this project yet.</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
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
              <div class="mt-1 text-xs text-slate-500">If this name doesn’t exist yet, it will be created.</div>
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
              {{ addSubmitting ? "Saving…" : "Save Contribution" }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
