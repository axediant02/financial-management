<script setup lang="ts">
import { computed, ref, watch } from "vue";

const props = defineProps<{
  modelValue: string;
  themeMode: "light" | "dark";
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();

const today = new Date();
const monthAnchor = ref<Date>(new Date(today.getFullYear(), today.getMonth(), 1));

const weekdayLabels = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

function parseDate(value: string) {
  const parsed = new Date(`${value}T00:00:00`);
  return Number.isNaN(parsed.getTime()) ? null : parsed;
}

function toDateString(date: Date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function isSameDay(left: Date, right: Date) {
  return left.getFullYear() === right.getFullYear() && left.getMonth() === right.getMonth() && left.getDate() === right.getDate();
}

function startOfMonth(date: Date) {
  return new Date(date.getFullYear(), date.getMonth(), 1);
}

function addMonths(date: Date, amount: number) {
  return new Date(date.getFullYear(), date.getMonth() + amount, 1);
}

function selectDate(date: Date) {
  emit("update:modelValue", toDateString(date));
}

const selectedDate = computed(() => parseDate(props.modelValue));

watch(
  () => props.modelValue,
  (value) => {
    const parsed = parseDate(value);
    if (parsed) {
      monthAnchor.value = startOfMonth(parsed);
    }
  },
  { immediate: true },
);

const gridDays = computed(() => {
  const first = startOfMonth(monthAnchor.value);
  const startOffset = first.getDay();
  const start = new Date(first);
  start.setDate(first.getDate() - startOffset);

  return Array.from({ length: 42 }, (_, index) => {
    const current = new Date(start);
    current.setDate(start.getDate() + index);
    const isOutsideMonth = current.getMonth() !== monthAnchor.value.getMonth();
    const isSelected = selectedDate.value ? isSameDay(current, selectedDate.value) : false;
    const isToday = isSameDay(current, today);
    return {
      date: current,
      label: current.getDate(),
      outside: isOutsideMonth,
      selected: isSelected,
      today: isToday,
    };
  });
});
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between gap-3">
      <div>
        <div class="text-sm font-semibold uppercase tracking-[0.2em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">
          Calendar
        </div>
        <div class="mt-1 text-base font-semibold" :class="themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
          {{ monthAnchor.toLocaleString("en-US", { month: "long", year: "numeric" }) }}
        </div>
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          class="rounded-[2px] border px-3 py-2 text-sm font-semibold"
          :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100 hover:bg-slate-700' : 'border-slate-300 bg-white text-slate-900 hover:bg-slate-50'"
          @click="monthAnchor = addMonths(monthAnchor, -1)"
        >
          Prev
        </button>
        <button
          type="button"
          class="rounded-[2px] border px-3 py-2 text-sm font-semibold"
          :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100 hover:bg-slate-700' : 'border-slate-300 bg-white text-slate-900 hover:bg-slate-50'"
          @click="monthAnchor = addMonths(monthAnchor, 1)"
        >
          Next
        </button>
      </div>
    </div>

    <div class="grid grid-cols-7 gap-2 text-center text-[11px] font-semibold uppercase tracking-[0.2em]" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">
      <div
        v-for="day in weekdayLabels"
        :key="day"
        class="rounded-[2px] py-1"
        :class="day === 'Sun' ? (themeMode === 'dark' ? 'bg-red-500/10 text-red-300' : 'bg-red-50 text-red-700') : ''"
      >
        {{ day }}
      </div>
    </div>

    <div class="grid grid-cols-7 gap-2">
      <button
        v-for="day in gridDays"
        :key="day.date.toISOString()"
        type="button"
        class="flex aspect-square flex-col items-center justify-center rounded-[2px] border text-sm font-semibold transition"
        :class="[
          themeMode === 'dark' ? 'border-slate-700 bg-slate-900 text-slate-100 hover:bg-slate-800' : 'border-slate-200 bg-white text-slate-900 hover:bg-slate-50',
          day.outside ? (themeMode === 'dark' ? 'text-slate-600' : 'text-slate-400') : '',
          day.date.getDay() === 0 && !day.selected ? (themeMode === 'dark' ? 'border-red-500/40 bg-red-500/15 text-red-200 hover:bg-red-500/25' : 'border-red-300 bg-red-100 text-red-800 hover:bg-red-200') : '',
          day.selected && day.date.getDay() !== 0 ? (themeMode === 'dark' ? 'border-cyan-300 bg-cyan-400 text-slate-950 shadow-[0_0_0_2px_rgba(34,211,238,0.25)]' : 'border-cyan-600 bg-cyan-300 text-slate-950 shadow-[0_0_0_2px_rgba(8,145,178,0.18)]') : '',
          day.today && !day.selected ? (themeMode === 'dark' ? 'border-emerald-500/40' : 'border-emerald-300') : '',
          day.selected && day.date.getDay() === 0 ? (themeMode === 'dark' ? 'border-pink-300 bg-pink-400 text-slate-950 shadow-[0_0_0_2px_rgba(244,114,182,0.25)]' : 'border-pink-600 bg-pink-300 text-slate-950 shadow-[0_0_0_2px_rgba(219,39,119,0.18)]') : '',
        ]"
        @click="selectDate(day.date)"
      >
        <span :class="day.selected ? 'font-bold' : (day.date.getDay() === 0 ? (themeMode === 'dark' ? 'text-red-200' : 'text-red-700') : '')">
          {{ day.label }}
        </span>
      </button>
    </div>

    <div class="rounded-[2px] border px-3 py-2 text-sm" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900 text-slate-300' : 'border-slate-200 bg-slate-50 text-slate-700'">
      Selected:
      <span class="font-semibold" :class="themeMode === 'dark' ? 'text-slate-100' : 'text-slate-900'">
        {{ modelValue || "No date selected" }}
      </span>
    </div>
  </div>
</template>
