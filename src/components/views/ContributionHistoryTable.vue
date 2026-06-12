<script setup lang="ts">
import { computed } from "vue";

type HistoryDate = {
  date: string;
  total: number;
};

type HistoryCell = {
  date: string;
  amount: number;
};

type HistoryRow = {
  id: string;
  name: string;
  total: number;
  cells: HistoryCell[];
};

const props = defineProps<{
  rows: HistoryRow[];
  dates: HistoryDate[];
  overallTotal: number;
  currentDayTotal: number;
  currentDayLabel: string;
  themeMode: "light" | "dark";
  formatMoney: (cents: number) => string;
  formatDate: (value: string) => string;
}>();

const shellClass = computed(() =>
  props.themeMode === "dark"
    ? "border-slate-700 bg-slate-900 text-slate-100"
    : "border-slate-300 bg-white text-slate-900",
);

const headerClass = computed(() =>
  props.themeMode === "dark" ? "border-slate-700 bg-slate-950" : "border-slate-200 bg-white",
);

const headRowClass = computed(() =>
  props.themeMode === "dark" ? "bg-slate-800 text-slate-200" : "bg-slate-100 text-slate-700",
);

const bodyRowClass = computed(() =>
  props.themeMode === "dark" ? "border-slate-700" : "border-slate-200",
);

const footerClass = computed(() =>
  props.themeMode === "dark" ? "border-slate-700 bg-slate-950" : "border-slate-50 bg-slate-50",
);

function isCurrentDay(date: string) {
  return date === props.currentDayLabel;
}
</script>

<template>
  <section class="rounded-[2px] border shadow-sm" :class="shellClass">
    <div class="flex flex-col gap-4 border-b px-4 py-4 lg:flex-row lg:items-end lg:justify-between" :class="headerClass">
      <div>
        <div class="text-lg font-semibold">Contribution History</div>
        <div class="mt-1 text-sm" :class="props.themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">
          Session-by-session contribution totals grouped by name, with per-person totals and a daily overall total.
        </div>
      </div>

      <div class="grid gap-3 sm:grid-cols-2">
        <div class="rounded-[2px] border px-4 py-3" :class="props.themeMode === 'dark' ? 'border-slate-700 bg-slate-950' : 'border-slate-200 bg-slate-50'">
          <div class="text-[11px] uppercase tracking-[0.3em]" :class="props.themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Current Day Total</div>
          <div class="mt-2 text-2xl font-semibold" :class="props.themeMode === 'dark' ? 'text-emerald-400' : 'text-emerald-700'">{{ formatMoney(currentDayTotal) }}</div>
        </div>
        <div class="rounded-[2px] border px-4 py-3" :class="props.themeMode === 'dark' ? 'border-slate-700 bg-slate-950' : 'border-slate-200 bg-slate-50'">
          <div class="text-[11px] uppercase tracking-[0.3em]" :class="props.themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">Overall Total</div>
          <div class="mt-2 text-2xl font-semibold" :class="props.themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">{{ formatMoney(overallTotal) }}</div>
        </div>
      </div>
    </div>

    <div class="overflow-x-auto">
      <table class="w-full caption-bottom text-sm">
        <thead :class="headRowClass">
          <tr class="border-b" :class="props.themeMode === 'dark' ? 'border-slate-700' : 'border-slate-200'">
            <th rowspan="2" class="whitespace-nowrap px-4 py-3 text-left font-semibold uppercase tracking-[0.2em]">
              Name
            </th>
            <th :colspan="Math.max(dates.length, 1)" class="whitespace-nowrap px-4 py-3 text-center font-semibold uppercase tracking-[0.2em]">
              Session Date
            </th>
            <th rowspan="2" class="whitespace-nowrap px-4 py-3 text-right font-semibold uppercase tracking-[0.2em]">
              Total
            </th>
          </tr>
          <tr class="border-b" :class="props.themeMode === 'dark' ? 'border-slate-700' : 'border-slate-200'">
            <th
              v-for="date in dates"
              :key="date.date"
              class="min-w-[10rem] px-4 py-3 text-center font-medium"
              :class="[
                props.themeMode === 'dark' ? 'text-slate-300' : 'text-slate-600',
                isCurrentDay(date.date) ? (props.themeMode === 'dark' ? 'bg-emerald-950 text-emerald-300' : 'bg-emerald-50 text-emerald-800') : '',
              ]"
            >
              <div class="flex flex-col items-center gap-1">
                <span class="whitespace-nowrap">{{ formatDate(date.date) }}</span>
                <span class="text-[11px] uppercase tracking-[0.25em]" :class="props.themeMode === 'dark' ? 'text-slate-500' : 'text-slate-400'">Session</span>
              </div>
            </th>
            <th v-if="dates.length === 0" class="px-4 py-3 text-center" :class="props.themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">
              No session dates
            </th>
          </tr>
        </thead>

        <tbody>
          <tr
            v-for="row in rows"
            :key="row.id"
            class="border-b last:border-b-0"
            :class="bodyRowClass"
          >
            <th scope="row" class="px-4 py-3 text-left font-medium" :class="props.themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
              <div class="flex flex-col">
                <span>{{ row.name }}</span>
                <span class="text-[11px] uppercase tracking-[0.25em]" :class="props.themeMode === 'dark' ? 'text-slate-500' : 'text-slate-400'">Member</span>
              </div>
            </th>
            <template v-if="dates.length > 0">
              <td
                v-for="cell in row.cells"
                :key="`${row.id}:${cell.date}`"
                class="px-4 py-3 text-center"
                :class="isCurrentDay(cell.date) ? (props.themeMode === 'dark' ? 'bg-emerald-950' : 'bg-emerald-50') : ''"
              >
                <span v-if="cell.amount > 0" class="font-semibold" :class="props.themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
                  {{ formatMoney(cell.amount) }}
                </span>
                <span v-else :class="props.themeMode === 'dark' ? 'text-slate-500' : 'text-slate-400'">-</span>
              </td>
            </template>
            <td v-else class="px-4 py-3 text-center" :class="props.themeMode === 'dark' ? 'text-slate-500' : 'text-slate-400'">
              -
            </td>
            <td class="px-4 py-3 text-right font-semibold" :class="props.themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
              {{ formatMoney(row.total) }}
            </td>
          </tr>

          <tr v-if="rows.length === 0">
            <td :colspan="Math.max(dates.length + 2, 3)" class="px-4 py-6 text-center" :class="props.themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">
              No contribution history yet.
            </td>
          </tr>
        </tbody>

        <tfoot class="border-t" :class="footerClass">
          <tr>
            <th scope="row" class="px-4 py-3 text-left font-semibold uppercase tracking-[0.2em]" :class="props.themeMode === 'dark' ? 'text-slate-300' : 'text-slate-700'">
              Total
            </th>
            <td
              v-for="date in dates"
              :key="`${date.date}-total`"
              class="px-4 py-3 text-center font-semibold"
              :class="[
                props.themeMode === 'dark' ? 'text-slate-200' : 'text-slate-700',
                isCurrentDay(date.date) ? (props.themeMode === 'dark' ? 'text-emerald-300' : 'text-emerald-800') : '',
              ]"
            >
              {{ formatMoney(date.total) }}
            </td>
            <td v-if="dates.length === 0" class="px-4 py-3 text-center" :class="props.themeMode === 'dark' ? 'text-slate-500' : 'text-slate-500'">
              -
            </td>
            <td class="px-4 py-3 text-right font-semibold" :class="props.themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
              {{ formatMoney(overallTotal) }}
            </td>
          </tr>
        </tfoot>
      </table>
    </div>
  </section>
</template>
