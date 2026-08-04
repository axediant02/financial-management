<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { exportCsv, projectBalances, projectsCreate, projectsDelete, projectsList } from "../../lib/api";
import { notify } from "../../lib/feedback";
import { centsFromPesos, formatPHPFromCents } from "../../lib/money";
import type { Project, ProjectBalanceRow } from "../../lib/types";

type ProjectStatusFilter = "all" | "active" | "paused" | "completed";

const props = defineProps<{ sessionToken: string }>();
const emit = defineEmits<{ (e: "open-project", id: number): void }>();

const loading = ref(true);
const errorMessage = ref<string | null>(null);
const projects = ref<Project[]>([]);
const balances = ref<ProjectBalanceRow[]>([]);
const searchQuery = ref("");
const statusFilter = ref<ProjectStatusFilter>("all");

const showAddProject = ref(false);
const formName = ref("");
const formTarget = ref("");
const formStatus = ref<"active" | "paused" | "completed">("active");
const formStart = ref("");
const formEnd = ref("");
const formDescription = ref("");
const startDateMax = computed(() => formEnd.value || undefined);
const endDateMin = computed(() => formStart.value || undefined);

function statusLabel(value: string) {
  if (value === "paused") return "on hold";
  return value;
}

function statusTone(value: string) {
  if (value === "completed") return "border-[#b9c5d6] bg-[#f5f7fb] text-[#5f6d82]";
  if (value === "paused") return "border-[#e2b39f] bg-[#fff4ef] text-[#b35a3f]";
  return "border-[#a7d0ae] bg-[#f2faf4] text-[#2e7a4f]";
}

function projectCode(id: number) {
  return `PRJ-${String(id).padStart(3, "0")}`;
}

const balanceByProjectId = computed(() => new Map(balances.value.map((row) => [row.project_id, row])));

const rows = computed(() => {
  const query = searchQuery.value.trim().toLowerCase();
  return projects.value
    .map((project) => {
      const balance = balanceByProjectId.value.get(project.id);
      const received = balance?.donations_cents ?? 0;
      const spent = balance?.expenses_cents ?? 0;
      const balanceCents = balance?.balance_cents ?? received - spent;
      return {
        project,
        received,
        spent,
        balanceCents,
      };
    })
    .filter(({ project }) => {
      if (statusFilter.value !== "all" && project.status !== statusFilter.value) return false;
      if (!query) return true;
      const code = projectCode(project.id).toLowerCase();
      return (
        project.name.toLowerCase().includes(query) ||
        code.includes(query) ||
        (project.description || "").toLowerCase().includes(query)
      );
    });
});

