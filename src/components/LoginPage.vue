<script setup lang="ts">
import { ref } from "vue";

const emit = defineEmits<{
  (e: "login", password: string): void;
}>();

const password = ref("");
const submitting = ref(false);
const errorMessage = ref<string | null>(null);

async function handleLogin() {
  errorMessage.value = null;
  submitting.value = true;
  try {
    emit("login", password.value);
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 px-4">
    <!-- Decorative Background Elements -->
    <div class="absolute inset-0 overflow-hidden pointer-events-none">
      <div class="absolute top-1/4 -left-32 w-96 h-96 bg-indigo-500/10 rounded-full blur-3xl"></div>
      <div class="absolute bottom-1/4 -right-32 w-96 h-96 bg-emerald-500/10 rounded-full blur-3xl"></div>
    </div>

    <!-- Login Card -->
    <div class="relative w-full max-w-md">
      <div class="bg-slate-800/80 backdrop-blur-xl border border-slate-700/50 rounded-2xl shadow-2xl p-8">
        <!-- Header -->
        <div class="text-center mb-8">
          <div class="inline-flex items-center justify-center w-16 h-16 bg-gradient-to-br from-indigo-500 to-emerald-500 rounded-xl mb-4">
            <svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          <h1 class="text-2xl font-bold text-white mb-2">Admin Login</h1>
          <p class="text-slate-400 text-sm">Enter your password to unlock the ledger</p>
        </div>

        <!-- Login Form -->
        <form @submit.prevent="handleLogin" class="space-y-5">
          <!-- Password Field -->
          <div>
            <label for="password" class="block text-sm font-medium text-slate-300 mb-2">Password</label>
            <div class="relative">
              <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                <svg class="w-5 h-5 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
              </div>
              <input
                v-model="password"
                type="password"
                id="password"
                placeholder="Enter your password"
                class="w-full bg-slate-900/50 border border-slate-600 rounded-xl py-3 pl-12 pr-4 text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all"
                required
              />
            </div>
          </div>

          <div
            v-if="errorMessage"
            class="rounded-xl border border-rose-500/40 bg-rose-500/10 p-3 text-rose-200 text-sm"
          >
            {{ errorMessage }}
          </div>

          <!-- Login Button -->
          <button
            type="submit"
            :disabled="submitting"
            class="w-full bg-gradient-to-r from-indigo-500 to-indigo-600 hover:from-indigo-600 hover:to-indigo-700 text-white font-semibold py-3 px-4 rounded-xl shadow-lg shadow-indigo-500/25 hover:shadow-indigo-500/40 transition-all duration-200 transform hover:scale-[1.02] active:scale-[0.98]"
          >
            {{ submitting ? "Unlocking…" : "Unlock" }}
          </button>
        </form>
      </div>
    </div>
  </div>
</template>
