<script setup lang="ts">
import { onMounted, ref } from "vue";
import { auditTrailList } from "../../lib/api";
import type { AuditEvent } from "../../lib/types";

const props = defineProps<{ sessionToken: string }>();

const loading = ref(true);
const errorMessage = ref<string | null>(null);
const events = ref<AuditEvent[]>([]);

function formatDateLabel(value: string) {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return value;
  return new Intl.DateTimeFormat("en-GB", {
    year: "numeric",
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    timeZone: "UTC",
    timeZoneName: "short",
  }).format(parsed);
}

function actionClass(action: string) {
  if (action === "delete") return "border-rose-200 bg-rose-50 text-rose-700";
  if (action === "create") return "border-emerald-200 bg-emerald-50 text-emerald-700";
  if (action === "update") return "border-amber-200 bg-amber-50 text-amber-700";
  return "border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] text-[var(--ledger-text)]";
}

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    events.value = await auditTrailList(props.sessionToken);
  } catch (error: unknown) {
    errorMessage.value = String(error);
  } finally {
    loading.value = false;
  }
}

onMounted(load);
</script>

<template>
  <div class="space-y-5 text-[var(--ledger-text)]">
    <section class="ledger-panel overflow-hidden rounded-[26px]">
      <div class="flex flex-col gap-5 border-b border-[color:var(--ledger-line)] px-6 py-5 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <p class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">SECURITY &amp; ACCOUNTABILITY</p>
          <h2 class="ledger-heading mt-2 text-4xl text-[var(--ledger-text)]">Audit Trail</h2>
          <p class="mt-3 max-w-2xl text-sm text-[var(--ledger-muted)]">
            Review successful changes and important operations performed in the ledger.
          </p>
        </div>
        <button
          type="button"
          class="inline-flex items-center gap-2 rounded-[12px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-4 py-3 text-sm font-semibold text-[var(--ledger-text)] transition hover:bg-[rgba(255,255,255,0.95)]"
          @click="load"
        >
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <path d="M4 4v6h6" />
            <path d="M20 20v-6h-6" />
            <path d="M20 8a8 8 0 0 0-14.9-3" />
            <path d="M4 16a8 8 0 0 0 14.9 3" />
          </svg>
          <span>{{ loading ? "Refreshing..." : "Refresh" }}</span>
        </button>
      </div>

      <div class="px-4 py-4">
        <section class="ledger-card overflow-hidden rounded-[4px]">
          <div class="flex items-center justify-between border-b border-[color:var(--ledger-line)] px-4 py-4">
            <div>
              <h3 class="ledger-heading text-2xl text-[var(--ledger-text)]">Activity history</h3>
              <p class="mt-1 text-sm text-[var(--ledger-muted)]">Newest actions appear first. Up to 500 entries are shown.</p>
            </div>
            <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">{{ events.length }} entries</div>
          </div>

          <div v-if="errorMessage" class="mx-4 mt-4 rounded-[4px] border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
            {{ errorMessage }}
          </div>

          <div v-else-if="loading" class="px-4 py-8 text-sm text-[var(--ledger-muted)]">Loading audit history...</div>

          <div v-else-if="events.length === 0" class="px-4 py-12 text-center">
            <div class="ledger-heading text-2xl text-[var(--ledger-text)]">No activity recorded yet.</div>
            <p class="mt-2 text-sm text-[var(--ledger-muted)]">Successful additions, updates, deletions, exports, and backups will appear here.</p>
          </div>

          <div v-else class="overflow-x-auto">
            <table class="w-full min-w-[900px] border-separate border-spacing-0">
              <thead>
                <tr class="bg-[rgba(240,229,203,0.85)] text-[11px] uppercase tracking-[0.28em] text-[var(--ledger-text)]">
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Timestamp (UTC)</th>
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Action</th>
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Entity</th>
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Record</th>
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Summary</th>
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Actor</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="event in events" :key="event.id" class="bg-[rgba(251,247,235,0.92)] transition hover:bg-[rgba(247,241,224,0.95)]">
                  <td class="whitespace-nowrap border-b border-[color:rgba(215,196,154,0.7)] px-4 py-4 text-sm text-[var(--ledger-text)]">{{ formatDateLabel(event.created_at) }}</td>
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-4">
                    <span class="inline-flex rounded-full border px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.2em]" :class="actionClass(event.action)">{{ event.action }}</span>
                  </td>
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-4 text-sm capitalize text-[var(--ledger-text)]">{{ event.entity }}</td>
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-4 font-mono text-sm text-[var(--ledger-text)]">{{ event.record_id == null ? "—" : `#${event.record_id}` }}</td>
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-4 text-sm text-[var(--ledger-text)]">{{ event.summary }}</td>
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-4 text-sm font-medium text-[var(--ledger-text)]">{{ event.actor }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>
      </div>
    </section>
  </div>
</template>
