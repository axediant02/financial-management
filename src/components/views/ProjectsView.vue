<script setup lang="ts">
import { onMounted, ref } from "vue";
import { projectBalances, projectsCreate, projectsDelete, projectsList } from "../../lib/api";
import { notify } from "../../lib/feedback";
import { centsFromPesos, formatPHPFromCents } from "../../lib/money";
import type { Project, ProjectBalanceRow } from "../../lib/types";

const props = defineProps<{ sessionToken: string }>();
const emit = defineEmits<{ (e: "open-project", id: number): void }>();

const loading = ref(true);
const errorMessage = ref<string | null>(null);
const projects = ref<Project[]>([]);
const balances = ref<ProjectBalanceRow[]>([]);

const showAddProject = ref(false);

const formName = ref("");
const formTarget = ref("");
const formStatus = ref("active");
const formStart = ref("");
const formEnd = ref("");
const formDescription = ref("");

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    projects.value = await projectsList(props.sessionToken);
    balances.value = await projectBalances(props.sessionToken, { from: null, to: null, project_id: null });
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    loading.value = false;
  }
}

function contributionsForProject(projectId: number): number {
  return balances.value.find((b) => b.project_id === projectId)?.donations_cents ?? 0;
}

function remainingToTargetForProject(project: Project): number {
  const total = contributionsForProject(project.id);
  return Math.max(0, project.target_amount_cents - total);
}

