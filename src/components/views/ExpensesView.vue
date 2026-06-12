<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { categoriesCreate, categoriesList, expensesCreate, expensesDelete, expensesList, projectsList } from "../../lib/api";
import { notify } from "../../lib/feedback";
import { centsFromPesos, formatPHPFromCents } from "../../lib/money";
import type { Category, Expense, Project } from "../../lib/types";

const props = defineProps<{ sessionToken: string }>();

const loading = ref(true);
const errorMessage = ref<string | null>(null);
const items = ref<Expense[]>([]);
const categories = ref<Category[]>([]);
const projects = ref<Project[]>([]);

const filterFrom = ref<string>("");
const filterTo = ref<string>("");
const filterProjectId = ref<string>("");

const formDate = ref<string>(new Date().toISOString().slice(0, 10));
const formAmount = ref<string>("");
const formCategoryId = ref<string>("");
const newCategoryName = ref<string>("");
const formProjectId = ref<string>("");
const formPayee = ref<string>("");
const formNotes = ref<string>("");

const categoriesById = computed(() => new Map(categories.value.map((c) => [c.id, c])));
const projectsById = computed(() => new Map(projects.value.map((p) => [p.id, p])));

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    categories.value = await categoriesList(props.sessionToken);
    projects.value = await projectsList(props.sessionToken);
    items.value = await expensesList(props.sessionToken, {
      from: filterFrom.value || null,
      to: filterTo.value || null,
      project_id: filterProjectId.value ? Number(filterProjectId.value) : null,
    });
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function createExpense() {
  errorMessage.value = null;
  if (!confirm("Save this expense record?")) return;
  try {
    const amountCents = centsFromPesos(formAmount.value);
    await expensesCreate(props.sessionToken, {
      spent_at: formDate.value,
      amount_cents: amountCents,
      category_id: formCategoryId.value ? Number(formCategoryId.value) : null,
      payee: formPayee.value || null,
      notes: formNotes.value || null,
      project_id: formProjectId.value ? Number(formProjectId.value) : null,
    });
    formAmount.value = "";
    formPayee.value = "";
    formNotes.value = "";
    await load();
    notify("Expense saved.");
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

async function addCategory() {
  errorMessage.value = null;
  const name = newCategoryName.value.trim();
  if (!name) return;
  if (!confirm(`Add category "${name}"?`)) return;
  try {
    await categoriesCreate(props.sessionToken, { name });
    newCategoryName.value = "";
    await load();
    notify(`Category "${name}" added.`);
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

async function removeExpense(id: number) {
  if (!confirm("Delete this expense?")) return;
  try {
    await expensesDelete(props.sessionToken, id);
    await load();
    notify("Expense deleted.");
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

onMounted(load);
</script>

<template>
  <div class="space-y-6">
    <div v-if="errorMessage" class="rounded-xl border border-rose-500/40 bg-rose-500/10 p-4 text-rose-200">
      {{ errorMessage }}
    </div>

    <div class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
      <div class="font-semibold">Add Expense</div>
      <div class="mt-4 grid grid-cols-1 md:grid-cols-6 gap-3">
        <div class="md:col-span-1">
          <label class="block text-xs text-slate-400 mb-1">Date</label>
          <input v-model="formDate" type="date" class="w-full rounded-xl border border-slate-700 bg-slate-950/60 px-3 py-2" />
        </div>
        <div class="md:col-span-1">
          <label class="block text-xs text-slate-400 mb-1">Amount (PHP)</label>
          <input v-model="formAmount" inputmode="decimal" placeholder="0.00" class="w-full rounded-xl border border-slate-700 bg-slate-950/60 px-3 py-2" />
        </div>
        <div class="md:col-span-1">
          <label class="block text-xs text-slate-400 mb-1">Category</label>
          <select v-model="formCategoryId" class="w-full rounded-xl border border-slate-700 bg-slate-950/60 px-3 py-2">
            <option value="">(none)</option>
            <option v-for="c in categories" :key="c.id" :value="String(c.id)">{{ c.name }}</option>
          </select>
          <div class="mt-2 flex gap-2">
            <input
              v-model="newCategoryName"
              placeholder="Quick add category…"
              class="flex-1 rounded-lg border border-slate-700 bg-slate-950/60 px-3 py-2 text-sm"
            />
            <button
              type="button"
              :disabled="!newCategoryName.trim()"
              class="rounded-lg bg-slate-800 hover:bg-slate-700 disabled:opacity-60 px-3 py-2 text-sm font-semibold"
              @click="addCategory"
            >
              Add
            </button>
          </div>
        </div>
        <div class="md:col-span-1">
          <label class="block text-xs text-slate-400 mb-1">Project</label>
          <select v-model="formProjectId" class="w-full rounded-xl border border-slate-700 bg-slate-950/60 px-3 py-2">
            <option value="">(none)</option>
            <option v-for="p in projects" :key="p.id" :value="String(p.id)">{{ p.name }}</option>
          </select>
        </div>
        <div class="md:col-span-1">
          <label class="block text-xs text-slate-400 mb-1">Payee</label>
          <input v-model="formPayee" placeholder="optional" class="w-full rounded-xl border border-slate-700 bg-slate-950/60 px-3 py-2" />
        </div>
        <div class="md:col-span-1">
          <label class="block text-xs text-slate-400 mb-1">Notes</label>
          <input v-model="formNotes" placeholder="optional" class="w-full rounded-xl border border-slate-700 bg-slate-950/60 px-3 py-2" />
        </div>
      </div>
      <div class="mt-4">
        <button class="rounded-xl bg-indigo-600 hover:bg-indigo-500 px-4 py-2 font-semibold" @click="createExpense">
          Save Expense
        </button>
      </div>
    </div>

    <div class="rounded-2xl border border-slate-800 bg-slate-900/40 overflow-hidden">
      <div class="p-5 flex items-center justify-between gap-4">
        <div>
          <div class="font-semibold">Expenses</div>
          <div class="text-sm text-slate-400">Latest 500 records</div>
        </div>
        <div class="flex flex-wrap items-end gap-2">
          <div>
            <div class="text-xs text-slate-400 mb-1">From</div>
            <input v-model="filterFrom" type="date" class="rounded-lg border border-slate-700 bg-slate-950/60 px-3 py-2 text-sm" />
          </div>
          <div>
            <div class="text-xs text-slate-400 mb-1">To</div>
            <input v-model="filterTo" type="date" class="rounded-lg border border-slate-700 bg-slate-950/60 px-3 py-2 text-sm" />
          </div>
          <div>
            <div class="text-xs text-slate-400 mb-1">Project</div>
            <select v-model="filterProjectId" class="rounded-lg border border-slate-700 bg-slate-950/60 px-3 py-2 text-sm">
              <option value="">All</option>
              <option v-for="p in projects" :key="p.id" :value="String(p.id)">{{ p.name }}</option>
            </select>
          </div>
          <button class="rounded-lg bg-slate-800 hover:bg-slate-700 px-3 py-2 text-sm font-semibold" @click="load">Apply</button>
        </div>
      </div>

      <div class="border-t border-slate-800">
        <table class="w-full text-sm">
          <thead class="bg-slate-950/40 text-slate-300">
            <tr>
              <th class="text-left p-3 font-medium">Date</th>
              <th class="text-left p-3 font-medium">Category</th>
              <th class="text-left p-3 font-medium">Project</th>
              <th class="text-right p-3 font-medium">Amount</th>
              <th class="text-left p-3 font-medium">Payee</th>
              <th class="text-left p-3 font-medium">Notes</th>
              <th class="text-right p-3 font-medium">Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-if="loading" class="border-t border-slate-800">
              <td class="p-3 text-slate-400" colspan="7">Loading…</td>
            </tr>
            <tr v-for="e in items" :key="e.id" class="border-t border-slate-800">
              <td class="p-3">{{ e.spent_at }}</td>
              <td class="p-3">{{ e.category_id ? categoriesById.get(e.category_id)?.name || `#${e.category_id}` : "-" }}</td>
              <td class="p-3">{{ e.project_id ? projectsById.get(e.project_id)?.name || `#${e.project_id}` : "-" }}</td>
              <td class="p-3 text-right font-semibold">{{ formatPHPFromCents(e.amount_cents) }}</td>
              <td class="p-3 text-slate-300">{{ e.payee || "" }}</td>
              <td class="p-3 text-slate-300">{{ e.notes || "" }}</td>
              <td class="p-3 text-right">
                <button class="rounded-lg bg-rose-600 hover:bg-rose-500 px-3 py-1.5 text-xs font-semibold" @click="removeExpense(e.id)">
                  Delete
                </button>
              </td>
            </tr>
            <tr v-if="!loading && items.length === 0" class="border-t border-slate-800">
              <td class="p-3 text-slate-400" colspan="7">No expenses found.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
