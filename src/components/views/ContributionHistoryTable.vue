<script setup lang="ts">
import { computed, ref, watch } from "vue";

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

type ProjectHistoryEntry = {
  id: string;
  date: string;
  contributor: string;
  amount: number;
};

type VisibleSlot =
  | { kind: "date"; key: string; date: string; total: number }
  | { kind: "empty"; key: string };

function getCellAmount(row: HistoryRow, date: string) {
  return row.cells.find((cell) => cell.date === date)?.amount ?? 0;
}

const props = defineProps<{
  rows: HistoryRow[];
  dates: HistoryDate[];
  overallTotal: number;
  currentDayTotal: number;
  currentDayLabel: string;
  themeMode: "light" | "dark";
  formatMoney: (cents: number) => string;
  formatDate: (value: string) => string;
  projectEntries?: ProjectHistoryEntry[];
  projectTitle?: string;
}>();

const emit = defineEmits<{
  (e: "page-change", startIndex: number): void;
}>();

const pageSize = 5;
const pageStart = ref(0);
const showAll = ref(false);
const userHasNavigated = ref(false);
const hasProjectEntries = computed(() => props.projectEntries !== undefined);
const projectEntries = computed(() => props.projectEntries ?? []);
const projectEntriesTotal = computed(() => projectEntries.value.reduce((sum, entry) => sum + entry.amount, 0));
const projectEntriesCount = computed(() => projectEntries.value.length);

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

const maxStartIndex = computed(() => Math.max(props.dates.length - pageSize, 0));

const visibleDates = computed(() => props.dates.slice(pageStart.value, pageStart.value + pageSize));

const visibleSlots = computed<VisibleSlot[]>(() => {
  const slots: VisibleSlot[] = visibleDates.value.map((date) => ({
    kind: "date",
    key: date.date,
    date: date.date,
    total: date.total,
  }));

  while (slots.length < pageSize) {
    slots.push({ kind: "empty", key: `empty-${pageStart.value}-${slots.length}` });
  }

  return slots;
});

const visibleDateTotal = computed(() => visibleDates.value.length);

const isLastWindow = computed(() => pageStart.value >= maxStartIndex.value);

const windowLabel = computed(() => {
  if (props.dates.length === 0) return "No dates";
  const start = visibleDates.value[0];
  const end = visibleDates.value[visibleDates.value.length - 1];
  if (!start || !end) return "No dates";
  return `${props.formatDate(start.date)} to ${props.formatDate(end.date)}`;
});

function clampPageStart(value: number) {
  return Math.min(Math.max(value, 0), maxStartIndex.value);
}

function syncPageStart(nextStart: number) {
  pageStart.value = clampPageStart(nextStart);
  emit("page-change", pageStart.value);
}

function previousPage() {
  userHasNavigated.value = true;
  syncPageStart(pageStart.value - pageSize);
}

function nextPage() {
  userHasNavigated.value = true;
  syncPageStart(pageStart.value + pageSize);
}

function showFullHistory() {
  showAll.value = true;
}

function hideFullHistory() {
  showAll.value = false;
}

watch(
  () => props.dates.length,
  () => {
    if (!userHasNavigated.value) {
      syncPageStart(maxStartIndex.value);
    } else if (pageStart.value > maxStartIndex.value) {
      syncPageStart(maxStartIndex.value);
    } else {
      pageStart.value = clampPageStart(pageStart.value);
      emit("page-change", pageStart.value);
    }
  },
);

function isCurrentDay(date: string) {
  return date === props.currentDayLabel;
}
</script>