async function createProject() {
  errorMessage.value = null;
  const name = formName.value.trim();
  if (!confirm(`Save project "${name}"?`)) return;
  try {
    await projectsCreate(props.sessionToken, {
      name: formName.value,
      description: formDescription.value || null,
      target_amount_cents: centsFromPesos(formTarget.value),
      status: formStatus.value,
      start_date: formStart.value || null,
      end_date: formEnd.value || null,
    });
    formName.value = "";
    formTarget.value = "";
    formDescription.value = "";
    showAddProject.value = false;
    await load();
    notify(`Project "${name}" saved.`);
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

async function removeProject(id: number) {
  if (!confirm("Delete this project? (Donations/expenses linked will remain but project link becomes empty)")) return;
  try {
    await projectsDelete(props.sessionToken, id);
    await load();
    notify("Project deleted.");
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

function openProject(id: number) {
  emit("open-project", id);
}

onMounted(load);
</script>

<template>
  <div class="space-y-6">
    <div v-if="errorMessage" class="rounded-xl border border-rose-500/40 bg-rose-500/10 p-4 text-rose-200">
      {{ errorMessage }}
    </div>

    <div class="rounded-2xl border border-slate-800 bg-slate-900/40 overflow-hidden">
      <div class="p-5 flex items-center justify-between">
        <div>
          <div class="font-semibold">Projects</div>
          <div class="text-sm text-slate-400">Budgets/targets for transparency reporting</div>
        </div>
        <button class="rounded-lg bg-slate-800 hover:bg-slate-700 px-3 py-2 text-sm font-semibold" @click="load">Refresh</button>
      </div>
      <div class="border-t border-slate-800">
        <table class="w-full text-sm table-fixed">
          <thead class="bg-slate-950/40 text-slate-300">
            <tr>
              <th class="text-left p-3 font-medium w-[22%]">Name</th>
              <th class="text-center p-3 font-medium w-[10%]">Status</th>
              <th class="text-center p-3 font-medium w-[14%]">Total Contributions</th>
              <th class="text-center p-3 font-medium w-[14%]">Target</th>
              <th class="text-center p-3 font-medium w-[16%]">Remaining to Target</th>
              <th class="text-left p-3 font-medium w-[14%]">Description</th>
              <th class="text-center p-3 font-medium w-[10%]">Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-if="loading" class="border-t border-slate-800">
              <td class="p-3 text-slate-400" colspan="7">Loading…</td>
            </tr>
            <tr
              v-for="p in projects"
              :key="p.id"
              class="border-t border-slate-800 hover:bg-slate-950/40 cursor-pointer"
              @click="openProject(p.id)"
            >
              <td class="p-3 font-semibold align-middle truncate" :title="p.name">{{ p.name }}</td>
              <td class="p-3 text-center align-middle">
                <span class="inline-flex items-center rounded-full bg-slate-800 px-2 py-1 text-xs">
                  {{ p.status }}
                </span>
              </td>
              <td class="p-3 text-center align-middle tabular-nums font-semibold">
                <span class="inline-flex items-center rounded-full bg-emerald-500/10 border border-emerald-500/20 px-2.5 py-1 text-emerald-200">
                  {{ formatPHPFromCents(contributionsForProject(p.id)) }}
                </span>
              </td>
              <td class="p-3 text-center align-middle tabular-nums">
                <span class="inline-flex items-center rounded-full bg-indigo-500/10 border border-indigo-500/20 px-2.5 py-1 text-indigo-200">
                  {{ formatPHPFromCents(p.target_amount_cents) }}
                </span>
              </td>
              <td class="p-3 text-center align-middle tabular-nums">
                <span
                  class="inline-flex items-center rounded-full border px-2.5 py-1"
                  :class="
                    remainingToTargetForProject(p) === 0
                      ? 'bg-emerald-500/10 border-emerald-500/20 text-emerald-200'
                      : 'bg-amber-500/10 border-amber-500/20 text-amber-200'
                  "
                >
                  {{ formatPHPFromCents(remainingToTargetForProject(p)) }}
                </span>
              </td>
              <td class="p-3 text-slate-300 align-middle truncate" :title="p.description || ''">
                {{ p.description || "" }}
              </td>
              <td class="p-3 text-center align-middle" @click.stop>
                <button class="rounded-lg bg-rose-600 hover:bg-rose-500 px-3 py-1.5 text-xs font-semibold" @click="removeProject(p.id)">
                  Delete
                </button>
              </td>
            </tr>
            <tr v-if="!loading && projects.length === 0" class="border-t border-slate-800">
              <td class="p-3 text-slate-400" colspan="7">No projects yet.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <button
      type="button"
      class="fixed bottom-6 right-6 h-14 w-14 rounded-2xl bg-indigo-600 hover:bg-indigo-500 shadow-2xl shadow-indigo-600/25 border border-indigo-400/30 flex items-center justify-center"
      title="Add project"
      @click="showAddProject = true"
    >
      <svg class="h-7 w-7 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v14m7-7H5" />
      </svg>
    </button>

    <div v-if="showAddProject" class="fixed inset-0 z-50">
      <div class="absolute inset-0 bg-black/60" @click="showAddProject = false"></div>
      <div class="absolute inset-0 flex items-center justify-center p-4">
        <div class="w-full max-w-3xl rounded-2xl border border-slate-800 bg-slate-950 p-6 shadow-2xl">
          <div class="flex items-center justify-between">
            <div>
              <div class="text-lg font-bold">Add Project</div>
              <div class="text-sm text-slate-400">Create a project with a target budget for tracking.</div>
            </div>
            <button class="rounded-lg bg-slate-800 hover:bg-slate-700 px-3 py-2 text-sm font-semibold" @click="showAddProject = false">
              Close
            </button>
          </div>

          <div class="mt-5 grid grid-cols-1 md:grid-cols-6 gap-3">
            <div class="md:col-span-2">
              <label class="block text-xs text-slate-400 mb-1">Name</label>
              <input v-model="formName" placeholder="e.g. Instruments" class="w-full rounded-xl border border-slate-700 bg-slate-900/40 px-3 py-2" />
            </div>
            <div class="md:col-span-1">
              <label class="block text-xs text-slate-400 mb-1">Target (PHP)</label>
              <input v-model="formTarget" inputmode="decimal" placeholder="0.00" class="w-full rounded-xl border border-slate-700 bg-slate-900/40 px-3 py-2" />
            </div>
            <div class="md:col-span-1">
              <label class="block text-xs text-slate-400 mb-1">Status</label>
              <select v-model="formStatus" class="w-full rounded-xl border border-slate-700 bg-slate-900/40 px-3 py-2">
                <option value="active">active</option>
                <option value="paused">paused</option>
                <option value="completed">completed</option>
              </select>
            </div>
            <div class="md:col-span-1">
              <label class="block text-xs text-slate-400 mb-1">Start</label>
              <input v-model="formStart" type="date" class="w-full rounded-xl border border-slate-700 bg-slate-900/40 px-3 py-2" />
            </div>
            <div class="md:col-span-1">
              <label class="block text-xs text-slate-400 mb-1">End (optional)</label>
              <input v-model="formEnd" type="date" class="w-full rounded-xl border border-slate-700 bg-slate-900/40 px-3 py-2" />
            </div>
            <div class="md:col-span-6">
              <label class="block text-xs text-slate-400 mb-1">Description</label>
              <input v-model="formDescription" placeholder="optional" class="w-full rounded-xl border border-slate-700 bg-slate-900/40 px-3 py-2" />
            </div>
          </div>

          <div class="mt-6 flex items-center justify-end gap-2">
            <button class="rounded-xl bg-slate-800 hover:bg-slate-700 px-4 py-2 font-semibold" @click="showAddProject = false">
              Cancel
            </button>
            <button class="rounded-xl bg-indigo-600 hover:bg-indigo-500 px-4 py-2 font-semibold" @click="createProject">
              Save Project
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
