<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import {
  donationsCreate,
  donationsDelete,
  donationsList,
  donorsCreate,
  donorsList,
  exportCsv,
  projectsList,
} from "../../lib/api";
import { notify } from "../../lib/feedback";
import { centsFromPesos, formatPHPFromCents } from "../../lib/money";
import type { Donation, Donor, Project } from "../../lib/types";

type JournalMeta = {
  method: string;
  reference: string;
};

type JournalEntry = {
  id: number;
  donated_at: string;
  amount_cents: number;
  donor_name: string;
  project_name: string;
  method: string;
  reference: string;
  anonymous: boolean;
};

const props = defineProps<{ sessionToken: string }>();

const loading = ref(true);
const saving = ref(false);
const errorMessage = ref<string | null>(null);
const lastSyncedAt = ref<string | null>(null);

const donations = ref<Donation[]>([]);
const donors = ref<Donor[]>([]);
const projects = ref<Project[]>([]);

const searchQuery = ref("");
const formDate = ref(getLocalDateString());
const formDonorName = ref("");
const formProjectId = ref("");
const formMethod = ref("Cash");
const formReference = ref("");
const formAmount = ref("");
const donorInput = ref<HTMLInputElement | null>(null);

const donorsById = computed(() => new Map(donors.value.map((donor) => [donor.id, donor])));
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
    day: "numeric",
    year: "numeric",
  }).format(parsed);
}

function formatFooterDate(value: string) {
  const parsed = new Date(`${value}T00:00:00`);
  if (Number.isNaN(parsed.getTime())) return value;
  return new Intl.DateTimeFormat("en-GB", {
    day: "2-digit",
    month: "short",
    year: "numeric",
  }).format(parsed);
}

function normalize(value: string) {
  return value.trim().toLowerCase();
}

function parseJournalMeta(notes?: string | null): JournalMeta {
  const fallback: JournalMeta = {
    method: "Cash",
    reference: "",
  };

  if (!notes) return fallback;

  const methodMatch = notes.match(/(?:method)\s*:\s*([^|]+)/i);
  const referenceMatch = notes.match(/(?:ref(?:erence)?)\s*:\s*([^|]+)/i);

  return {
    method: methodMatch?.[1]?.trim() || fallback.method,
    reference: referenceMatch?.[1]?.trim() || fallback.reference,
  };
}

function serializeJournalMeta(method: string, reference: string) {
  const parts: string[] = [];
  const cleanMethod = method.trim();
  const cleanReference = reference.trim();

  if (cleanMethod) parts.push(`Method: ${cleanMethod}`);
  if (cleanReference) parts.push(`Ref: ${cleanReference}`);
  return parts.join(" | ") || null;
}

const journalEntries = computed<JournalEntry[]>(() =>
  donations.value
    .slice()
    .sort((a, b) => {
      const byDate = b.donated_at.localeCompare(a.donated_at);
      return byDate !== 0 ? byDate : b.id - a.id;
    })
    .map((donation) => {
      const donor = donation.anonymous
        ? "Anonymous"
        : donation.donor_id
          ? donorsById.value.get(donation.donor_id)?.name?.trim() || `#${donation.donor_id}`
          : "Anonymous";
      const project = donation.project_id
        ? projectsById.value.get(donation.project_id)?.name?.trim() || `#${donation.project_id}`
        : "General Fund (unassigned)";
      const meta = parseJournalMeta(donation.notes);
      return {
        id: donation.id,
        donated_at: donation.donated_at,
        amount_cents: donation.amount_cents,
        donor_name: donor,
        project_name: project,
        method: meta.method,
        reference: meta.reference,
        anonymous: donation.anonymous,
      };
    }),
);

const filteredEntries = computed(() => {
  const query = normalize(searchQuery.value);
  if (!query) return journalEntries.value;

  return journalEntries.value.filter((entry) => {
    const haystack = [
      entry.donated_at,
      formatDateLabel(entry.donated_at),
      entry.donor_name,
      entry.project_name,
      entry.method,
      entry.reference,
      formatPHPFromCents(entry.amount_cents),
    ]
      .join(" ")
      .toLowerCase();
    return haystack.includes(query);
  });
});

const totalContributions = computed(() =>
  filteredEntries.value.reduce((sum, entry) => sum + entry.amount_cents, 0),
);

const entryCount = computed(() => filteredEntries.value.length);

const latestRecordedDate = computed(() => {
  const latest = journalEntries.value[0];
  return latest?.donated_at || getLocalDateString();
});

const lastBalanceLabel = computed(() => formatFooterDate(latestRecordedDate.value));

function clearForm(keepIdentity = true) {
  formAmount.value = "";
  formReference.value = "";
  if (!keepIdentity) {
    formDonorName.value = "";
    formProjectId.value = "";
    formMethod.value = "Cash";
  }
}

