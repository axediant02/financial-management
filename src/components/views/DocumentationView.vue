<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import {
  documentationDetail,
  documentationExpensesCreate,
  documentationExpensesDelete,
  documentationsCreate,
  documentationsDelete,
  documentationsList,
  exportCsv,
} from "../../lib/api";
import { notify } from "../../lib/feedback";
import { centsFromPesos, formatPHPFromCents } from "../../lib/money";
import type { DocumentationDetail, DocumentationRecord } from "../../lib/types";

const props = defineProps<{ sessionToken: string }>();

const DOC_KEY = "pft_nav_documentation_id";

const loading = ref(true);
const savingDocumentation = ref(false);
const savingExpense = ref(false);
const errorMessage = ref<string | null>(null);
const records = ref<DocumentationRecord[]>([]);
const detail = ref<DocumentationDetail | null>(null);
const searchQuery = ref("");
const selectedDocumentationId = ref<number | null>(null);

const formEventName = ref("");
const formEventDate = ref(getLocalDateString());
const formCollected = ref("");
const formNotes = ref("");

const expenseDate = ref(getLocalDateString());
const expensePayee = ref("");
const expenseAmount = ref("");
const expenseNotes = ref("");

function getLocalDateString(date = new Date()) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function formatDateLabel(value: string) {
  const parsed = new Date(`${value}T00:00:00`);
  if (Number.isNaN(parsed.getTime())) return value;
  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "2-digit",
    year: "numeric",
  }).format(parsed);
}

function normalize(value: string) {
  return value.trim().toLowerCase();
}

function clearDocumentationForm() {
  formEventName.value = "";
  formEventDate.value = getLocalDateString();
  formCollected.value = "";
  formNotes.value = "";
}

function clearExpenseForm() {
  expenseDate.value = getLocalDateString();
  expensePayee.value = "";
  expenseAmount.value = "";
  expenseNotes.value = "";
}

const filteredRecords = computed(() => {
  const query = normalize(searchQuery.value);
  if (!query) return records.value;

  return records.value.filter((record) => {
    const haystack = [
      record.event_name,
      record.event_date,
      formatDateLabel(record.event_date),
      record.notes || "",
      formatPHPFromCents(record.registration_collected_cents),
      formatPHPFromCents(record.expenses_cents),
      formatPHPFromCents(record.balance_cents),
    ]
      .join(" ")
      .toLowerCase();
    return haystack.includes(query);
  });
});

const filteredTotals = computed(() =>
  filteredRecords.value.reduce(
    (acc, record) => {
      acc.collected += record.registration_collected_cents;
      acc.expenses += record.expenses_cents;
      acc.balance += record.balance_cents;
      return acc;
    },
    { collected: 0, expenses: 0, balance: 0 },
  ),
);

const selectedRecord = computed(() => detail.value?.documentation || null);
const selectedExpenses = computed(() => detail.value?.expenses || []);

async function loadRecords() {
  const list = await documentationsList(props.sessionToken);
  records.value = list;

  const persisted = localStorage.getItem(DOC_KEY);
  const persistedId = persisted && /^\d+$/.test(persisted) ? Number(persisted) : null;
  const nextSelected =
    (persistedId != null && list.some((record) => record.id === persistedId) && persistedId) ||
    (selectedDocumentationId.value != null && list.some((record) => record.id === selectedDocumentationId.value)
      ? selectedDocumentationId.value
      : null) ||
    list[0]?.id ||
    null;

  if (nextSelected !== selectedDocumentationId.value) {
    selectedDocumentationId.value = nextSelected;
  } else if (nextSelected == null) {
    detail.value = null;
  }
}

async function loadDetail(id: number) {
  detail.value = await documentationDetail(props.sessionToken, id);
}

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    await loadRecords();
    if (selectedDocumentationId.value != null) {
      await loadDetail(selectedDocumentationId.value);
    }
    if (records.value.length === 0) {
      detail.value = null;
    }
  } catch (error: any) {
    errorMessage.value = String(error);
  } finally {
    loading.value = false;
  }
}

watch(
  () => selectedDocumentationId.value,
  async (value) => {
    if (value == null) {
      localStorage.removeItem(DOC_KEY);
      detail.value = null;
      return;
    }
    localStorage.setItem(DOC_KEY, String(value));
    try {
      detail.value = await documentationDetail(props.sessionToken, value);
    } catch (error: any) {
      errorMessage.value = String(error);
    }
  },
);