<template>
  <section class="rounded-[2px] border shadow-sm" :class="shellClass">
    <template v-if="hasProjectEntries">
      <div class="flex flex-col gap-4 border-b px-4 py-4 lg:flex-row lg:items-end lg:justify-between" :class="headerClass">
        <div>
          <div class="text-lg font-semibold">Contribution History</div>
          <div class="mt-1 text-sm" :class="props.themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">
            {{ projectTitle ? `Project-only contribution log for ${projectTitle}.` : "Project-only contribution log with the date, contributor, and amount for each record." }}
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
              <th class="whitespace-nowrap px-4 py-3 text-left font-semibold uppercase tracking-[0.2em]">
                Contribution Date
              </th>
              <th class="whitespace-nowrap px-4 py-3 text-left font-semibold uppercase tracking-[0.2em]">
                Person
              </th>
              <th class="whitespace-nowrap px-4 py-3 text-right font-semibold uppercase tracking-[0.2em]">
                Amount
              </th>
            </tr>
          </thead>

          <tbody>
            <tr
              v-for="entry in projectEntries"
              :key="entry.id"
              class="border-b last:border-b-0"
              :class="bodyRowClass"
            >
              <td class="px-4 py-3" :class="props.themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
                {{ formatDate(entry.date) }}
              </td>
              <td class="px-4 py-3 font-medium" :class="props.themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
                {{ entry.contributor }}
              </td>
              <td class="px-4 py-3 text-right font-semibold" :class="props.themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
                {{ formatMoney(entry.amount) }}
              </td>
            </tr>

            <tr v-if="projectEntries.length === 0">
              <td colspan="3" class="px-4 py-6 text-center" :class="props.themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">
                No contributions recorded for this project yet.
              </td>
            </tr>
          </tbody>

          <tfoot class="border-t" :class="footerClass">
            <tr>
              <th scope="row" class="px-4 py-3 text-left font-semibold uppercase tracking-[0.2em]" :class="props.themeMode === 'dark' ? 'text-slate-300' : 'text-slate-700'">
                Total
              </th>
              <td class="px-4 py-3 text-left font-semibold" :class="props.themeMode === 'dark' ? 'text-slate-300' : 'text-slate-700'">
                {{ projectEntriesCount }} entries
              </td>
              <td class="px-4 py-3 text-right font-semibold" :class="props.themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
                {{ formatMoney(projectEntriesTotal) }}
              </td>
            </tr>
          </tfoot>
        </table>
      </div>
    </template>
    <template v-else>
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

    <div class="flex flex-col gap-3 border-b px-4 py-3 sm:flex-row sm:items-center sm:justify-between" :class="headerClass">
      <div class="text-sm" :class="props.themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">
        Showing <span class="font-semibold" :class="props.themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">{{ visibleDateTotal }}</span>
        of <span class="font-semibold" :class="props.themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">{{ dates.length }}</span> session dates
        <span v-if="dates.length > 0">- {{ windowLabel }}</span>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        <button
          type="button"
          class="rounded-[2px] border px-3 py-2 text-sm font-semibold transition"
          :class="[
            props.themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100 hover:bg-slate-700' : 'border-slate-300 bg-white text-slate-900 hover:bg-slate-50',
              pageStart <= 0 ? 'opacity-50' : '',
            ]"
          :disabled="pageStart <= 0"
          @click="previousPage"
        >
          Prev
        </button>
        <button
          type="button"
          class="rounded-[2px] border px-3 py-2 text-sm font-semibold transition"
          :class="[
            props.themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100 hover:bg-slate-700' : 'border-slate-300 bg-white text-slate-900 hover:bg-slate-50',
            isLastWindow ? 'opacity-50' : '',
          ]"
          :disabled="isLastWindow"
          @click="nextPage"
        >
          Next
        </button>
        <button
          type="button"
          class="rounded-[2px] border px-3 py-2 text-sm font-semibold"
          :class="props.themeMode === 'dark' ? 'border-blue-500 bg-blue-500 text-white hover:bg-blue-400' : 'border-blue-600 bg-blue-600 text-white hover:bg-blue-500'"
          @click="showFullHistory"
        >
          See all
        </button>
      </div>
    </div>

    <div class="overflow-x-auto">
      <table class="w-full caption-bottom text-sm">
        <thead :class="headRowClass">
          <tr class="border-b" :class="props.themeMode === 'dark' ? 'border-slate-700' : 'border-slate-200'">
            <th rowspan="2" class="whitespace-nowrap px-4 py-3 text-left font-semibold uppercase tracking-[0.2em]">
              Name
            </th>
            <th :colspan="pageSize" class="whitespace-nowrap px-4 py-3 text-center font-semibold uppercase tracking-[0.2em]">
              Session Date
            </th>
            <th rowspan="2" class="whitespace-nowrap px-4 py-3 text-right font-semibold uppercase tracking-[0.2em]">
              Total
            </th>
          </tr>
          <tr class="border-b" :class="props.themeMode === 'dark' ? 'border-slate-700' : 'border-slate-200'">
            <th
              v-for="slot in visibleSlots"
              :key="slot.key"
              class="min-w-[10rem] px-4 py-3 text-center font-medium"
              :class="slot.kind === 'date'
                ? [
                    props.themeMode === 'dark' ? 'text-slate-300' : 'text-slate-600',
                    isCurrentDay(slot.date) ? (props.themeMode === 'dark' ? 'bg-emerald-950 text-emerald-300' : 'bg-emerald-50 text-emerald-800') : '',
                  ]
                : [props.themeMode === 'dark' ? 'text-slate-600' : 'text-slate-400']"
            >
              <div class="flex flex-col items-center gap-1">
                <span v-if="slot.kind === 'date'" class="whitespace-nowrap">{{ formatDate(slot.date) }}</span>
                <span v-else class="whitespace-nowrap">-</span>
                <span class="text-[11px] uppercase tracking-[0.25em]" :class="props.themeMode === 'dark' ? 'text-slate-500' : 'text-slate-400'">
                  {{ slot.kind === 'date' ? 'Session' : 'Open slot' }}
                </span>
              </div>
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
            <template v-if="visibleSlots.length > 0">
              <td
                v-for="slot in visibleSlots"
                :key="`${row.id}:${slot.key}`"
                class="px-4 py-3 text-center"
                :class="slot.kind === 'date' && isCurrentDay(slot.date) ? (props.themeMode === 'dark' ? 'bg-emerald-950' : 'bg-emerald-50') : ''"
              >
                <template v-if="slot.kind === 'date'">
                  <span v-if="getCellAmount(row, slot.date) > 0" class="font-semibold" :class="props.themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
                    {{ formatMoney(getCellAmount(row, slot.date)) }}
                  </span>
                  <span v-else :class="props.themeMode === 'dark' ? 'text-slate-500' : 'text-slate-400'">-</span>
                </template>
                <span v-else :class="props.themeMode === 'dark' ? 'text-slate-500' : 'text-slate-400'">-</span>
              </td>
            </template>
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
              v-for="slot in visibleSlots"
              :key="`${slot.key}-total`"
              class="px-4 py-3 text-center font-semibold"
              :class="slot.kind === 'date'
                ? [
                    props.themeMode === 'dark' ? 'text-slate-200' : 'text-slate-700',
                    isCurrentDay(slot.date) ? (props.themeMode === 'dark' ? 'text-emerald-300' : 'text-emerald-800') : '',
                  ]
                : [props.themeMode === 'dark' ? 'text-slate-600' : 'text-slate-400']"
            >
              <template v-if="slot.kind === 'date'">
                {{ formatMoney(slot.total) }}
              </template>
              <template v-else>
                -
              </template>
            </td>
            <td class="px-4 py-3 text-right font-semibold" :class="props.themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
              {{ formatMoney(overallTotal) }}
            </td>
          </tr>
        </tfoot>
      </table>
    </div>

    <div v-if="showAll" class="fixed inset-0 z-50">
      <div class="absolute inset-0 bg-black/60" @click="hideFullHistory"></div>
      <div class="absolute inset-0 flex items-center justify-center p-4">
        <div class="flex h-[90vh] w-full max-w-7xl flex-col overflow-hidden rounded-[2px] border shadow-2xl" :class="shellClass">
          <div class="flex items-start justify-between gap-4 border-b px-4 py-4" :class="headerClass">
            <div>
              <div class="text-lg font-semibold">Full Contribution History</div>
              <div class="mt-1 text-sm" :class="props.themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">
                Complete date history with every session visible at once.
              </div>
            </div>
            <button
              type="button"
              class="rounded-[2px] border px-3 py-2 text-sm font-semibold"
              :class="props.themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100 hover:bg-slate-700' : 'border-slate-300 bg-white text-slate-900 hover:bg-slate-50'"
              @click="hideFullHistory"
            >
              Close
            </button>
          </div>

          <div class="overflow-auto p-4">
            <div class="overflow-x-auto">
              <table class="w-full caption-bottom text-sm">
                <thead :class="headRowClass">
                  <tr class="border-b" :class="props.themeMode === 'dark' ? 'border-slate-700' : 'border-slate-200'">
                    <th rowspan="2" class="whitespace-nowrap px-4 py-3 text-left font-semibold uppercase tracking-[0.2em]">Name</th>
                    <th :colspan="pageSize" class="whitespace-nowrap px-4 py-3 text-center font-semibold uppercase tracking-[0.2em]">Session Date</th>
                    <th rowspan="2" class="whitespace-nowrap px-4 py-3 text-right font-semibold uppercase tracking-[0.2em]">Total</th>
                  </tr>
                  <tr class="border-b" :class="props.themeMode === 'dark' ? 'border-slate-700' : 'border-slate-200'">
                    <th
                      v-for="date in dates"
                      :key="`full-${date.date}`"
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
                    <th v-if="dates.length === 0" class="px-4 py-3 text-center" :class="props.themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">No session dates</th>
                  </tr>
                </thead>

                <tbody>
                  <tr
                    v-for="row in rows"
                    :key="`full-${row.id}`"
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
                        :key="`full-${row.id}:${cell.date}`"
                        class="px-4 py-3 text-center"
                        :class="isCurrentDay(cell.date) ? (props.themeMode === 'dark' ? 'bg-emerald-950' : 'bg-emerald-50') : ''"
                      >
                        <span v-if="cell.amount > 0" class="font-semibold" :class="props.themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
                          {{ formatMoney(cell.amount) }}
                        </span>
                        <span v-else :class="props.themeMode === 'dark' ? 'text-slate-500' : 'text-slate-400'">-</span>
                      </td>
                    </template>
                    <td v-else class="px-4 py-3 text-center" :class="props.themeMode === 'dark' ? 'text-slate-500' : 'text-slate-400'">-</td>
                    <td class="px-4 py-3 text-right font-semibold" :class="props.themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
                      {{ formatMoney(row.total) }}
                    </td>
                  </tr>
                </tbody>

                <tfoot class="border-t" :class="footerClass">
                  <tr>
                    <th scope="row" class="px-4 py-3 text-left font-semibold uppercase tracking-[0.2em]" :class="props.themeMode === 'dark' ? 'text-slate-300' : 'text-slate-700'">Total</th>
                    <td
                      v-for="date in dates"
                      :key="`full-${date.date}-total`"
                      class="px-4 py-3 text-center font-semibold"
                      :class="[
                        props.themeMode === 'dark' ? 'text-slate-200' : 'text-slate-700',
                        isCurrentDay(date.date) ? (props.themeMode === 'dark' ? 'text-emerald-300' : 'text-emerald-800') : '',
                      ]"
                    >
                      {{ formatMoney(date.total) }}
                    </td>
                    <td v-if="dates.length === 0" class="px-4 py-3 text-center" :class="props.themeMode === 'dark' ? 'text-slate-500' : 'text-slate-500'">-</td>
                    <td class="px-4 py-3 text-right font-semibold" :class="props.themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
                      {{ formatMoney(overallTotal) }}
                    </td>
                  </tr>
                </tfoot>
              </table>
            </div>
          </div>
        </div>
      </div>
    </div>
    </template>
  </section>
</template>