const totals = computed(() =>
  rows.value.reduce(
    (acc, row) => {
      acc.target += row.project.target_amount_cents;
      acc.received += row.received;
      acc.spent += row.spent;
      acc.balance += row.balanceCents;
      return acc;
    },
    { target: 0, received: 0, spent: 0, balance: 0 },
  ),
);

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    const [projectRows, balanceRows] = await Promise.all([
      projectsList(props.sessionToken),
      projectBalances(props.sessionToken, { from: null, to: null, project_id: null }),
    ]);
    projects.value = projectRows;
    balances.value = balanceRows;
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function exportProjects() {
  const dest = await save({
    defaultPath: "projects.csv",
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!dest) return;
  await exportCsv(props.sessionToken, {
    kind: "projects",
    filter: { from: null, to: null, project_id: null },
    dest_path: dest,
  });
  notify("Projects export created.");
}

async function createProject() {
  errorMessage.value = null;
  const name = formName.value.trim();
  if (!name) {
    notify("Enter a project name first.");
    return;
  }
  if (formStart.value && formEnd.value && formEnd.value < formStart.value) {
    notify("End date cannot be earlier than start date.");
    return;
  }
  if (!confirm(`Save project "${name}"?`)) return;

  try {
    await projectsCreate(props.sessionToken, {
      name,
      description: formDescription.value || null,
      target_amount_cents: centsFromPesos(formTarget.value),
      status: formStatus.value,
      start_date: formStart.value || null,
      end_date: formEnd.value || null,
    });

    formName.value = "";
    formTarget.value = "";
    formDescription.value = "";
    formStatus.value = "active";
    formStart.value = "";
    formEnd.value = "";
    showAddProject.value = false;

    await load();
    notify(`Project "${name}" saved.`);
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

async function removeProject(id: number, name: string) {
  if (!confirm(`Delete project "${name}"? Donations and expenses will remain, but their project link will be cleared.`)) {
    return;
  }
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

function periodLabel(project: Project) {
  const start = project.start_date || "—";
  const end = project.end_date || "—";
  return `${start} - ${end}`;
}

onMounted(load);
</script>

<template>
  <div class="space-y-5">
    <section class="ledger-panel rounded-[26px] px-6 py-6 md:px-8">
      <div class="flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between">
        <div class="max-w-3xl">
          <div class="ledger-eyebrow text-[11px] text-[#8d6f2f]">Book of Accounts - FY 2026</div>
          <h2 class="ledger-heading mt-2 text-4xl font-normal text-[#1f3558] md:text-[4.1rem]">
            Projects Register
          </h2>
          <p class="mt-3 max-w-2xl text-[15px] leading-7 text-[#6a6b5d]">
            All fund drives on record with targets and running balances.
          </p>
        </div>

        <div class="flex flex-wrap gap-3">
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-xl border border-[#d7c49a] bg-[#fbf7eb] px-4 py-3 text-sm font-semibold text-[#243858] transition hover:bg-[#f4ecd7]"
            @click="exportProjects"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <path d="M12 3v12" />
              <path d="m8 11 4 4 4-4" />
              <path d="M4 21h16" />
            </svg>
            Export
          </button>
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-xl border border-[#243858] bg-[#243858] px-4 py-3 text-sm font-semibold text-white transition hover:bg-[#1f2f4a]"
            @click="showAddProject = true"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <path d="M12 5v14" />
              <path d="M5 12h14" />
            </svg>
            New Project
          </button>
        </div>
      </div>
    </section>

    <div v-if="errorMessage" class="rounded-2xl border border-[#e3b2a3] bg-[#fff4ef] px-4 py-3 text-[#9d3f27]">
      {{ errorMessage }}
    </div>

    <section class="ledger-panel rounded-[18px] px-4 py-4 md:px-5">
      <div class="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <label class="flex min-w-0 flex-1 items-center gap-3 rounded-[12px] border border-[#d7c49a] bg-[#fbf7eb] px-4 py-3">
          <svg class="h-4 w-4 shrink-0 text-[#7a755f]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <circle cx="11" cy="11" r="7" />
            <path d="m20 20-3.5-3.5" />
          </svg>
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search project name or code..."
            class="w-full bg-transparent text-sm text-[#243858] outline-none placeholder:text-[#7a755f]"
          />
        </label>

        <div class="flex flex-wrap gap-2">
          <button
            type="button"
            class="rounded-lg border px-4 py-2 text-sm font-semibold transition"
            :class="statusFilter === 'all' ? 'border-[#243858] bg-[#243858] text-white' : 'border-[#d7c49a] bg-[#fbf7eb] text-[#243858] hover:bg-[#f4ecd7]'"
            @click="statusFilter = 'all'"
          >
            All
          </button>
          <button
            type="button"
            class="rounded-lg border px-4 py-2 text-sm font-semibold transition"
            :class="statusFilter === 'active' ? 'border-[#243858] bg-[#243858] text-white' : 'border-[#d7c49a] bg-[#fbf7eb] text-[#243858] hover:bg-[#f4ecd7]'"
            @click="statusFilter = 'active'"
          >
            Active
          </button>
          <button
            type="button"
            class="rounded-lg border px-4 py-2 text-sm font-semibold transition"
            :class="statusFilter === 'paused' ? 'border-[#243858] bg-[#243858] text-white' : 'border-[#d7c49a] bg-[#fbf7eb] text-[#243858] hover:bg-[#f4ecd7]'"
            @click="statusFilter = 'paused'"
          >
            On hold
          </button>
          <button
            type="button"
            class="rounded-lg border px-4 py-2 text-sm font-semibold transition"
            :class="statusFilter === 'completed' ? 'border-[#243858] bg-[#243858] text-white' : 'border-[#d7c49a] bg-[#fbf7eb] text-[#243858] hover:bg-[#f4ecd7]'"
            @click="statusFilter = 'completed'"
          >
            Completed
          </button>
        </div>
      </div>
    </section>

    <section class="ledger-panel overflow-hidden rounded-[18px]">
      <div class="overflow-x-auto">
        <table class="w-full min-w-[1100px] text-sm">
          <thead>
            <tr class="border-b ledger-rule bg-[#f3ebd7] text-[#3f5d89]">
              <th class="px-5 py-3 text-left font-medium ledger-eyebrow text-[11px]">Code</th>
              <th class="px-5 py-3 text-left font-medium ledger-eyebrow text-[11px]">Project</th>
              <th class="px-5 py-3 text-left font-medium ledger-eyebrow text-[11px]">Period</th>
              <th class="px-5 py-3 text-left font-medium ledger-eyebrow text-[11px]">Status</th>
              <th class="px-5 py-3 text-right font-medium ledger-eyebrow text-[11px]">Target</th>
              <th class="px-5 py-3 text-right font-medium ledger-eyebrow text-[11px]">Received</th>
              <th class="px-5 py-3 text-right font-medium ledger-eyebrow text-[11px]">Spent</th>
              <th class="px-5 py-3 text-right font-medium ledger-eyebrow text-[11px]">Balance</th>
              <th class="px-5 py-3 text-right font-medium ledger-eyebrow text-[11px]">Actions</th>
            </tr>
          </thead>

          <tbody>
            <tr v-if="loading" class="border-b border-[#eadfbd] bg-[#fbf7eb]">
              <td class="px-5 py-4 text-[#6a6b5d]" colspan="9">Loading...</td>
            </tr>

            <tr
              v-for="row in rows"
              :key="row.project.id"
              class="border-b border-[#eadfbd] bg-[#fbf7eb] transition hover:bg-[#f7f1e0]"
              @click="openProject(row.project.id)"
            >
              <td class="px-5 py-4 whitespace-nowrap font-mono text-[#1f3558]">
                {{ projectCode(row.project.id) }}
              </td>
              <td class="px-5 py-4">
                <div class="font-semibold text-[#1f3558]">{{ row.project.name }}</div>
                <div class="mt-1 max-w-[34rem] truncate text-xs text-[#6a6b5d]" :title="row.project.description || ''">
                  {{ row.project.description || "" }}
                </div>
              </td>
              <td class="px-5 py-4 whitespace-nowrap font-mono text-[#6a6b5d]">
                {{ periodLabel(row.project) }}
              </td>
              <td class="px-5 py-4">
                <span class="inline-flex rounded-full border px-3 py-1 text-xs font-semibold uppercase tracking-[0.2em]" :class="statusTone(row.project.status)">
                  {{ statusLabel(row.project.status) }}
                </span>
              </td>
              <td class="px-5 py-4 text-right font-mono text-[#1f3558]">
                {{ formatPHPFromCents(row.project.target_amount_cents) }}
              </td>
              <td class="px-5 py-4 text-right font-mono text-[var(--ledger-green)]">
                {{ formatPHPFromCents(row.received) }}
              </td>
              <td class="px-5 py-4 text-right font-mono text-[var(--ledger-red)]">
                {{ formatPHPFromCents(row.spent) }}
              </td>
              <td class="px-5 py-4 text-right font-mono text-[#1f3558]">
                {{ formatPHPFromCents(row.balanceCents) }}
              </td>
              <td class="px-5 py-4 text-right" @click.stop>
                <button
                  type="button"
                  class="rounded-lg border border-[#d7c49a] bg-[#fbf7eb] px-3 py-2 text-xs font-semibold text-[#243858] transition hover:bg-[#f4ecd7]"
                  @click="removeProject(row.project.id, row.project.name)"
                >
                  Delete
                </button>
              </td>
            </tr>

            <tr v-if="!loading && rows.length === 0" class="border-b border-[#eadfbd] bg-[#fbf7eb]">
              <td class="px-5 py-4 text-[#6a6b5d]" colspan="9">No projects found.</td>
            </tr>
          </tbody>

          <tfoot class="border-t-2 border-[#98a8be] bg-[#f5efdf]">
            <tr>
              <th class="px-5 py-4 text-left ledger-eyebrow text-[11px] text-[#3f5d89]" colspan="4">Totals</th>
              <th class="px-5 py-4 text-right font-mono font-semibold text-[#1f3558]">
                {{ formatPHPFromCents(totals.target) }}
              </th>
              <th class="px-5 py-4 text-right font-mono font-semibold text-[var(--ledger-green)]">
                {{ formatPHPFromCents(totals.received) }}
              </th>
              <th class="px-5 py-4 text-right font-mono font-semibold text-[var(--ledger-red)]">
                {{ formatPHPFromCents(totals.spent) }}
              </th>
              <th class="px-5 py-4 text-right font-mono font-semibold text-[#1f3558]">
                {{ formatPHPFromCents(totals.balance) }}
              </th>
              <th class="px-5 py-4"></th>
            </tr>
          </tfoot>
        </table>
      </div>
    </section>

    <section class="rounded-[16px] border border-[#d7c49a] bg-[#f6efdf] px-5 py-4 text-sm text-[#5f6d82]">
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div>
          Entries are recorded in Philippine Peso (PHP). All records stored locally.
        </div>
        <div>
          Last verified balance 27 Jul 2026.
        </div>
      </div>
    </section>

    <button
      type="button"
      class="fixed bottom-6 right-6 flex h-14 w-14 items-center justify-center rounded-2xl bg-[#243858] text-white shadow-2xl shadow-[#243858]/25 transition hover:bg-[#1d2c45]"
      title="Add project"
      @click="showAddProject = true"
    >
      <svg class="h-8 w-8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.25" aria-hidden="true">
        <path d="M12 5v14" />
        <path d="M5 12h14" />
      </svg>
    </button>

    <div v-if="showAddProject" class="fixed inset-0 z-50">
      <div class="absolute inset-0 bg-black/50" @click="showAddProject = false"></div>
      <div class="absolute inset-0 flex items-center justify-center p-4">
        <div class="w-full max-w-3xl rounded-[22px] border border-[#d7c49a] bg-[#fbf7eb] p-6 shadow-2xl">
          <div class="flex items-start justify-between gap-4">
            <div>
              <div class="ledger-heading text-3xl font-normal text-[#1f3558]">New Project</div>
              <div class="mt-1 text-sm text-[#6a6b5d]">Create a project with a target budget for tracking.</div>
            </div>
            <button class="rounded-lg border border-[#d7c49a] bg-white px-3 py-2 text-sm font-semibold text-[#243858] transition hover:bg-[#f4ecd7]" @click="showAddProject = false">
              Close
            </button>
          </div>

          <div class="mt-6 grid grid-cols-1 gap-3 md:grid-cols-6">
            <div class="md:col-span-2">
              <label class="mb-1 block text-xs uppercase tracking-[0.2em] text-[#6a6b5d]">Name</label>
              <input v-model="formName" placeholder="e.g. Instruments" class="w-full rounded-xl border border-[#d7c49a] bg-white px-3 py-2 text-[#243858] outline-none" />
            </div>
            <div class="md:col-span-1">
              <label class="mb-1 block text-xs uppercase tracking-[0.2em] text-[#6a6b5d]">Target (PHP)</label>
              <input v-model="formTarget" inputmode="decimal" placeholder="0.00" class="w-full rounded-xl border border-[#d7c49a] bg-white px-3 py-2 text-[#243858] outline-none" />
            </div>
            <div class="md:col-span-1">
              <label class="mb-1 block text-xs uppercase tracking-[0.2em] text-[#6a6b5d]">Status</label>
              <select v-model="formStatus" class="w-full rounded-xl border border-[#d7c49a] bg-white px-3 py-2 text-[#243858] outline-none">
                <option value="active">active</option>
                <option value="paused">on hold</option>
                <option value="completed">completed</option>
              </select>
            </div>
            <div class="md:col-span-1">
              <label class="mb-1 block text-xs uppercase tracking-[0.2em] text-[#6a6b5d]">Start</label>
              <input
                v-model="formStart"
                type="date"
                :max="startDateMax"
                class="w-full rounded-xl border border-[#d7c49a] bg-white px-3 py-2 text-[#243858] outline-none"
              />
            </div>
            <div class="md:col-span-1">
              <label class="mb-1 block text-xs uppercase tracking-[0.2em] text-[#6a6b5d]">End</label>
              <input
                v-model="formEnd"
                type="date"
                :min="endDateMin"
                class="w-full rounded-xl border border-[#d7c49a] bg-white px-3 py-2 text-[#243858] outline-none"
              />
            </div>
            <div class="md:col-span-6">
              <label class="mb-1 block text-xs uppercase tracking-[0.2em] text-[#6a6b5d]">Description</label>
              <input v-model="formDescription" placeholder="optional" class="w-full rounded-xl border border-[#d7c49a] bg-white px-3 py-2 text-[#243858] outline-none" />
            </div>
          </div>

          <div class="mt-6 flex items-center justify-end gap-2">
            <button class="rounded-xl border border-[#d7c49a] bg-white px-4 py-2 font-semibold text-[#243858] transition hover:bg-[#f4ecd7]" @click="showAddProject = false">
              Cancel
            </button>
            <button class="rounded-xl bg-[#243858] px-4 py-2 font-semibold text-white transition hover:bg-[#1f2f4a]" @click="createProject">
              Save Project
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
