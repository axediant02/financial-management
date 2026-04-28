<script setup lang="ts">
import { onMounted, ref } from "vue";
import { ledgerSummary, projectBalances } from "../../lib/api";
import { formatPHPFromCents } from "../../lib/money";
import type { LedgerSummary, ProjectBalanceRow } from "../../lib/types";

const props = defineProps<{ sessionToken: string }>();
const emit = defineEmits<{
  (e: "create-project"): void;
  (e: "open-project", id: number): void;
}>();

const loading = ref(true);
const summary = ref<LedgerSummary | null>(null);
const balances = ref<ProjectBalanceRow[]>([]);
const errorMessage = ref<string | null>(null);

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    summary.value = await ledgerSummary(props.sessionToken, { from: null, to: null, project_id: null });
    balances.value = await projectBalances(props.sessionToken, { from: null, to: null, project_id: null });
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(load);
</script>

<template>
  <div>
    <div v-if="errorMessage" class="rounded-xl border border-rose-500/40 bg-rose-500/10 p-4 text-rose-200">
      {{ errorMessage }}
    </div>

    <div v-else-if="loading" class="text-slate-300">Loading…</div>

    <div v-else class="space-y-6">
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
          <div class="text-xs uppercase tracking-wider text-slate-400">Total</div>
          <div class="mt-2 text-2xl font-bold">{{ formatPHPFromCents(summary?.total_donations_cents || 0) }}</div>
          <div class="mt-1 text-xs text-slate-500">Total accumulated contributions</div>
        </div>
        <div class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
          <div class="text-xs uppercase tracking-wider text-slate-400">Total Expenses</div>
          <div class="mt-2 text-2xl font-bold">{{ formatPHPFromCents(summary?.total_expenses_cents || 0) }}</div>
        </div>
        <div class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
          <div class="text-xs uppercase tracking-wider text-slate-400">Balance</div>
          <div class="mt-2 text-2xl font-bold">{{ formatPHPFromCents(summary?.balance_cents || 0) }}</div>
        </div>
      </div>

      <div class="rounded-2xl border border-slate-800 bg-slate-900/40 overflow-hidden">
        <div class="p-5 flex items-center justify-between">
          <div>
            <div class="font-semibold">Project Balances</div>
            <div class="text-sm text-slate-400">Contributions minus expenses per project</div>
          </div>
          <button class="rounded-lg bg-slate-800 hover:bg-slate-700 px-3 py-2 text-xs font-semibold" @click="load">
            Refresh
          </button>
        </div>
        <div class="border-t border-slate-800">
          <table class="w-full text-sm">
            <thead class="bg-slate-950/40 text-slate-300">
              <tr>
                <th class="text-left p-3 font-medium">Project</th>
                <th class="text-right p-3 font-medium">Contributions</th>
                <th class="text-right p-3 font-medium">Expenses</th>
                <th class="text-right p-3 font-medium">Balance</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="row in balances"
                :key="row.project_id"
                class="border-t border-slate-800 hover:bg-slate-950/40 cursor-pointer"
                @click="emit('open-project', row.project_id)"
              >
                <td class="p-3">{{ row.project_name }}</td>
                <td class="p-3 text-right">{{ formatPHPFromCents(row.donations_cents) }}</td>
                <td class="p-3 text-right">{{ formatPHPFromCents(row.expenses_cents) }}</td>
                <td class="p-3 text-right font-semibold">{{ formatPHPFromCents(row.balance_cents) }}</td>
              </tr>
              <tr v-if="balances.length === 0" class="border-t border-slate-800">
                <td class="p-3 text-slate-400" colspan="4">No projects yet.</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <button
        type="button"
        class="fixed bottom-6 right-6 h-14 w-14 rounded-2xl bg-indigo-600 hover:bg-indigo-500 shadow-2xl shadow-indigo-600/25 border border-indigo-400/30 flex items-center justify-center"
        title="Add project"
        @click="emit('create-project')"
      >
        <svg class="h-7 w-7 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v14m7-7H5" />
        </svg>
      </button>
    </div>
  </div>
</template>
