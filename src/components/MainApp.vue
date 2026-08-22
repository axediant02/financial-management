<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { backupCreate } from "../lib/api";
import { notify } from "../lib/feedback";
import OverviewDashboard from "./views/OverviewDashboard.vue";
import DonationsView from "./views/DonationsView.vue";
import ExpensesView from "./views/ExpensesView.vue";
import ProjectsView from "./views/ProjectsView.vue";
import DocumentationView from "./views/DocumentationView.vue";
import ReportsView from "./views/ReportsView.vue";
import BackupsView from "./views/BackupsView.vue";
import ProjectDetailView from "./views/ProjectDetailView.vue";

const props = defineProps<{ sessionToken: string; themeMode: "light" | "dark" }>();
const emit = defineEmits<{ (e: "logout"): void }>();

type Tab = "overview" | "donations" | "expenses" | "projects" | "documentation" | "reports" | "backups";
type DetailTab = Tab | "project_detail";

const NAV_TAB_KEY = "pft_nav_tab";
const NAV_PROJECT_KEY = "pft_nav_project_id";
const NAV_PREV_TAB_KEY = "pft_nav_prev_tab";
const SIDEBAR_KEY = "pft_sidebar_collapsed";
const selectedProjectId = ref<number | null>(null);
const tab = ref<DetailTab>("overview");
const previousTab = ref<Exclude<DetailTab, "project_detail">>("overview");
const sidebarCollapsed = ref(localStorage.getItem(SIDEBAR_KEY) === "true");
const showCreateRecordChooser = ref(false);

function isDetailTab(value: string): value is DetailTab {
  return (
    value === "overview" ||
    value === "donations" ||
    value === "expenses" ||
    value === "projects" ||
    value === "documentation" ||
    value === "reports" ||
    value === "backups" ||
    value === "project_detail"
  );
}

function isTab(value: string): value is Tab {
  return (
    value === "overview" ||
    value === "donations" ||
    value === "expenses" ||
    value === "projects" ||
    value === "documentation" ||
    value === "reports" ||
    value === "backups"
  );
}

onMounted(() => {
  const savedTab = localStorage.getItem(NAV_TAB_KEY);
  const savedProjectId = localStorage.getItem(NAV_PROJECT_KEY);
  const savedPrevTab = localStorage.getItem(NAV_PREV_TAB_KEY);
  if (savedTab && isDetailTab(savedTab)) {
    tab.value = savedTab;
  }
  if (savedProjectId && /^\d+$/.test(savedProjectId)) {
    selectedProjectId.value = Number(savedProjectId);
  }
  if (savedPrevTab && isTab(savedPrevTab)) {
    previousTab.value = savedPrevTab;
  }
  if (tab.value === "project_detail" && selectedProjectId.value == null) {
    tab.value = "projects";
  }
  notify("Unlocked");
});

