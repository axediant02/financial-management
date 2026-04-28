<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { donationsCreate, donationsDelete, donationsList, donorsCreate, donorsList, projectsList } from "../../lib/api";
import { centsFromPesos, formatPHPFromCents } from "../../lib/money";
import type { Donation, Donor, Project } from "../../lib/types";

const props = defineProps<{ sessionToken: string }>();

const loading = ref(true);
const errorMessage = ref<string | null>(null);
const items = ref<Donation[]>([]);
const donors = ref<Donor[]>([]);
const projects = ref<Project[]>([]);

const filterFrom = ref<string>("");
const filterTo = ref<string>("");
const filterProjectId = ref<string>("");

const formDate = ref<string>(new Date().toISOString().slice(0, 10));
const formAmount = ref<string>("");
const formDonorId = ref<string>("");
const newDonorName = ref<string>("");
const formAnonymous = ref<boolean>(false);
const formProjectId = ref<string>("");
const formNotes = ref<string>("");

const donorsById = computed(() => new Map(donors.value.map((d) => [d.id, d])));
const projectsById = computed(() => new Map(projects.value.map((p) => [p.id, p])));

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    donors.value = await donorsList(props.sessionToken);
    projects.value = await projectsList(props.sessionToken);
    items.value = await donationsList(props.sessionToken, {
      from: filterFrom.value || null,
      to: filterTo.value || null,
      project_id: filterProjectId.value ? Number(filterProjectId.value) : null,
    });
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function createDonation() {
  errorMessage.value = null;
  try {
    const amountCents = centsFromPesos(formAmount.value);
    await donationsCreate(props.sessionToken, {
      donated_at: formDate.value,
      amount_cents: amountCents,
      donor_id: formAnonymous.value ? null : formDonorId.value ? Number(formDonorId.value) : null,
      anonymous: formAnonymous.value,
      notes: formNotes.value || null,
      project_id: formProjectId.value ? Number(formProjectId.value) : null,
    });
    formAmount.value = "";
    formNotes.value = "";
    await load();
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

async function addDonor() {
  errorMessage.value = null;
  const name = newDonorName.value.trim();
  if (!name) return;
  try {
    await donorsCreate(props.sessionToken, { name, notes: null });
    newDonorName.value = "";
    await load();
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

async function removeDonation(id: number) {
  if (!confirm("Delete this contribution?")) return;
  try {
    await donationsDelete(props.sessionToken, id);
    await load();
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

onMounted(load);
</script>

<template>
  <div class="space-y-6">
    <div v-if="errorMessage" class="rounded-xl border border-rose-500/40 bg-rose-500/10 p-4 text-rose-200">
      {{ errorMessage }}
    </div>

    <div class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
      <div class="font-semibold">Add Contribution</div>
      <div class="mt-4 grid grid-cols-1 md:grid-cols-6 gap-3">
        <div class="md:col-span-1">
          <label class="block text-xs text-slate-400 mb-1">Date</label>
          <input v-model="formDate" type="date" class="w-full rounded-xl border border-slate-700 bg-slate-950/60 px-3 py-2" />
        </div>
        <div class="md:col-span-1">
          <label class="block text-xs text-slate-400 mb-1">Amount (PHP)</label>
          <input v-model="formAmount" inputmode="decimal" placeholder="0.00" class="w-full rounded-xl border border-slate-700 bg-slate-950/60 px-3 py-2" />
        </div>
        <div class="md:col-span-1">
          <label class="block text-xs text-slate-400 mb-1">Anonymous</label>
          <label class="flex items-center gap-2 rounded-xl border border-slate-700 bg-slate-950/60 px-3 py-2">
            <input v-model="formAnonymous" type="checkbox" class="accent-indigo-500" />
            <span class="text-sm text-slate-200">Yes</span>
          </label>
        </div>
        <div class="md:col-span-1">
          <label class="block text-xs text-slate-400 mb-1">Name</label>
          <select v-model="formDonorId" :disabled="formAnonymous" class="w-full rounded-xl border border-slate-700 bg-slate-950/60 px-3 py-2 disabled:opacity-60">
            <option value="">(optional)</option>
            <option v-for="d in donors" :key="d.id" :value="String(d.id)">{{ d.name }}</option>
          </select>
          <div class="mt-2 flex gap-2">
            <input
              v-model="newDonorName"
              :disabled="formAnonymous"
              placeholder="Quick add name…"
              class="flex-1 rounded-lg border border-slate-700 bg-slate-950/60 px-3 py-2 text-sm disabled:opacity-60"
            />
            <button
              type="button"
              :disabled="formAnonymous || !newDonorName.trim()"
              class="rounded-lg bg-slate-800 hover:bg-slate-700 disabled:opacity-60 px-3 py-2 text-sm font-semibold"
              @click="addDonor"
            >
              Add
            </button>
          </div>
        </div>
        <div class="md:col-span-1">
          <label class="block text-xs text-slate-400 mb-1">Project</label>
          <select v-model="formProjectId" class="w-full rounded-xl border border-slate-700 bg-slate-950/60 px-3 py-2">
            <option value="">(none)</option>
            <option v-for="p in projects" :key="p.id" :value="String(p.id)">{{ p.name }}</option>
          </select>
        </div>
        <div class="md:col-span-1">
          <label class="block text-xs text-slate-400 mb-1">Notes</label>
          <input v-model="formNotes" placeholder="optional" class="w-full rounded-xl border border-slate-700 bg-slate-950/60 px-3 py-2" />
        </div>
      </div>
      <div class="mt-4">
        <button class="rounded-xl bg-indigo-600 hover:bg-indigo-500 px-4 py-2 font-semibold" @click="createDonation">
          Save Contribution
        </button>
      </div>
    </div>

    <div class="rounded-2xl border border-slate-800 bg-slate-900/40 overflow-hidden">
      <div class="p-5 flex items-center justify-between gap-4">
        <div>
          <div class="font-semibold">Contributions</div>
          <div class="text-sm text-slate-400">Latest 500 records</div>
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
          <button class="rounded-lg bg-slate-800 hover:bg-slate-700 px-3 py-2 text-sm font-semibold" @click="load">Apply</button>
        </div>
      </div>

      <div class="border-t border-slate-800">
        <table class="w-full text-sm table-fixed">
          <thead class="bg-slate-950/40 text-slate-300">
            <tr>
              <th class="text-left p-3 font-medium w-[16.666%]">Date</th>
              <th class="text-left p-3 font-medium w-[16.666%]">Name</th>
              <th class="text-left p-3 font-medium w-[16.666%]">Project</th>
              <th class="text-right p-3 font-medium w-[16.666%]">Amount</th>
              <th class="text-left p-3 font-medium w-[16.666%]">Notes</th>
              <th class="text-right p-3 font-medium w-[16.666%]">Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-if="loading" class="border-t border-slate-800">
              <td class="p-3 text-slate-400" colspan="6">Loading…</td>
            </tr>
            <tr v-for="d in items" :key="d.id" class="border-t border-slate-800">
              <td class="p-3 truncate" :title="d.donated_at">{{ d.donated_at }}</td>
              <td class="p-3 truncate" :title="d.anonymous ? 'Anonymous' : (d.donor_id ? (donorsById.get(d.donor_id)?.name || `#${d.donor_id}`) : '-')">
                <span v-if="d.anonymous" class="rounded-full bg-slate-800 px-2 py-1 text-xs">Anonymous</span>
                <span v-else>{{ d.donor_id ? donorsById.get(d.donor_id)?.name || `#${d.donor_id}` : "-" }}</span>
              </td>
              <td class="p-3 truncate" :title="d.project_id ? (projectsById.get(d.project_id)?.name || `#${d.project_id}`) : '-'">
                {{ d.project_id ? projectsById.get(d.project_id)?.name || `#${d.project_id}` : "-" }}
              </td>
              <td class="p-3 text-right font-semibold">{{ formatPHPFromCents(d.amount_cents) }}</td>
              <td class="p-3 text-slate-300 truncate" :title="d.notes || ''">{{ d.notes || "" }}</td>
              <td class="p-3 text-right">
                <button class="rounded-lg bg-rose-600 hover:bg-rose-500 px-3 py-1.5 text-xs font-semibold" @click="removeDonation(d.id)">
                  Delete
                </button>
              </td>
            </tr>
            <tr v-if="!loading && items.length === 0" class="border-t border-slate-800">
              <td class="p-3 text-slate-400" colspan="6">No contributions found.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
