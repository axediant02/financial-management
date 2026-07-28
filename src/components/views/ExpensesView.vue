<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import {
  expensesCreate,
  expensesDelete,
  expensesList,
  exportCsv,
  categoriesList,
  projectsList,
} from "../../lib/api";
import { notify } from "../../lib/feedback";
import { centsFromPesos, formatPHPFromCents } from "../../lib/money";
import type { Category, Expense, Project } from "../../lib/types";

const props = defineProps<{ sessionToken: string }>();

const loading = ref(true);
const saving = ref(false);
const errorMessage = ref<string | null>(null);

const items = ref<Expense[]>([]);
const categories = ref<Category[]>([]);
const projects = ref<Project[]>([]);

const filterQuery = ref("");
const formDate = ref(getLocalDateString());
const formPayee = ref("");
const formCategoryId = ref("");
const formProjectId = ref("");
const formVoucherNo = ref("DV-2042");
const formAmount = ref("");
const formNotes = ref("");

const categoriesById = computed(() => new Map(categories.value.map((category) => [category.id, category])));
const projectsById = computed(() => new Map(projects.value.map((project) => [project.id, project])));

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

function formatVoucherNo(value: number) {
  return `DV-${String(value).padStart(4, "0")}`;
}

function normalize(value: string) {
  return value.trim().toLowerCase();
}

function parseExpenseMeta(notes?: string | null) {
  if (!notes) {
    return { voucherNo: "", memo: "" };
  }

  const voucherMatch = notes.match(/(?:voucher)\s*:\s*([^|]+)/i);
  const memoMatch = notes.match(/(?:memo)\s*:\s*([^|]+)/i);

  return {
    voucherNo: voucherMatch?.[1]?.trim() || "",
    memo: memoMatch?.[1]?.trim() || "",
  };
}

function serializeExpenseMeta(voucherNo: string, memo: string) {
  const parts: string[] = [];
  const cleanVoucher = voucherNo.trim();
  const cleanMemo = memo.trim();

  if (cleanVoucher) parts.push(`Voucher: ${cleanVoucher}`);
  if (cleanMemo) parts.push(`Memo: ${cleanMemo}`);
  return parts.join(" | ") || null;
}

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    const [categoryRows, projectRows, expenseRows] = await Promise.all([
      categoriesList(props.sessionToken),
      projectsList(props.sessionToken),
      expensesList(props.sessionToken, {
        from: null,
        to: null,
        project_id: null,
      }),
    ]);

    categories.value = categoryRows;
    projects.value = projectRows;
    items.value = expenseRows;
    if (expenseRows.length > 0) {
      formVoucherNo.value = formatVoucherNo(expenseRows[0].id + 1);
    }
  } catch (error: any) {
    errorMessage.value = String(error);
  } finally {
    loading.value = false;
  }
}