async function exportCurrentCsv() {
  const dest = await save({
    defaultPath: "documentation-register.csv",
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!dest) return;

  await exportCsv(props.sessionToken, {
    kind: "documentations",
    filter: { from: null, to: null, project_id: null },
    dest_path: dest,
  });
  notify("Documentation export created.");
}

async function submitDocumentation() {
  errorMessage.value = null;
  savingDocumentation.value = true;

  try {
    const eventName = formEventName.value.trim();
    if (!eventName) {
      notify("Enter an event name first.");
      return;
    }

    const collectedCents = centsFromPesos(formCollected.value);
    if (collectedCents <= 0) {
      notify("Total registration collected must be greater than 0.");
      return;
    }

    if (!confirm(`Save documentation for "${eventName}"?`)) return;

    const created = await documentationsCreate(props.sessionToken, {
      event_name: eventName,
      event_date: formEventDate.value,
      registration_collected_cents: collectedCents,
      notes: formNotes.value.trim() || null,
    });

    clearDocumentationForm();
    await loadRecords();
    selectedDocumentationId.value = created.id;
    await loadDetail(created.id);
    notify("Documentation saved.");
  } catch (error: any) {
    errorMessage.value = String(error);
  } finally {
    savingDocumentation.value = false;
  }
}

async function submitExpense() {
  errorMessage.value = null;
  if (selectedDocumentationId.value == null) {
    notify("Select a registration first.");
    return;
  }

  savingExpense.value = true;
  try {
    const amountCents = centsFromPesos(expenseAmount.value);
    if (amountCents <= 0) {
      notify("Expense amount must be greater than 0.");
      return;
    }

    if (!confirm("Save this expense deduction?")) return;

    await documentationExpensesCreate(props.sessionToken, {
      documentation_id: selectedDocumentationId.value,
      spent_at: expenseDate.value,
      amount_cents: amountCents,
      payee: expensePayee.value.trim() || null,
      notes: expenseNotes.value.trim() || null,
    });

    clearExpenseForm();
    await loadRecords();
    await loadDetail(selectedDocumentationId.value);
    notify("Expense saved.");
  } catch (error: any) {
    errorMessage.value = String(error);
  } finally {
    savingExpense.value = false;
  }
}

async function removeDocumentation(id: number, eventName: string) {
  if (!confirm(`Delete documentation "${eventName}"?`)) return;
  try {
    await documentationsDelete(props.sessionToken, id);
    if (selectedDocumentationId.value === id) {
      selectedDocumentationId.value = null;
    }
    await loadRecords();
    if (selectedDocumentationId.value != null) {
      await loadDetail(selectedDocumentationId.value);
    }
    notify("Documentation deleted.");
  } catch (error: any) {
    errorMessage.value = String(error);
  }
}

async function removeExpense(id: number) {
  if (!confirm("Delete this expense deduction?")) return;
  try {
    await documentationExpensesDelete(props.sessionToken, id);
    if (selectedDocumentationId.value != null) {
      await loadRecords();
      await loadDetail(selectedDocumentationId.value);
    }
    notify("Expense deleted.");
  } catch (error: any) {
    errorMessage.value = String(error);
  }
}

function selectRecord(id: number) {
  selectedDocumentationId.value = id;
}

onMounted(load);
</script>

<template>
  <div class="space-y-5">
    <section class="ledger-panel rounded-[26px] px-6 py-6 md:px-8">
      <div class="flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between">
        <div class="max-w-3xl">
          <div class="ledger-eyebrow text-[11px] text-[#8d6f2f]">Book of Accounts - FY 2026</div>
          <h2 class="ledger-heading mt-2 text-4xl font-normal text-[#1f3558] md:text-[4.1rem]">
            Documentation Register
          </h2>
          <p class="mt-3 max-w-2xl text-[15px] leading-7 text-[#6a6b5d]">
            Track registration collections, deduct event expenses, and see the running balance per event.
          </p>
        </div>

        <div class="flex flex-wrap gap-3">
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-xl border border-[#d7c49a] bg-[#fbf7eb] px-4 py-3 text-sm font-semibold text-[#243858] transition hover:bg-[#f4ecd7]"
            @click="exportCurrentCsv"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <path d="M12 3v12" />
              <path d="m8 11 4 4 4-4" />
              <path d="M4 21h16" />
            </svg>
            Export
          </button>
        </div>
      </div>
    </section>

    <section class="grid gap-4 md:grid-cols-3">
      <article class="ledger-card rounded-[4px] p-5">
        <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">Total Collected</div>
        <div class="mt-4 text-[28px] font-semibold tracking-tight text-[var(--ledger-text)]">
          {{ formatPHPFromCents(filteredTotals.collected) }}
        </div>
      </article>
      <article class="ledger-card rounded-[4px] p-5">
        <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">Total Expenses</div>
        <div class="mt-4 text-[28px] font-semibold tracking-tight text-[var(--ledger-red)]">
          {{ formatPHPFromCents(filteredTotals.expenses) }}
        </div>
      </article>
      <article class="ledger-card rounded-[4px] p-5">
        <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">Balance</div>
        <div class="mt-4 text-[28px] font-semibold tracking-tight text-[var(--ledger-green)]">
          {{ formatPHPFromCents(filteredTotals.balance) }}
        </div>
      </article>
    </section>

    <div v-if="errorMessage" class="rounded-2xl border border-[#e3b2a3] bg-[#fff4ef] px-4 py-3 text-[#9d3f27]">
      {{ errorMessage }}
    </div>

    <section class="grid gap-4 xl:grid-cols-[360px_minmax(0,1fr)]">
      <div class="space-y-4">
        <form class="ledger-panel rounded-[18px] px-5 py-5" @submit.prevent="submitDocumentation">
          <div class="ledger-eyebrow text-[11px] text-[#34527f]">Registration Fund Record</div>
          <h3 class="ledger-heading mt-2 text-2xl font-normal text-[#1f3558]">
            New Registration
          </h3>
          <p class="mt-2 text-sm leading-6 text-[#6a6b5d]">
            Record the gross registration collected before expenses are deducted.
          </p>

          <div class="mt-6 grid gap-4">
            <label class="grid gap-2">
              <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">Event Name</span>
              <input
                v-model="formEventName"
                placeholder="e.g. Youth Camp"
                class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)]"
              />
            </label>

            <label class="grid gap-2">
              <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">Event Date</span>
              <input
                v-model="formEventDate"
                type="date"
                class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition focus:border-[color:var(--ledger-gold)]"
              />
            </label>

            <label class="grid gap-2">
              <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">Total Collected (PHP)</span>
              <input
                v-model="formCollected"
                inputmode="decimal"
                placeholder="0.00"
                class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-right text-sm font-medium text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)]"
              />
            </label>

            <label class="grid gap-2">
              <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">Notes</span>
              <textarea
                v-model="formNotes"
                rows="4"
                placeholder="Optional event summary or memo"
                class="min-h-24 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 py-3 text-sm text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)]"
              ></textarea>
            </label>

            <button
              type="submit"
              :disabled="savingDocumentation"
              class="inline-flex h-12 items-center justify-center gap-2 rounded-[4px] bg-[var(--ledger-navy)] px-4 text-sm font-semibold text-white transition hover:bg-[var(--ledger-navy-2)] disabled:cursor-not-allowed disabled:opacity-70"
            >
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M5 12h14" />
                <path d="m12 5 7 7-7 7" />
              </svg>
              <span>{{ savingDocumentation ? "Saving..." : "Save Registration" }}</span>
            </button>
          </div>
        </form>

        <section class="ledger-panel rounded-[18px] px-4 py-4">
          <div class="flex flex-col gap-3">
            <div>
              <h3 class="ledger-heading text-2xl text-[var(--ledger-text)]">
                Registration Events
              </h3>
              <p class="mt-1 text-sm text-[var(--ledger-muted)]">
                Select an event to manage its expense deductions.
              </p>
            </div>

            <label class="flex items-center gap-3 rounded-[12px] border border-[#d7c49a] bg-[#fbf7eb] px-4 py-3">
              <svg class="h-4 w-4 shrink-0 text-[#7a755f]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <circle cx="11" cy="11" r="7" />
                <path d="m20 20-3.5-3.5" />
              </svg>
              <input
                v-model="searchQuery"
                type="text"
                placeholder="Search event, amount or balance..."
                class="w-full bg-transparent text-sm text-[#243858] outline-none placeholder:text-[#7a755f]"
              />
            </label>
          </div>

          <div class="mt-4 space-y-2">
            <button
              v-for="record in filteredRecords"
              :key="record.id"
              type="button"
              class="w-full rounded-[14px] border px-4 py-4 text-left transition"
              :class="selectedDocumentationId === record.id
                ? 'border-[#243858] bg-[#eef3fb]'
                : 'border-[#eadfbd] bg-[#fbf7eb] hover:bg-[#f7f1e0]'"
              @click="selectRecord(record.id)"
            >
              <div class="flex items-start justify-between gap-3">
                <div>
                  <div class="font-semibold text-[#1f3558]">{{ record.event_name }}</div>
                  <div class="mt-1 text-xs text-[#6a6b5d]">{{ formatDateLabel(record.event_date) }}</div>
                </div>
                <button
                  type="button"
                  class="rounded-lg border border-[#d7c49a] bg-white px-2 py-1 text-[11px] font-semibold text-[#243858] transition hover:bg-[#f4ecd7]"
                  @click.stop="removeDocumentation(record.id, record.event_name)"
                >
                  Delete
                </button>
              </div>
              <div class="mt-3 grid grid-cols-3 gap-2 text-xs">
                <div>
                  <div class="text-[var(--ledger-muted)]">Collected</div>
                  <div class="font-mono text-[var(--ledger-text)]">{{ formatPHPFromCents(record.registration_collected_cents) }}</div>
                </div>
                <div>
                  <div class="text-[var(--ledger-muted)]">Expenses</div>
                  <div class="font-mono text-[var(--ledger-red)]">{{ formatPHPFromCents(record.expenses_cents) }}</div>
                </div>
                <div>
                  <div class="text-[var(--ledger-muted)]">Balance</div>
                  <div class="font-mono text-[var(--ledger-green)]">{{ formatPHPFromCents(record.balance_cents) }}</div>
                </div>
              </div>
            </button>

            <div v-if="filteredRecords.length === 0" class="rounded-[14px] border border-[#eadfbd] bg-[#fbf7eb] px-4 py-8 text-center text-sm text-[#6a6b5d]">
              No registration events found.
            </div>
          </div>
        </section>
      </div>

      <section class="ledger-panel overflow-hidden rounded-[18px]">
        <div v-if="selectedRecord" class="space-y-4">
          <div class="border-b border-[color:var(--ledger-line)] px-4 py-4">
            <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
              <div>
                <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">Selected Registration</div>
                <h3 class="ledger-heading mt-2 text-2xl text-[var(--ledger-text)]">
                  {{ selectedRecord.event_name }}
                </h3>
                <p class="mt-1 text-sm text-[var(--ledger-muted)]">
                  {{ formatDateLabel(selectedRecord.event_date) }}
                </p>
              </div>
              <div class="flex flex-wrap gap-2">
                <span class="rounded-full border border-[#9ec8a9] bg-[rgba(241,250,243,0.95)] px-3 py-1 text-xs font-semibold uppercase tracking-[0.2em] text-[var(--ledger-green)]">
                  Active
                </span>
              </div>
            </div>
          </div>

          <div class="grid gap-4 px-4 md:grid-cols-3">
            <article class="ledger-card rounded-[4px] p-5">
              <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">Total Collected</div>
              <div class="mt-4 text-[28px] font-semibold tracking-tight text-[var(--ledger-text)]">
                {{ formatPHPFromCents(selectedRecord.registration_collected_cents) }}
              </div>
            </article>
            <article class="ledger-card rounded-[4px] p-5">
              <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">Total Expenses</div>
              <div class="mt-4 text-[28px] font-semibold tracking-tight text-[var(--ledger-red)]">
                {{ formatPHPFromCents(selectedRecord.expenses_cents) }}
              </div>
            </article>
            <article class="ledger-card rounded-[4px] p-5">
              <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">Balance</div>
              <div class="mt-4 text-[28px] font-semibold tracking-tight text-[var(--ledger-green)]">
                {{ formatPHPFromCents(selectedRecord.balance_cents) }}
              </div>
            </article>
          </div>

          <div class="grid gap-4 px-4 xl:grid-cols-[340px_minmax(0,1fr)]">
            <form class="ledger-card rounded-[4px] p-5" @submit.prevent="submitExpense">
              <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">Expense Deduction</div>
              <h4 class="ledger-heading mt-2 text-2xl text-[var(--ledger-text)]">
                Add Expense
              </h4>

              <div class="mt-6 grid gap-4">
                <label class="grid gap-2">
                  <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">Date</span>
                  <input
                    v-model="expenseDate"
                    type="date"
                    class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition focus:border-[color:var(--ledger-gold)]"
                  />
                </label>

                <label class="grid gap-2">
                  <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">Payee</span>
                  <input
                    v-model="expensePayee"
                    placeholder="Supplier or person paid"
                    class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)]"
                  />
                </label>

                <label class="grid gap-2">
                  <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">Amount (PHP)</span>
                  <input
                    v-model="expenseAmount"
                    inputmode="decimal"
                    placeholder="0.00"
                    class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-right text-sm font-medium text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)]"
                  />
                </label>

                <label class="grid gap-2">
                  <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">Notes</span>
                  <textarea
                    v-model="expenseNotes"
                    rows="3"
                    placeholder="Optional expense note"
                    class="min-h-24 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 py-3 text-sm text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)]"
                  ></textarea>
                </label>

                <button
                  type="submit"
                  :disabled="savingExpense"
                  class="inline-flex h-12 items-center justify-center gap-2 rounded-[4px] bg-[var(--ledger-red)] px-4 text-sm font-semibold text-white transition hover:bg-[#a73d24] disabled:cursor-not-allowed disabled:opacity-70"
                >
                  <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                    <path d="M5 12h14" />
                    <path d="m12 5 7 7-7 7" />
                  </svg>
                  <span>{{ savingExpense ? "Saving..." : "Save Expense" }}</span>
                </button>
              </div>
            </form>

            <section class="ledger-card overflow-hidden rounded-[4px]">
              <div class="flex flex-col gap-3 border-b border-[color:var(--ledger-line)] px-4 py-4 lg:flex-row lg:items-center lg:justify-between">
                <div>
                  <h4 class="ledger-heading text-2xl text-[var(--ledger-text)]">
                    Expense Ledger
                  </h4>
                  <p class="mt-1 text-sm text-[var(--ledger-muted)]">
                    Posted expense deductions for this registration event.
                  </p>
                </div>
                <div class="inline-flex h-11 items-center rounded-[999px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-4 text-sm font-semibold text-[var(--ledger-text)]">
                  {{ selectedExpenses.length }} entries
                </div>
              </div>

              <div v-if="selectedExpenses.length === 0" class="px-4 py-8 text-sm text-[var(--ledger-muted)]">
                No expense deductions recorded yet.
              </div>

              <div v-else class="overflow-x-auto">
                <table class="w-full min-w-[760px] border-separate border-spacing-0">
                  <thead>
                    <tr class="bg-[rgba(240,229,203,0.85)] text-[11px] uppercase tracking-[0.28em] text-[var(--ledger-text)]">
                      <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Date</th>
                      <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Payee</th>
                      <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Notes</th>
                      <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-right font-semibold">Amount</th>
                      <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-right font-semibold">Action</th>
                    </tr>
                  </thead>

                  <tbody>
                    <tr
                      v-for="expense in selectedExpenses"
                      :key="expense.id"
                      class="group bg-[rgba(251,247,235,0.92)]"
                    >
                      <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 whitespace-nowrap text-sm text-[var(--ledger-text)]">
                        {{ formatDateLabel(expense.spent_at) }}
                      </td>
                      <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-sm text-[var(--ledger-text)]">
                        {{ expense.payee || "—" }}
                      </td>
                      <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-sm text-[var(--ledger-text)]">
                        {{ expense.notes || "—" }}
                      </td>
                      <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-right text-sm font-semibold text-[var(--ledger-red)]">
                        {{ formatPHPFromCents(expense.amount_cents) }}
                      </td>
                      <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-right" @click.stop>
                        <button
                          type="button"
                          class="inline-flex items-center gap-1 rounded-[999px] border border-transparent px-3 py-1 text-xs font-semibold text-[var(--ledger-muted)] opacity-0 transition group-hover:opacity-100 hover:border-rose-200 hover:bg-rose-50 hover:text-rose-700"
                          @click="removeExpense(expense.id)"
                        >
                          <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                            <path d="M3 6h18" />
                            <path d="M8 6V4h8v2" />
                            <path d="M6 6l1 14h10l1-14" />
                            <path d="M10 11v5" />
                            <path d="M14 11v5" />
                          </svg>
                          <span>Delete</span>
                        </button>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </section>
          </div>
        </div>

        <div v-else class="px-4 py-8 text-sm text-[var(--ledger-muted)]">
          Select a registration event to review its balance and expense deductions.
        </div>
      </section>
    </section>
  </div>
</template>
