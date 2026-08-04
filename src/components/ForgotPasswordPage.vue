<script setup lang="ts">
import { ref } from "vue";
import { completeAdminPasswordReplace, requestAdminPasswordReplace } from "../lib/api";

const emit = defineEmits<{
  (e: "back-to-login"): void;
  (e: "password-replaced"): void;
}>();

const replaceCode = ref("");
const challengeCode = ref("");
const challengeExpiresAt = ref("");
const newPassword = ref("");
const confirmPassword = ref("");
const requestingCode = ref(false);
const submitting = ref(false);
const errorMessage = ref<string | null>(null);
const successMessage = ref<string | null>(null);

async function handleRequestCode() {
  errorMessage.value = null;
  successMessage.value = null;
  requestingCode.value = true;
  try {
    const challenge = await requestAdminPasswordReplace();
    challengeCode.value = challenge.code;
    challengeExpiresAt.value = challenge.expires_at;
    replaceCode.value = challenge.code;
    successMessage.value = "Replacement code created and saved in the database.";
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    requestingCode.value = false;
  }
}

async function handleSubmit() {
  errorMessage.value = null;
  successMessage.value = null;

  if (!replaceCode.value.trim()) {
    errorMessage.value = "Enter the replacement code.";
    return;
  }

  if (newPassword.value.trim().length < 8) {
    errorMessage.value = "New password must be at least 8 characters.";
    return;
  }

  if (newPassword.value !== confirmPassword.value) {
    errorMessage.value = "Passwords do not match.";
    return;
  }

  submitting.value = true;
  try {
    await completeAdminPasswordReplace(replaceCode.value.trim(), newPassword.value);
    replaceCode.value = "";
    challengeCode.value = "";
    challengeExpiresAt.value = "";
    newPassword.value = "";
    confirmPassword.value = "";
    successMessage.value = "Password replaced successfully.";
    emit("password-replaced");
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 px-4">
    <div class="absolute inset-0 overflow-hidden pointer-events-none">
      <div class="absolute top-1/4 -left-32 w-96 h-96 bg-amber-500/10 rounded-full blur-3xl"></div>
      <div class="absolute bottom-1/4 -right-32 w-96 h-96 bg-emerald-500/10 rounded-full blur-3xl"></div>
    </div>

    <div class="relative w-full max-w-md">
      <div class="bg-slate-800/80 backdrop-blur-xl border border-slate-700/50 rounded-2xl shadow-2xl p-8">
        <div class="text-center mb-8">
          <div class="inline-flex items-center justify-center w-16 h-16 bg-gradient-to-br from-amber-500 to-emerald-500 rounded-xl mb-4">
            <svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 11c1.657 0 3 .895 3 2s-1.343 2-3 2-3 .895-3 2 1.343 2 3 2m0-8V9m0 2V7m0 2h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          <h1 class="text-2xl font-bold text-white mb-2">Forgot Password</h1>
          <p class="text-slate-400 text-sm">Generate a one-time code, then set a new admin password</p>
        </div>

        <div class="space-y-5">
          <button
            type="button"
            :disabled="requestingCode"
            class="w-full rounded-xl border border-amber-500/40 bg-amber-500/10 text-amber-100 font-semibold py-3 px-4 hover:bg-amber-500/20 disabled:opacity-60 transition-all duration-200"
            @click="handleRequestCode"
          >
            {{ requestingCode ? "Generating code..." : "Generate replace code" }}
          </button>

          <div v-if="challengeCode" class="rounded-xl border border-slate-600 bg-slate-900/40 p-4">
            <div class="text-xs uppercase tracking-[0.2em] text-slate-400">Code stored in database</div>
            <div class="mt-2 font-mono text-lg text-white tracking-[0.35em]">{{ challengeCode }}</div>
            <div class="mt-1 text-xs text-slate-400">
              Expires at {{ challengeExpiresAt }}
            </div>
          </div>

          <div>
            <label for="replaceCode" class="block text-xs font-medium text-slate-300 mb-2">
              Replace code
            </label>
            <input
              v-model="replaceCode"
              id="replaceCode"
              type="text"
              autocomplete="off"
              placeholder="Enter code"
              class="w-full bg-slate-900/50 border border-slate-600 rounded-xl py-3 px-4 text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-amber-500 focus:border-transparent transition-all uppercase tracking-[0.2em]"
            />
          </div>

          <div>
            <label for="newPassword" class="block text-xs font-medium text-slate-300 mb-2">
              New password
            </label>
            <input
              v-model="newPassword"
              id="newPassword"
              type="password"
              autocomplete="new-password"
              placeholder="Enter new password"
              class="w-full bg-slate-900/50 border border-slate-600 rounded-xl py-3 px-4 text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-amber-500 focus:border-transparent transition-all"
            />
          </div>

          <div>
            <label for="confirmPassword" class="block text-xs font-medium text-slate-300 mb-2">
              Confirm new password
            </label>
            <input
              v-model="confirmPassword"
              id="confirmPassword"
              type="password"
              autocomplete="new-password"
              placeholder="Repeat new password"
              class="w-full bg-slate-900/50 border border-slate-600 rounded-xl py-3 px-4 text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-amber-500 focus:border-transparent transition-all"
            />
          </div>

          <div
            v-if="errorMessage"
            class="rounded-xl border border-rose-500/40 bg-rose-500/10 p-3 text-rose-200 text-sm"
          >
            {{ errorMessage }}
          </div>

          <div
            v-if="successMessage"
            class="rounded-xl border border-emerald-500/40 bg-emerald-500/10 p-3 text-emerald-200 text-sm"
          >
            {{ successMessage }}
          </div>

          <button
            type="button"
            :disabled="submitting"
            class="w-full bg-gradient-to-r from-emerald-500 to-emerald-600 hover:from-emerald-600 hover:to-emerald-700 text-white font-semibold py-3 px-4 rounded-xl shadow-lg shadow-emerald-500/25 hover:shadow-emerald-500/40 transition-all duration-200 transform hover:scale-[1.02] active:scale-[0.98]"
            @click="handleSubmit"
          >
            {{ submitting ? "Replacing..." : "Replace password" }}
          </button>

          <button
            type="button"
            class="w-full rounded-xl border border-slate-600 bg-slate-900/40 text-slate-200 font-semibold py-3 px-4 hover:bg-slate-900/60 transition-all duration-200"
            @click="emit('back-to-login')"
          >
            Back to login
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