function focusDonorField() {
  donorInput.value?.focus();
}

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    const [donorRows, projectRows, donationRows] = await Promise.all([
      donorsList(props.sessionToken),
      projectsList(props.sessionToken),
      donationsList(props.sessionToken, { from: null, to: null, project_id: null }),
    ]);
    donors.value = donorRows;
    projects.value = projectRows;
    donations.value = donationRows;
    lastSyncedAt.value = new Date().toLocaleString();
  } catch (error: any) {
    errorMessage.value = String(error);
  } finally {
    loading.value = false;
  }
}

async function exportCurrentCsv() {
  const dest = await save({
    defaultPath: "contributions-journal.csv",
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!dest) return;

  await exportCsv(props.sessionToken, {
    kind: "donations",
    filter: { from: null, to: null, project_id: null },
    dest_path: dest,
  });
  notify("Contribution CSV exported.");
}

async function submitContribution() {
  errorMessage.value = null;
  saving.value = true;

  try {
    const amountCents = centsFromPesos(formAmount.value);
    if (amountCents <= 0) {
      notify("Contribution amount must be greater than 0.");
      return;
    }

    const donorName = formDonorName.value.trim();
    const anonymous = donorName.length === 0;
    let donorId: number | null = null;

    if (!anonymous) {
      const existing = donors.value.find((donor) => normalize(donor.name) === normalize(donorName));
      if (existing) {
        donorId = existing.id;
      } else {
        const created = await donorsCreate(props.sessionToken, { name: donorName, notes: null });
        donorId = created.id;
        donors.value = [
          ...donors.value,
          {
            id: created.id,
            name: donorName,
            notes: null,
            created_at: new Date().toISOString(),
          },
        ];
      }
    }

    await donationsCreate(props.sessionToken, {
      donated_at: formDate.value,
      amount_cents: amountCents,
      donor_id: anonymous ? null : donorId,
      anonymous,
      notes: serializeJournalMeta(formMethod.value, formReference.value),
      project_id: formProjectId.value ? Number(formProjectId.value) : null,
    });

    await load();
    clearForm(true);
    notify("Contribution posted.");
  } catch (error: any) {
    errorMessage.value = String(error);
  } finally {
    saving.value = false;
  }
}

async function deleteEntry(id: number) {
  if (!confirm("Delete this contribution record?")) return;
  try {
    await donationsDelete(props.sessionToken, id);
    await load();
    notify("Contribution deleted.");
  } catch (error: any) {
    errorMessage.value = String(error);
  }
}

function moneyClass(amountCents: number) {
  return amountCents >= 0 ? "text-[var(--ledger-green)]" : "text-[var(--ledger-red)]";
}

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
            Contributions Journal
          </h2>
          <p class="mt-3 max-w-2xl text-sm text-[var(--ledger-muted)]">
            Post new receipts and review the full contribution history.
          </p>
        </div>

        <button
          type="button"
          class="inline-flex items-center gap-2 rounded-[12px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-4 py-3 text-sm font-semibold text-[var(--ledger-text)] transition hover:bg-[rgba(255,255,255,0.95)]"
          @click="exportCurrentCsv"
        >
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <path d="M12 3v12" />
            <path d="m7 10 5 5 5-5" />
            <path d="M5 20h14" />
          </svg>
          <span>Export CSV</span>
        </button>
      </div>

      <div class="grid gap-4 px-4 py-4 lg:grid-cols-[340px_minmax(0,1fr)]">
        <form
          class="ledger-card rounded-[4px] p-5"
          @submit.prevent="submitContribution"
        >
          <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">
            OFFICIAL RECEIPT ENTRY
          </div>
          <h3 class="ledger-heading mt-2 text-2xl text-[var(--ledger-text)]">
            New Contribution
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
                Donor
              </span>
              <input
                v-model="formDonorName"
                ref="donorInput"
                list="contribution-donors"
                placeholder="Name of contributor"
                class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)]"
              />
              <datalist id="contribution-donors">
                <option v-for="donor in donors" :key="donor.id" :value="donor.name" />
              </datalist>
            </label>

            <label class="grid gap-2">
              <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                Project
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
                  Method
                </span>
                <select
                  v-model="formMethod"
                  class="h-12 w-full min-w-0 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition focus:border-[color:var(--ledger-gold)]"
                >
                  <option>Cash</option>
                  <option>GCash</option>
                  <option>Bank</option>
                  <option>Check</option>
                </select>
              </label>

              <label class="grid min-w-0 gap-2">
                <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                  OR / REF
                </span>
                <input
                  v-model="formReference"
                  placeholder="OR-10232"
                  class="h-12 w-full min-w-0 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-sm text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)]"
                />
              </label>
            </div>

            <label class="grid gap-2">
              <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                Amount (PHP)
              </span>
              <input
                v-model="formAmount"
                inputmode="decimal"
                placeholder="0.00"
                class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 text-right text-sm font-medium text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)]"
              />
            </label>

            <button
              type="submit"
              :disabled="saving"
              class="inline-flex h-12 items-center justify-center gap-2 rounded-[4px] bg-[var(--ledger-navy)] px-4 text-sm font-semibold text-white transition hover:bg-[var(--ledger-navy-2)] disabled:cursor-not-allowed disabled:opacity-70"
            >
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M5 12h14" />
                <path d="m12 5 7 7-7 7" />
              </svg>
              <span>{{ saving ? "Posting..." : "Post Contribution" }}</span>
            </button>

            <p class="text-center text-xs leading-5 text-[var(--ledger-muted)]">
              Saved to the local database. Press Enter to post and start a new line.
            </p>
          </div>
        </form>

        <section class="ledger-card overflow-hidden rounded-[4px]">
          <div class="flex flex-col gap-3 border-b border-[color:var(--ledger-line)] px-4 py-4 lg:flex-row lg:items-center lg:justify-between">
            <div>
              <h3 class="ledger-heading text-2xl text-[var(--ledger-text)]">
                Contribution History
              </h3>
              <p class="mt-1 text-sm text-[var(--ledger-muted)]">
                Review the full receipt log, filtered by donor, project, or OR number.
              </p>
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
                  v-model="searchQuery"
                  placeholder="Filter donor, project or OR no."
                  class="h-11 w-full rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-10 text-sm text-[var(--ledger-text)] outline-none transition placeholder:text-[var(--ledger-muted)] focus:border-[color:var(--ledger-gold)]"
                />
              </div>

              <div class="inline-flex h-11 items-center rounded-[999px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-4 text-sm font-semibold text-[var(--ledger-text)]">
                {{ entryCount }} entries
              </div>
            </div>
          </div>

          <div v-if="errorMessage" class="mx-4 mt-4 rounded-[4px] border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
            {{ errorMessage }}
          </div>

          <div v-else-if="loading" class="px-4 py-8 text-sm text-[var(--ledger-muted)]">
            Loading contribution records...
          </div>

          <div v-else class="overflow-x-auto">
            <table class="w-full min-w-[860px] border-separate border-spacing-0">
              <thead>
                <tr class="bg-[rgba(240,229,203,0.85)] text-[11px] uppercase tracking-[0.28em] text-[var(--ledger-text)]">
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Date</th>
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Donor</th>
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Project</th>
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Method</th>
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Reference</th>
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-right font-semibold">Amount</th>
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-right font-semibold">Action</th>
                </tr>
              </thead>

              <tbody>
                <tr
                  v-for="entry in filteredEntries"
                  :key="entry.id"
                  class="group"
                >
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-sm text-[var(--ledger-text)]">
                    {{ formatDateLabel(entry.donated_at) }}
                  </td>
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-sm font-medium text-[var(--ledger-text)]">
                    {{ entry.donor_name }}
                  </td>
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-sm text-[var(--ledger-text)]">
                    {{ entry.project_name }}
                  </td>
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-sm text-[var(--ledger-text)]">
                    {{ entry.method }}
                  </td>
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-sm text-[var(--ledger-text)]">
                    {{ entry.reference || "—" }}
                  </td>
                  <td
                    class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-right text-sm font-semibold"
                    :class="moneyClass(entry.amount_cents)"
                  >
                    {{ formatPHPFromCents(entry.amount_cents) }}
                  </td>
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-3 text-right">
                    <button
                      type="button"
                      class="inline-flex items-center gap-1 rounded-[999px] border border-transparent px-3 py-1 text-xs font-semibold text-[var(--ledger-muted)] opacity-0 transition group-hover:opacity-100 hover:border-rose-200 hover:bg-rose-50 hover:text-rose-700"
                      @click="deleteEntry(entry.id)"
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

                <tr v-if="filteredEntries.length === 0">
                  <td colspan="7" class="px-4 py-10 text-center text-sm text-[var(--ledger-muted)]">
                    No contribution records found.
                  </td>
                </tr>
              </tbody>

              <tfoot>
                <tr class="bg-[rgba(244,237,220,0.9)]">
                  <th
                    colspan="5"
                    class="border-t border-[color:var(--ledger-line)] px-4 py-4 text-left text-[11px] uppercase tracking-[0.28em] text-[var(--ledger-text)]"
                  >
                    Total Contributions
                  </th>
                  <td
                    class="border-t border-[color:var(--ledger-line)] px-4 py-4 text-right text-sm font-semibold text-[var(--ledger-text)]"
                  >
                    {{ formatPHPFromCents(totalContributions) }}
                  </td>
                  <td class="border-t border-[color:var(--ledger-line)] px-4 py-4"></td>
                </tr>
              </tfoot>
            </table>
          </div>
        </section>
      </div>
    </section>

    <p class="px-1 text-xs text-[var(--ledger-muted)]">
      Entries are recorded in Philippine Peso (PHP). All records stored locally - last verified balance
      {{ lastBalanceLabel }}.
    </p>

    <div class="fixed bottom-5 right-5 z-20">
      <button
        type="button"
        class="flex h-14 w-14 items-center justify-center rounded-[18px] bg-[var(--ledger-navy)] text-white shadow-[0_18px_40px_rgba(20,35,60,0.25)] transition hover:bg-[var(--ledger-navy-2)]"
        title="Quick add"
        @click="focusDonorField"
      >
        <svg class="h-6 w-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <path d="M12 5v14" />
          <path d="M5 12h14" />
        </svg>
      </button>
    </div>
  </div>
</template>
