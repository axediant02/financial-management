<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { backupCreate } from "../lib/api";
import OverviewDashboard from "./views/OverviewDashboard.vue";
import DonationsView from "./views/DonationsView.vue";
import ExpensesView from "./views/ExpensesView.vue";
import ProjectsView from "./views/ProjectsView.vue";
import ReportsView from "./views/ReportsView.vue";
import BackupsView from "./views/BackupsView.vue";
import ProjectDetailView from "./views/ProjectDetailView.vue";

const props = defineProps<{ sessionToken: string }>();
const emit = defineEmits<{ (e: "logout"): void }>();

type Tab = "overview" | "donations" | "expenses" | "projects" | "reports" | "backups";
type DetailTab = Tab | "project_detail";
const selectedProjectId = ref<number | null>(null);
const NAV_TAB_KEY = "pft_nav_tab";
const NAV_PROJECT_KEY = "pft_nav_project_id";

function isDetailTab(value: string): value is DetailTab {
  return (
    value === "overview" ||
    value === "donations" ||
    value === "expenses" ||
    value === "projects" ||
    value === "reports" ||
    value === "backups" ||
    value === "project_detail"
  );
}

const tab = ref<DetailTab>("overview");
const title = computed(() => {
  switch (tab.value) {
    case "overview":
      return "Overview";
    case "donations":
      return "Contributions";
    case "expenses":
      return "Expenses";
    case "projects":
      return "Projects";
    case "reports":
      return "Reports";
    case "backups":
      return "Backups";
    case "project_detail":
      return "Project";
  }
});

const toast = ref<string | null>(null);
let toastTimeout: any = null;
function showToast(message: string) {
  toast.value = message;
  clearTimeout(toastTimeout);
  toastTimeout = setTimeout(() => (toast.value = null), 2500);
}

onMounted(() => {
  const savedTab = localStorage.getItem(NAV_TAB_KEY);
  const savedProjectId = localStorage.getItem(NAV_PROJECT_KEY);
  if (savedTab && isDetailTab(savedTab)) {
    tab.value = savedTab;
  }
  if (savedProjectId && /^\d+$/.test(savedProjectId)) {
    selectedProjectId.value = Number(savedProjectId);
  }
  if (tab.value === "project_detail" && selectedProjectId.value == null) {
    tab.value = "projects";
  }
  showToast("Unlocked");
});

watch(
  () => tab.value,
  (value) => {
    localStorage.setItem(NAV_TAB_KEY, value);
    if (value !== "project_detail") {
      localStorage.removeItem(NAV_PROJECT_KEY);
    }
  },
);

watch(
  () => selectedProjectId.value,
  (value) => {
    if (value == null) {
      localStorage.removeItem(NAV_PROJECT_KEY);
    } else {
      localStorage.setItem(NAV_PROJECT_KEY, String(value));
    }
  },
);

async function quickBackup() {
  const path = await backupCreate(props.sessionToken);
  showToast(`Backup created: ${path}`);
}
</script>

<template>
  <div class="min-h-screen flex">
    <aside class="w-64 shrink-0 border-r border-slate-800 bg-slate-950/40 p-4">
      <div class="rounded-xl border border-slate-800 bg-slate-900/40 p-4">
        <div class="text-sm text-slate-400">Project Funds Tracker</div>
        <div class="mt-1 font-semibold">Church Ledger</div>
        <div class="mt-3 flex gap-2">
          <button
            class="flex-1 rounded-lg bg-slate-800 hover:bg-slate-700 px-3 py-2 text-xs font-semibold"
            @click="quickBackup"
          >
            Backup
          </button>
          <button
            class="flex-1 rounded-lg bg-rose-600 hover:bg-rose-500 px-3 py-2 text-xs font-semibold"
            @click="emit('logout')"
          >
            Lock
          </button>
        </div>
      </div>

      <nav class="mt-4 space-y-1">
        <button
          class="w-full text-left rounded-xl px-3 py-2 text-sm hover:bg-slate-900"
          :class="tab === 'overview' ? 'bg-slate-900 text-white' : 'text-slate-300'"
          @click="tab = 'overview'"
        >
          Overview
        </button>
        <button
          class="w-full text-left rounded-xl px-3 py-2 text-sm hover:bg-slate-900"
          :class="tab === 'projects' ? 'bg-slate-900 text-white' : 'text-slate-300'"
          @click="tab = 'projects'"
        >
          Projects
        </button>
        <button
          class="w-full text-left rounded-xl px-3 py-2 text-sm hover:bg-slate-900"
          :class="tab === 'expenses' ? 'bg-slate-900 text-white' : 'text-slate-300'"
          @click="tab = 'expenses'"
        >
          Expenses
        </button>
        <button
          class="w-full text-left rounded-xl px-3 py-2 text-sm hover:bg-slate-900"
          :class="tab === 'donations' ? 'bg-slate-900 text-white' : 'text-slate-300'"
          @click="tab = 'donations'"
        >
          Contributions
        </button>
        <button
          class="w-full text-left rounded-xl px-3 py-2 text-sm hover:bg-slate-900"
          :class="tab === 'reports' ? 'bg-slate-900 text-white' : 'text-slate-300'"
          @click="tab = 'reports'"
        >
          Reports
        </button>
        <button
          class="w-full text-left rounded-xl px-3 py-2 text-sm hover:bg-slate-900"
          :class="tab === 'backups' ? 'bg-slate-900 text-white' : 'text-slate-300'"
          @click="tab = 'backups'"
        >
          Backups
        </button>
      </nav>
    </aside>

    <main class="flex-1 p-6">
      <header class="flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-bold">{{ title }}</h1>
          <p class="text-sm text-slate-400">Currency: PHP • Offline-only</p>
        </div>
      </header>

      <div class="mt-6">
        <OverviewDashboard
          v-if="tab === 'overview'"
          :session-token="sessionToken"
          @create-project="tab = 'projects'"
          @open-project="(id: number) => { selectedProjectId = id; tab = 'project_detail'; }"
        />
        <ProjectsView
          v-else-if="tab === 'projects'"
          :session-token="sessionToken"
          @open-project="(id: number) => { selectedProjectId = id; tab = 'project_detail'; }"
        />
        <ExpensesView v-else-if="tab === 'expenses'" :session-token="sessionToken" />
        <DonationsView v-else-if="tab === 'donations'" :session-token="sessionToken" />
        <ReportsView v-else-if="tab === 'reports'" :session-token="sessionToken" />
        <BackupsView v-else-if="tab === 'backups'" :session-token="sessionToken" />
        <ProjectDetailView
          v-else-if="tab === 'project_detail' && selectedProjectId != null"
          :session-token="sessionToken"
          :project-id="selectedProjectId"
          @back="tab = 'projects'"
        />
      </div>
    </main>

    <div
      v-if="toast"
      class="fixed bottom-5 right-5 rounded-xl border border-slate-700 bg-slate-900/90 px-4 py-3 text-sm text-slate-200 shadow-2xl"
    >
      {{ toast }}
    </div>
  </div>
</template>