watch(
  () => tab.value,
  (value) => {
    localStorage.setItem(NAV_TAB_KEY, value);
    if (value !== "project_detail") {
      localStorage.removeItem(NAV_PROJECT_KEY);
      localStorage.removeItem(NAV_PREV_TAB_KEY);
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

watch(
  () => sidebarCollapsed.value,
  (value) => {
    localStorage.setItem(SIDEBAR_KEY, String(value));
  },
);

async function quickBackup() {
  const path = await backupCreate(props.sessionToken);
  notify(`Backup created: ${path}`);
}

function goToOverview() {
  tab.value = "overview";
}

function openProjects() {
  tab.value = "projects";
}

function openCreateRecord() {
  showCreateRecordChooser.value = true;
}

function openDocumentation() {
  tab.value = "documentation";
}

function openDonations() {
  tab.value = "donations";
}

function openExpenses() {
  tab.value = "expenses";
}

function closeCreateRecordChooser() {
  showCreateRecordChooser.value = false;
}

function chooseCreateProject() {
  showCreateRecordChooser.value = false;
  tab.value = "projects";
}

function chooseCreateDocumentation() {
  showCreateRecordChooser.value = false;
  tab.value = "documentation";
}

function openProjectDetail(id: number) {
  if (tab.value !== "project_detail") {
    previousTab.value = tab.value as Exclude<DetailTab, "project_detail">;
    localStorage.setItem(NAV_PREV_TAB_KEY, previousTab.value);
  }
  selectedProjectId.value = id;
  tab.value = "project_detail";
}

function backToPreviousTab() {
  tab.value = previousTab.value;
}

function detailBackLabel() {
  switch (previousTab.value) {
    case "overview":
      return "Dashboard";
    case "donations":
      return "Contributions";
    case "expenses":
      return "Expenses";
    case "projects":
      return "Projects";
    case "documentation":
      return "Documentation";
    case "reports":
      return "Reports";
    case "backups":
      return "Backups";
    default:
      return "Previous page";
  }
}

function toggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value;
}
</script>

<template>
  <div class="h-full overflow-hidden ledger-shell">
    <div class="flex h-full overflow-hidden">
      <aside
        class="ledger-sidebar flex shrink-0 flex-col border-r border-[#314868] py-4 text-white transition-[width,padding] duration-200"
        :class="sidebarCollapsed ? 'w-[88px] px-2' : 'w-[262px] px-3'"
      >
        <div class="rounded-[22px] border border-[#5f7190] bg-[rgba(26,41,69,0.92)] p-4 shadow-[0_18px_35px_rgba(0,0,0,0.15)]">
          <div class="flex items-start gap-3" :class="sidebarCollapsed ? 'flex-col items-center gap-2' : ''">
            <div class="flex h-10 w-10 items-center justify-center rounded-lg border border-[#c8a85d] text-[#e7d39a]">
              <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
                <path d="M12 3l7 3v6c0 4.5-3.1 8.7-7 9-3.9-.3-7-4.5-7-9V6l7-3Z" />
              </svg>
            </div>
            <div v-if="!sidebarCollapsed">
              <div class="text-lg font-semibold leading-tight text-white">Church Ledger</div>
              <div class="mt-1 text-[11px] uppercase tracking-[0.34em] text-[#c7d0e2]">
                Project Funds Tracker
              </div>
            </div>
          </div>

          <button
            type="button"
            class="mt-4 flex w-full items-center justify-center rounded-lg border border-[#5f7190] bg-[#203355] px-3 py-2 text-xs font-semibold text-white transition hover:bg-[#284066]"
            @click="toggleSidebar"
          >
            {{ sidebarCollapsed ? "Expand" : "Collapse" }}
          </button>
        </div>

        <nav class="mt-4 space-y-2">
          <button
            class="flex w-full items-center rounded-[10px] px-4 py-3 text-left text-sm font-semibold transition"
            :class="[
              tab === 'overview' ? 'bg-[#31476a] text-white shadow-inner' : 'text-[#d8e0ec] hover:bg-white/5',
              sidebarCollapsed ? 'justify-center gap-0' : 'gap-3',
            ]"
            @click="goToOverview"
            :title="sidebarCollapsed ? 'Dashboard' : undefined"
          >
            <span class="flex h-5 w-5 items-center justify-center">
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M3 12l9-8 9 8" />
                <path d="M5 10v10h14V10" />
              </svg>
            </span>
            <span v-if="!sidebarCollapsed">Dashboard</span>
          </button>
          <button
            class="flex w-full items-center rounded-[10px] px-4 py-3 text-left text-sm font-semibold transition"
            :class="[
              tab === 'projects' ? 'bg-[#31476a] text-white shadow-inner' : 'text-[#d8e0ec] hover:bg-white/5',
              sidebarCollapsed ? 'justify-center gap-0' : 'gap-3',
            ]"
            @click="openProjects"
            :title="sidebarCollapsed ? 'Projects' : undefined"
          >
            <span class="flex h-5 w-5 items-center justify-center">
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M4 7h16v10H4z" />
                <path d="M8 7V5h8v2" />
              </svg>
            </span>
            <span v-if="!sidebarCollapsed">Projects</span>
          </button>
          <button
            class="flex w-full items-center rounded-[10px] px-4 py-3 text-left text-sm font-semibold transition"
            :class="[
              tab === 'documentation' ? 'bg-[#31476a] text-white shadow-inner' : 'text-[#d8e0ec] hover:bg-white/5',
              sidebarCollapsed ? 'justify-center gap-0' : 'gap-3',
            ]"
            @click="openDocumentation"
            :title="sidebarCollapsed ? 'Documentation' : undefined"
          >
            <span class="flex h-5 w-5 items-center justify-center">
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M6 4h9l3 3v13H6z" />
                <path d="M15 4v4h4" />
                <path d="M8 12h8" />
                <path d="M8 16h8" />
              </svg>
            </span>
            <span v-if="!sidebarCollapsed">Documentation</span>
          </button>
          <button
            class="flex w-full items-center rounded-[10px] px-4 py-3 text-left text-sm font-semibold transition"
            :class="[
              tab === 'donations' ? 'bg-[#31476a] text-white shadow-inner' : 'text-[#d8e0ec] hover:bg-white/5',
              sidebarCollapsed ? 'justify-center gap-0' : 'gap-3',
            ]"
            @click="openDonations"
            :title="sidebarCollapsed ? 'Contributions' : undefined"
          >
            <span class="flex h-5 w-5 items-center justify-center">
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M12 5v14" />
                <path d="M7 12h10" />
              </svg>
            </span>
            <span v-if="!sidebarCollapsed">Contributions</span>
          </button>
          <button
            class="flex w-full items-center rounded-[10px] px-4 py-3 text-left text-sm font-semibold transition"
            :class="[
              tab === 'expenses' ? 'bg-[#31476a] text-white shadow-inner' : 'text-[#d8e0ec] hover:bg-white/5',
              sidebarCollapsed ? 'justify-center gap-0' : 'gap-3',
            ]"
            @click="openExpenses"
            :title="sidebarCollapsed ? 'Expenses' : undefined"
          >
            <span class="flex h-5 w-5 items-center justify-center">
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M5 5h14v14H5z" />
                <path d="M8 9h8" />
                <path d="M8 13h8" />
              </svg>
            </span>
            <span v-if="!sidebarCollapsed">Expenses</span>
          </button>
          <button
            class="flex w-full items-center rounded-[10px] px-4 py-3 text-left text-sm font-semibold transition"
            :class="[
              tab === 'reports' ? 'bg-[#31476a] text-white shadow-inner' : 'text-[#d8e0ec] hover:bg-white/5',
              sidebarCollapsed ? 'justify-center gap-0' : 'gap-3',
            ]"
            @click="tab = 'reports'"
            :title="sidebarCollapsed ? 'Reports' : undefined"
          >
            <span class="flex h-5 w-5 items-center justify-center">
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M6 20h12" />
                <path d="M8 17V8" />
                <path d="M12 17V4" />
                <path d="M16 17v-6" />
              </svg>
            </span>
            <span v-if="!sidebarCollapsed">Reports</span>
          </button>
          <button
            class="flex w-full items-center rounded-[10px] px-4 py-3 text-left text-sm font-semibold transition"
            :class="[
              tab === 'backups' ? 'bg-[#31476a] text-white shadow-inner' : 'text-[#d8e0ec] hover:bg-white/5',
              sidebarCollapsed ? 'justify-center gap-0' : 'gap-3',
            ]"
            @click="tab = 'backups'"
            :title="sidebarCollapsed ? 'Backups' : undefined"
          >
            <span class="flex h-5 w-5 items-center justify-center">
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M12 3v4" />
                <path d="M9 7h6" />
                <path d="M5 11h14v8H5z" />
              </svg>
            </span>
            <span v-if="!sidebarCollapsed">Backups</span>
          </button>
        </nav>

        <div class="mt-auto border-t border-[#334b6d] pt-4 text-sm" :class="sidebarCollapsed ? 'text-center' : ''">
          <div class="space-y-3 rounded-[18px] border border-[#334b6d] bg-[rgba(18,31,53,0.9)] p-4">
            <div class="flex items-center gap-2 text-[#d8e0ec]" :class="sidebarCollapsed ? 'justify-center' : ''">
              <svg class="h-4 w-4 text-[#a8b6ca]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M6 10V8a6 6 0 1 1 12 0v2" />
                <rect x="5" y="10" width="14" height="10" rx="2" />
              </svg>
              <span v-if="!sidebarCollapsed">Admin session locked</span>
            </div>
            <button
              class="flex items-center gap-2 text-left text-[#e9d59b] underline decoration-[#e9d59b]/50 underline-offset-4"
              :class="sidebarCollapsed ? 'justify-center' : ''"
              @click="emit('logout')"
              :title="sidebarCollapsed ? 'Lock the ledger' : undefined"
            >
              <svg class="h-4 w-4 text-[#e9d59b]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M6 10V8a6 6 0 1 1 12 0v2" />
                <rect x="5" y="10" width="14" height="10" rx="2" />
              </svg>
              <span v-if="!sidebarCollapsed">Lock the ledger</span>
            </button>
            <button
              type="button"
              class="flex items-center gap-2 rounded-lg border border-[#5f7190] bg-[#203355] px-3 py-2 text-xs font-semibold text-white transition hover:bg-[#284066]"
              :class="sidebarCollapsed ? 'justify-center' : ''"
              @click="quickBackup"
              :title="sidebarCollapsed ? 'Backup now' : undefined"
            >
              <svg class="h-4 w-4 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M12 5v10" />
                <path d="M8 11l4 4 4-4" />
                <path d="M5 19h14" />
              </svg>
              <span v-if="!sidebarCollapsed">Backup now</span>
            </button>
          </div>
        </div>
      </aside>

      <main class="min-w-0 flex-1 overflow-y-auto px-4 py-4 md:px-5 md:py-5">
        <OverviewDashboard
          v-if="tab === 'overview'"
          :session-token="sessionToken"
          @create-project="tab = 'projects'"
          @open-project="openProjectDetail"
          @open-projects="openProjects"
          @open-create-record="openCreateRecord"
          @open-expenses="openExpenses"
        />
          <ProjectsView
          v-else-if="tab === 'projects'"
          :session-token="sessionToken"
          @open-project="openProjectDetail"
          @open-documentation="openDocumentation"
        />
        <DocumentationView
          v-else-if="tab === 'documentation'"
          :session-token="sessionToken"
        />
        <ExpensesView v-else-if="tab === 'expenses'" :session-token="sessionToken" />
        <DonationsView v-else-if="tab === 'donations'" :session-token="sessionToken" />
        <ReportsView v-else-if="tab === 'reports'" :session-token="sessionToken" />
        <BackupsView v-else-if="tab === 'backups'" :session-token="sessionToken" />
        <ProjectDetailView
          v-else-if="tab === 'project_detail' && selectedProjectId != null"
          :session-token="sessionToken"
          :project-id="selectedProjectId"
          :theme-mode="themeMode"
          :back-label="detailBackLabel()"
          @back="backToPreviousTab"
        />
      </main>
    </div>

    <div v-if="showCreateRecordChooser" class="fixed inset-0 z-50">
      <div class="absolute inset-0 bg-black/50" @click="closeCreateRecordChooser"></div>
      <div class="absolute inset-0 flex items-center justify-center p-4">
        <div class="w-full max-w-2xl rounded-[22px] border border-[#d7c49a] bg-[#fbf7eb] p-6 shadow-2xl">
          <div class="flex items-start justify-between gap-4">
            <div>
              <div class="ledger-heading text-3xl font-normal text-[#1f3558]">New Record</div>
              <div class="mt-1 text-sm text-[#6a6b5d]">Choose whether to create a project or documentation record.</div>
            </div>
            <button
              class="rounded-lg border border-[#d7c49a] bg-white px-3 py-2 text-sm font-semibold text-[#243858] transition hover:bg-[#f4ecd7]"
              @click="closeCreateRecordChooser"
            >
              Close
            </button>
          </div>

          <div class="mt-6 grid gap-3 md:grid-cols-2">
            <button
              type="button"
              class="rounded-[18px] border border-[#d7c49a] bg-white p-5 text-left transition hover:bg-[#f4ecd7]"
              @click="chooseCreateProject"
            >
              <div class="text-lg font-semibold text-[#1f3558]">Project</div>
              <p class="mt-2 text-sm leading-6 text-[#6a6b5d]">
                Create a fund drive with a target budget, status, and date range.
              </p>
            </button>
            <button
              type="button"
              class="rounded-[18px] border border-[#d7c49a] bg-white p-5 text-left transition hover:bg-[#f4ecd7]"
              @click="chooseCreateDocumentation"
            >
              <div class="text-lg font-semibold text-[#1f3558]">Documentation</div>
              <p class="mt-2 text-sm leading-6 text-[#6a6b5d]">
                Record post-event registration funds with the event name and collected amount.
              </p>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
