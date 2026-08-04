<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { Moon, Sun } from "lucide-vue-next";
import { appStatus, bootstrapAdmin, login, logout } from "./lib/api";
import { notify } from "./lib/feedback";
import SetupAdmin from "./components/SetupAdmin.vue";
import LoginPage from "./components/LoginPage.vue";
import ForgotPasswordPage from "./components/ForgotPasswordPage.vue";
import MainApp from "./components/MainApp.vue";

const THEME_KEY = "pft_theme_mode";

const loading = ref(true);
const hasAdmin = ref(false);
const sessionToken = ref<string | null>(localStorage.getItem("pft_session_token"));
const dbPath = ref<string | null>(null);
const appDataDir = ref<string | null>(null);
const errorMessage = ref<string | null>(null);
const toast = ref<string | null>(null);
let toastTimeout: ReturnType<typeof setTimeout> | null = null;
const themeMode = ref<"light" | "dark">(localStorage.getItem(THEME_KEY) === "dark" ? "dark" : "light");
const authScreen = ref<"login" | "forgot">("login");

const isAuthed = computed(() => !!sessionToken.value);

function showToast(message: string) {
  toast.value = message;
  if (toastTimeout) clearTimeout(toastTimeout);
  toastTimeout = setTimeout(() => {
    toast.value = null;
  }, 2500);
}

function applyTheme() {
  document.documentElement.dataset.theme = themeMode.value;
  localStorage.setItem(THEME_KEY, themeMode.value);
}

function toggleTheme() {
  themeMode.value = themeMode.value === "light" ? "dark" : "light";
}

async function refreshStatus() {
  const status = await appStatus();
  hasAdmin.value = status.has_admin;
  dbPath.value = status.db_path;
  appDataDir.value = status.app_data_dir;
}

async function handleBootstrap(password: string) {
  errorMessage.value = null;
  await bootstrapAdmin(password);
  const res = await login(password);
  sessionToken.value = res.session_token;
  localStorage.setItem("pft_session_token", res.session_token);
  await refreshStatus();
  notify("Admin password created.");
}

function handleLoginSuccess(token: string) {
  sessionToken.value = token;
  localStorage.setItem("pft_session_token", token);
  notify("Logged in.");
}

function handleOpenForgotPassword() {
  authScreen.value = "forgot";
}

function handleBackToLogin() {
  authScreen.value = "login";
}

function handlePasswordReplaced() {
  authScreen.value = "login";
  notify("Password replaced. Sign in with the new password.");
}

async function handleLogout() {
  errorMessage.value = null;
  const token = sessionToken.value;
  sessionToken.value = null;
  authScreen.value = "login";
  localStorage.removeItem("pft_session_token");
  localStorage.removeItem("pft_nav_tab");
  localStorage.removeItem("pft_nav_project_id");
  if (token) {
    try {
      await logout(token);
    } catch {
      // ignore
    }
  }
  notify("Logged out.");
}

watch(themeMode, () => {
  applyTheme();
  window.dispatchEvent(new CustomEvent("pft:theme-change", { detail: themeMode.value }));
});

onMounted(async () => {
  applyTheme();
  window.addEventListener("pft:toast", (event: Event) => {
    const detail = (event as CustomEvent).detail;
    if (typeof detail === "string" && detail.trim()) {
      showToast(detail);
    }
  });
  window.addEventListener("pft:theme-change", (event: Event) => {
    const detail = (event as CustomEvent).detail;
    if (detail === "light" || detail === "dark") {
      themeMode.value = detail;
    }
  });
  window.addEventListener("pft:unauthorized", () => {
    errorMessage.value = "Session expired. Please login again.";
    void handleLogout();
  });
  try {
    await refreshStatus();
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div :class="themeMode === 'dark' ? 'h-full overflow-hidden bg-slate-950 text-slate-100' : 'h-full overflow-hidden bg-slate-50 text-slate-900'">
    <button
      type="button"
      class="fixed right-4 top-4 z-50 inline-flex h-10 w-10 items-center justify-center rounded-[2px] border border-slate-300 bg-white text-slate-900 shadow-sm transition hover:bg-slate-50"
      :aria-label="themeMode === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'"
      :title="themeMode === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'"
      @click="toggleTheme"
    >
      <Sun v-if="themeMode === 'dark'" class="h-4 w-4" />
      <Moon v-else class="h-4 w-4" />
    </button>

    <div
      v-if="toast"
      class="fixed bottom-5 right-5 z-50 rounded-[2px] border px-4 py-3 text-sm shadow-2xl"
      :class="themeMode === 'dark'
        ? 'border-slate-700 bg-slate-900 text-slate-100'
        : 'border-slate-300 bg-white text-slate-900'"
    >
      {{ toast }}
    </div>

    <div v-if="loading" class="h-full flex items-center justify-center">
      <div :class="themeMode === 'dark' ? 'text-slate-300' : 'text-slate-600'">Loading...</div>
    </div>

    <div v-else class="h-full">
      <div v-if="errorMessage" class="max-w-3xl mx-auto p-6">
        <div :class="themeMode === 'dark' ? 'rounded-xl border border-rose-500/40 bg-rose-500/10 p-4 text-rose-200' : 'rounded-xl border border-rose-200 bg-rose-50 p-4 text-rose-700'">
          {{ errorMessage }}
        </div>
      </div>

      <SetupAdmin
        v-if="!hasAdmin"
        :db-path="dbPath"
        :app-data-dir="appDataDir"
        @bootstrap="handleBootstrap"
      />

      <LoginPage
        v-else-if="!isAuthed && authScreen === 'login'"
        @login-success="handleLoginSuccess"
        @forgot-password="handleOpenForgotPassword"
      />

      <ForgotPasswordPage
        v-else-if="!isAuthed && authScreen === 'forgot'"
        @back-to-login="handleBackToLogin"
        @password-replaced="handlePasswordReplaced"
      />

      <MainApp v-else :session-token="sessionToken!" :theme-mode="themeMode" @logout="handleLogout" />
    </div>
  </div>
</template>