async function exportCurrentCsv() {
  const dest = await save({
    defaultPath: "disbursements-journal.csv",
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!dest) return;

  await exportCsv(props.sessionToken, {
    kind: "expenses",
    filter: { from: null, to: null, project_id: null },
    dest_path: dest,
  });
  notify("Expenses CSV exported.");
}

async function createExpense() {
  errorMessage.value = null;
  saving.value = true;

  try {
    const amountCents = centsFromPesos(formAmount.value);
    if (amountCents <= 0) {
      notify("Expense amount must be greater than 0.");
      return;
    }

    if (!confirm("Post this disbursement voucher?")) return;

    await expensesCreate(props.sessionToken, {
      spent_at: formDate.value,
      amount_cents: amountCents,
      category_id: formCategoryId.value ? Number(formCategoryId.value) : null,
      payee: formPayee.value.trim() || null,
      notes: serializeExpenseMeta(formVoucherNo.value, formNotes.value),
      project_id: formProjectId.value ? Number(formProjectId.value) : null,
    });

    formPayee.value = "";
    formAmount.value = "";
    formNotes.value = "";

    await load();
    notify("Expense posted.");
  } catch (error: any) {
    errorMessage.value = String(error);
  } finally {
    saving.value = false;
  }
}

async function removeExpense(id: number) {
  if (!confirm("Delete this expense voucher?")) return;
  try {
    await expensesDelete(props.sessionToken, id);
    await load();
    notify("Expense deleted.");
  } catch (error: any) {
    errorMessage.value = String(error);
  }
}

const expenseRows = computed(() =>
  items.value
    .slice()
    .sort((a, b) => b.spent_at.localeCompare(a.spent_at) || b.id - a.id)
    .map((item) => {
      const category = item.category_id ? categoriesById.value.get(item.category_id)?.name?.trim() : null;
      const project = item.project_id ? projectsById.value.get(item.project_id)?.name?.trim() : null;
      const meta = parseExpenseMeta(item.notes);
      return {
        ...item,
        category_name: category || (item.category_id ? `#${item.category_id}` : "Uncategorized"),
        project_name: project || (item.project_id ? `#${item.project_id}` : "General Fund (unassigned)"),
        voucher_no: meta.voucherNo || formatVoucherNo(item.id),
        memo: meta.memo,
      };
    }),
);

const filteredRows = computed(() => {
  const query = normalize(filterQuery.value);
  if (!query) return expenseRows.value;

  return expenseRows.value.filter((item) => {
    const haystack = [
      item.spent_at,
      formatDateLabel(item.spent_at),
      item.payee || "",
      item.category_name,
      item.project_name,
      item.voucher_no,
      item.memo,
      formatPHPFromCents(item.amount_cents),
    ]
      .join(" ")
      .toLowerCase();
    return haystack.includes(query);
  });
});

const totalDisbursements = computed(() =>
  filteredRows.value.reduce((sum, item) => sum + item.amount_cents, 0),
);

const breakdownRows = computed(() => {
  const totals = new Map<string, number>();

  for (const item of expenseRows.value) {
    const key = item.category_name;
    totals.set(key, (totals.get(key) || 0) + item.amount_cents);
  }

  const rows = [...totals.entries()]
    .map(([name, amountCents]) => ({ name, amountCents }))
    .sort((a, b) => b.amountCents - a.amountCents)
    .slice(0, 4);

  const max = rows[0]?.amountCents || 0;
  return rows.map((row) => ({
    ...row,
    width: max > 0 ? Math.max(12, Math.round((row.amountCents / max) * 100)) : 12,
  }));
});

const displayedCount = computed(() => filteredRows.value.length);

onMounted(load);
</script>

<template>
  <div class="space-y-5 text-[var(--ledger-text)]">
    <section class="ledger-panel overflow-hidden rounded-[26px]">
      <div class="flex flex-col gap-5 border-b border-[color:var(--ledger-line)] px-6 py-5 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <p class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">
            BOOK OF ACCOUNTS · FY 2026
          </p>
          <h2 class="ledger-heading mt-2 text-4xl text-[var(--ledger-text)]">
            Disbursements Journal
          </h2>
          <p class="mt-3 max-w-2xl text-sm text-[var(--ledger-muted)]">
            Issue vouchers and review all money released from the funds.
          </p>
        </div>

        <button
          type="button"
          class="inline-flex items-center gap-2 rounded-[12px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-4 py-3 text-sm font-semibold text-[var(--ledger-text)] transition hover:bg-[rgba(255,255,255,0.95)]"
          @click="exportCurrentCsv"
        >
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <path d="M12 3v12" />
            <path d="m8 11 4 4 4-4" />
            <path d="M4 21h16" />
          </svg>
          <span>Export CSV</span>
        </button>
      </div>

      <div class="grid gap-4 px-4 py-4 lg:grid-cols-[340px_minmax(0,1fr)]">
        <form class="ledger-card rounded-[4px] p-5" @submit.prevent="createExpense">
          <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">
            DISBURSEMENT VOUCHER
          </div>
          <h3 class="ledger-heading mt-2 text-2xl text-[var(--ledger-text)]">
            New Expense
          </h3>

          <div class="mt-6 grid gap-4">
            <label class="grid gap-2">
              <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                Date
              </span>
              <input
                v-model="formDate"
                type="date"
                class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition focus:border-[color:var(--ledger-gold)]"
              />
            </label>

            <label class="grid gap-2">
              <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                Payee
              </span>
              <input
                v-model="formPayee"
                placeholder="Supplier or person paid"
                class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)]"
              />
            </label>

            <label class="grid gap-2">
              <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                Category
              </span>
              <select
                v-model="formCategoryId"
                class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition focus:border-[color:var(--ledger-gold)]"
              >
                <option value="">Uncategorized</option>
                <option v-for="category in categories" :key="category.id" :value="String(category.id)">
                  {{ category.name }}
                </option>
              </select>
            </label>

            <label class="grid gap-2">
              <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                Charged To
              </span>
              <select
                v-model="formProjectId"
                class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition focus:border-[color:var(--ledger-gold)]"
              >
                <option value="">General Fund (unassigned)</option>
                <option v-for="project in projects" :key="project.id" :value="String(project.id)">
                  {{ project.name }}
                </option>
              </select>
            </label>

            <div class="grid gap-3 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
              <label class="grid min-w-0 gap-2">
                <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                  Voucher No.
                </span>
                <input
                  v-model="formVoucherNo"
                  class="h-12 w-full min-w-0 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition focus:border-[color:var(--ledger-gold)]"
                />
              </label>

              <label class="grid min-w-0 gap-2">
                <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                  Amount (PHP)
                </span>
                <input
                  v-model="formAmount"
                  inputmode="decimal"
                  placeholder="0.00"
                  class="h-12 w-full min-w-0 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-right text-sm font-medium text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)]"
                />
              </label>
            </div>

            <label class="grid gap-2">
              <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                Notes
              </span>
              <textarea
                v-model="formNotes"
                rows="3"
                placeholder="Optional memo or expense note"
                class="min-h-24 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 py-3 text-sm text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)]"
              ></textarea>
            </label>

            <button
              type="submit"
              :disabled="saving"
              class="inline-flex h-12 items-center justify-center gap-2 rounded-[4px] bg-[var(--ledger-red)] px-4 text-sm font-semibold text-white transition hover:bg-[#a73d24] disabled:cursor-not-allowed disabled:opacity-70"
            >
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M5 12h14" />
                <path d="m12 5 7 7-7 7" />
              </svg>
              <span>{{ saving ? "Posting..." : "Post Disbursement" }}</span>
            </button>
          </div>
        </form>

        <div class="grid gap-4">
          <section class="ledger-card overflow-hidden rounded-[4px]">
            <div class="px-4 py-4">
              <div class="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
                <div>
                  <h3 class="ledger-heading text-2xl text-[var(--ledger-text)]">
                    Expense History
                  </h3>
                </div>

                <div class="flex w-full flex-col gap-3 sm:w-auto sm:flex-row sm:items-center">
                  <div class="relative w-full sm:w-[18rem]">
                    <svg
                      class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--ledger-muted)]"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      aria-hidden="true"
                    >
                      <circle cx="11" cy="11" r="7" />
                      <path d="m20 20-3.5-3.5" />
                    </svg>
                    <input
                      v-model="filterQuery"
                      placeholder="Filter payee, category or voucher"
                      class="h-11 w-full rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-10 text-sm text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)]"
                    />
                  </div>

                  <div class="inline-flex h-11 items-center rounded-[999px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-4 text-sm font-semibold text-[var(--ledger-text)]">
                    {{ displayedCount }} entries
                  </div>
                </div>
              </div>
            </div>

            <div v-if="errorMessage" class="mx-4 mt-2 rounded-[4px] border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
              {{ errorMessage }}
            </div>

            <div v-else-if="loading" class="px-4 py-8 text-sm text-[var(--ledger-muted)]">
              Loading expense records...
            </div>

            <div v-else class="overflow-x-auto">
              <table class="w-full min-w-[900px] border-separate border-spacing-0">
                <thead>
                  <tr class="bg-[rgba(240,229,203,0.85)] text-[11px] uppercase tracking-[0.28em] text-[var(--ledger-text)]">
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Date</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Payee</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Category</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Charged To</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Voucher</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-right font-semibold">Amount</th>
                    <th class="border-b border-[color:var(--ledger-line)] px-3 py-3 text-right font-semibold"></th>
                  </tr>
                </thead>

                <tbody>
                  <tr v-for="item in filteredRows" :key="item.id" class="group">
                    <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 whitespace-nowrap text-sm text-[var(--ledger-text)]">
                      {{ formatDateLabel(item.spent_at) }}
                    </td>
                    <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-sm text-[var(--ledger-text)]">
                      {{ item.payee || "—" }}
                    </td>
                    <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-sm text-[var(--ledger-text)]">
                      {{ item.category_name }}
                    </td>
                    <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-sm text-[var(--ledger-text)]">
                      {{ item.project_name }}
                    </td>
                    <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 whitespace-nowrap text-sm text-[var(--ledger-text)]">
                      {{ item.voucher_no }}
                    </td>
                    <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-right text-sm font-semibold tabular-nums text-[var(--ledger-red)]">
                      {{ formatPHPFromCents(item.amount_cents) }}
                    </td>
                    <td class="border-b border-[color:rgba(215,196,154,0.7)] px-3 py-3 text-right">
                      <button
                        type="button"
                        class="inline-flex items-center justify-center rounded-[999px] border border-transparent p-2 text-[var(--ledger-muted)] opacity-0 transition group-hover:opacity-100 hover:border-rose-200 hover:bg-rose-50 hover:text-rose-700"
                        title="Delete expense"
                        @click="removeExpense(item.id)"
                      >
                        <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                          <path d="M3 6h18" />
                          <path d="M8 6V4h8v2" />
                          <path d="M6 6l1 14h10l1-14" />
                          <path d="M10 11v5" />
                          <path d="M14 11v5" />
                        </svg>
                      </button>
                    </td>
                  </tr>

                  <tr v-if="filteredRows.length === 0">
                    <td colspan="7" class="px-4 py-10 text-center text-sm text-[var(--ledger-muted)]">
                      No expense records found.
                    </td>
                  </tr>
                </tbody>

                <tfoot>
                  <tr class="bg-[rgba(244,237,220,0.9)]">
                    <th
                      colspan="5"
                      class="border-t border-[color:var(--ledger-line)] px-4 py-4 text-left text-[11px] uppercase tracking-[0.28em] text-[var(--ledger-text)]"
                    >
                      Total Disbursements
                    </th>
                    <td class="border-t border-[color:var(--ledger-line)] px-4 py-4 text-right text-sm font-semibold text-[var(--ledger-text)]">
                      {{ formatPHPFromCents(totalDisbursements) }}
                    </td>
                    <td class="border-t border-[color:var(--ledger-line)] px-3 py-4"></td>
                  </tr>
                </tfoot>
              </table>
            </div>
          </section>

          <section class="ledger-card rounded-[4px] p-5">
            <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">
              BREAKDOWN BY CATEGORY
            </div>

            <div class="mt-4 grid gap-4">
              <div v-for="row in breakdownRows" :key="row.name" class="grid gap-1.5">
                <div class="flex items-center justify-between gap-4 text-sm">
                  <span class="text-[var(--ledger-text)]">{{ row.name }}</span>
                  <span class="font-mono text-[var(--ledger-text)]">{{ formatPHPFromCents(row.amountCents) }}</span>
                </div>
                <div class="h-2 rounded-full bg-[rgba(215,196,154,0.35)]">
                  <div
                    class="h-2 rounded-full bg-[var(--ledger-red)]"
                    :style="{ width: `${row.width}%` }"
                  />
                </div>
              </div>

              <div v-if="breakdownRows.length === 0" class="text-sm text-[var(--ledger-muted)]">
                No expenses recorded yet.
              </div>
            </div>
          </section>
        </div>
      </div>
    </section>
  </div>
</template>
