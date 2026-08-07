<script setup lang="ts">
import { ref } from "vue";

defineProps<{
  dbPath: string | null;
  appDataDir: string | null;
}>();

const emit = defineEmits<{
  (e: "bootstrap", password: string): void;
}>();

const password = ref("");
const confirm = ref("");
const submitting = ref(false);
const errorMessage = ref<string | null>(null);

async function submit() {
  errorMessage.value = null;
  if (password.value.trim().length < 8) {
    errorMessage.value = "Password must be at least 8 characters.";
    return;
  }
  if (password.value !== confirm.value) {
    errorMessage.value = "Passwords do not match.";
    return;
  }
  submitting.value = true;
  try {
    emit("bootstrap", password.value);
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center px-4">
    <div class="w-full max-w-lg">
      <div class="rounded-2xl border border-slate-800 bg-slate-900/60 p-8 shadow-2xl">
        <h1 class="text-2xl font-bold">Project Funds Tracker</h1>
        <p class="mt-2 text-slate-300 text-sm">
          First run setup. Create the admin password for this device.
        </p>

        <div class="mt-5 rounded-xl bg-slate-950/40 border border-slate-800 p-4 text-xs text-slate-300">
          <div class="flex items-center justify-between gap-3">
            <span class="text-slate-400">Database</span>
            <span class="truncate">{{ dbPath || "-" }}</span>
          </div>
          <div class="mt-2 flex items-center justify-between gap-3">
            <span class="text-slate-400">App data</span>
            <span class="truncate">{{ appDataDir || "-" }}</span>
          </div>
        </div>

        <form class="mt-6 space-y-4" @submit.prevent="submit">
          <div>
            <label class="block text-sm text-slate-200 mb-1">Admin password</label>
            <input
              v-model="password"
              type="password"
              class="w-full rounded-xl border border-slate-700 bg-slate-950/60 px-4 py-3 text-slate-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
              placeholder="At least 8 characters"
              autocomplete="new-password"
            />
          </div>
          <div>
            <label class="block text-sm text-slate-200 mb-1">Confirm password</label>
            <input
              v-model="confirm"
              type="password"
              class="w-full rounded-xl border border-slate-700 bg-slate-950/60 px-4 py-3 text-slate-100 focus:outline-none focus:ring-2 focus:ring-indigo-500"
              placeholder="Repeat password"
              autocomplete="new-password"
            />
          </div>

          <div
            v-if="errorMessage"
            class="rounded-xl border border-rose-500/40 bg-rose-500/10 p-3 text-rose-200 text-sm"
          >
            {{ errorMessage }}
          </div>

          <button
            type="submit"
            :disabled="submitting"
            class="w-full rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-60 px-4 py-3 font-semibold"
          >
            {{ submitting ? "Creating…" : "Create Admin Password" }}
          </button>
        </form>
      </div>
      <p class="mt-4 text-center text-xs text-slate-500">
        Currency: PHP
      </p>
    </div>
  </div>
</template>

