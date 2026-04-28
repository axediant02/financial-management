<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { appStatus, bootstrapAdmin, login, logout } from "./lib/api";
import SetupAdmin from "./components/SetupAdmin.vue";
import LoginPage from "./components/LoginPage.vue";
import MainApp from "./components/MainApp.vue";

const loading = ref(true);
const hasAdmin = ref(false);
const sessionToken = ref<string | null>(localStorage.getItem("pft_session_token"));
const dbPath = ref<string | null>(null);
const appDataDir = ref<string | null>(null);
const errorMessage = ref<string | null>(null);

const isAuthed = computed(() => !!sessionToken.value);

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
}

async function handleLogin(password: string) {
  errorMessage.value = null;
  const res = await login(password);
  sessionToken.value = res.session_token;
  localStorage.setItem("pft_session_token", res.session_token);
}

async function handleLogout() {
  errorMessage.value = null;
  const token = sessionToken.value;
  sessionToken.value = null;
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
}

onMounted(async () => {
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
  <div class="min-h-screen bg-slate-950 text-slate-100">
    <div v-if="loading" class="min-h-screen flex items-center justify-center">
      <div class="text-slate-300">Loading…</div>
    </div>

    <div v-else>
      <div v-if="errorMessage" class="max-w-3xl mx-auto p-6">
        <div class="rounded-xl border border-rose-500/40 bg-rose-500/10 p-4 text-rose-200">
          {{ errorMessage }}
        </div>
      </div>

      <SetupAdmin
        v-if="!hasAdmin"
        :db-path="dbPath"
        :app-data-dir="appDataDir"
        @bootstrap="handleBootstrap"
      />

      <LoginPage v-else-if="!isAuthed" @login="handleLogin" />

      <MainApp v-else :session-token="sessionToken!" @logout="handleLogout" />
    </div>
  </div>
</template>
