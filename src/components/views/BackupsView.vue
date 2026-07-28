<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { appStatus, backupCreate, backupList, backupRestore, databaseHealth } from "../../lib/api";
import { notify } from "../../lib/feedback";
import type { AppStatus, BackupInfo, DatabaseHealth } from "../../lib/types";

const props = defineProps<{ sessionToken: string }>();

const loading = ref(true);
const working = ref(false);
const errorMessage = ref<string | null>(null);

const items = ref<BackupInfo[]>([]);
const appInfo = ref<AppStatus | null>(null);
const health = ref<DatabaseHealth | null>(null);

const backupDir = computed(() => {
  const dir = appInfo.value?.app_data_dir?.trim();
  if (!dir) return "App data\\backups";
  return `${dir}\\backups`;
});

const totalSizeBytes = computed(() => items.value.reduce((sum, item) => sum + item.bytes, 0));
const retainedCopies = computed(() => items.value.length);

function formatDateLabel(value: string) {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return value;
  return new Intl.DateTimeFormat("en-GB", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(parsed);
}

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatCheckedDate(value: string) {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return value;
  return new Intl.DateTimeFormat("en-GB", {
    year: "numeric",
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(parsed);
}

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    const [statusValue, backupRows, healthValue] = await Promise.all([
      appStatus(),
      backupList(props.sessionToken),
      databaseHealth(props.sessionToken),
    ]);
    appInfo.value = statusValue;
    items.value = backupRows;
    health.value = healthValue;
  } catch (error: any) {
    errorMessage.value = String(error);
  } finally {
    loading.value = false;
  }
}

async function create() {
  errorMessage.value = null;
  if (!confirm("Create a new backup now?")) return;

  working.value = true;
  try {
    const path = await backupCreate(props.sessionToken);
    await load();
    notify(`Backup created: ${path}`);
  } catch (error: any) {
    errorMessage.value = String(error);
  } finally {
    working.value = false;
  }
}

async function restoreFromFile() {
  errorMessage.value = null;
  const path = await open({
    multiple: false,
    title: "Select a backup file to restore",
    filters: [{ name: "SQLite Backup", extensions: ["sqlite3", "db", "sqlite"] }],
  });
  if (!path) return;
  if (!confirm("Restore this backup? The current database will be replaced.")) return;

  working.value = true;
  try {
    await backupRestore(props.sessionToken, String(path));
    await load();
    notify("Restore complete. The app is using the restored database.");
  } catch (error: any) {
    errorMessage.value = String(error);
  } finally {
    working.value = false;
  }
}

async function restoreBackup(path: string) {
  if (!confirm("Restore this backup? The current database will be replaced.")) return;

  working.value = true;
  try {
    await backupRestore(props.sessionToken, path);
    await load();
    notify("Restore complete. The app is using the restored database.");
  } catch (error: any) {
    errorMessage.value = String(error);
  } finally {
    working.value = false;
  }
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
            Backup &amp; Restore
          </h2>
          <p class="mt-3 max-w-2xl text-sm text-[var(--ledger-muted)]">
            Snapshots of the local SQLite ledger. Nothing leaves this machine.
          </p>
        </div>

        <div class="flex flex-wrap gap-2 print:hidden">
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-[12px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-4 py-3 text-sm font-semibold text-[var(--ledger-text)] transition hover:bg-[rgba(255,255,255,0.95)]"
            @click="restoreFromFile"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <path d="M12 3v12" />
              <path d="m8 7 4-4 4 4" />
              <path d="M5 21h14" />
            </svg>
            <span>Restore from file</span>
          </button>
          <button
            type="button"
            class="inline-flex items-center gap-2 rounded-[12px] border border-[var(--ledger-navy)] bg-[var(--ledger-navy)] px-4 py-3 text-sm font-semibold text-white transition hover:bg-[var(--ledger-navy-2)]"
            @click="create"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <path d="M12 5v10" />
              <path d="m8 11 4 4 4-4" />
              <path d="M5 19h14" />
            </svg>
            <span>{{ working ? "Working..." : "Back up now" }}</span>
          </button>
        </div>
      </div>

      <div class="grid gap-4 px-4 py-4 lg:grid-cols-[minmax(0,1fr)_320px]">
        <section class="ledger-card overflow-hidden rounded-[4px]">
          <div class="flex items-center justify-between border-b border-[color:var(--ledger-line)] px-4 py-4">
            <div>
              <h3 class="ledger-heading text-2xl text-[var(--ledger-text)]">
                Backup History
              </h3>
            </div>
            <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">
              Retaining last 7 copies
            </div>
          </div>

          <div v-if="errorMessage" class="mx-4 mt-4 rounded-[4px] border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
            {{ errorMessage }}
          </div>

          <div v-else-if="loading" class="px-4 py-8 text-sm text-[var(--ledger-muted)]">
            Loading backup files...
          </div>

          <div v-else class="overflow-x-auto">
            <table class="w-full min-w-[760px] border-separate border-spacing-0">
              <thead>
                <tr class="bg-[rgba(240,229,203,0.85)] text-[11px] uppercase tracking-[0.28em] text-[var(--ledger-text)]">
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">File</th>
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Created</th>
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-left font-semibold">Type</th>
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-right font-semibold">Size</th>
                  <th class="border-b border-[color:var(--ledger-line)] px-4 py-3 text-right font-semibold">Action</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="backup in items" :key="backup.full_path" class="group bg-[rgba(251,247,235,0.92)] transition hover:bg-[rgba(247,241,224,0.95)]">
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-4">
                    <div class="font-mono text-xs text-[var(--ledger-text)] break-all">{{ backup.file_name }}</div>
                  </td>
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-4 whitespace-nowrap text-sm text-[var(--ledger-text)]">
                    {{ formatDateLabel(backup.created_at) }}
                  </td>
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-4">
                    <span class="inline-flex rounded-full border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.2em] text-[var(--ledger-text)]">
                      Backup
                    </span>
                  </td>
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-4 text-right text-sm text-[var(--ledger-text)]">
                    {{ formatBytes(backup.bytes) }}
                  </td>
                  <td class="border-b border-[color:rgba(215,196,154,0.7)] px-4 py-4 text-right">
                    <button
                      type="button"
                      class="inline-flex items-center gap-2 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 py-2 text-xs font-semibold text-[var(--ledger-text)] transition hover:bg-white"
                      @click="restoreBackup(backup.full_path)"
                    >
                      <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                        <path d="M3 12a9 9 0 1 0 9-9" />
                        <path d="M3 4v8h8" />
                      </svg>
                      <span>Restore</span>
                    </button>
                  </td>
                </tr>
                <tr v-if="items.length === 0">
                  <td colspan="5" class="px-4 py-10 text-center text-sm text-[var(--ledger-muted)]">
                    No backups found yet.
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>

        <div class="grid gap-4">
          <section class="ledger-card rounded-[4px] p-5">
            <div class="ledger-eyebrow text-[11px] text-[var(--ledger-muted)]">
              AUTO-BACKUP ROTATION
            </div>

            <div class="mt-4 space-y-4">
              <div class="flex items-center justify-between gap-3">
                <div class="text-sm text-[var(--ledger-text)]">Enabled</div>
                <div class="inline-flex h-5 w-5 items-center justify-center rounded-[4px] bg-[var(--ledger-green)] text-white">
                  <svg class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" aria-hidden="true">
                    <path d="M5 12l4 4 10-10" />
                  </svg>
                </div>
              </div>

              <div class="grid gap-2">
                <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                  Frequency
                </span>
                <div class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 py-3 text-sm text-[var(--ledger-text)]">
                  Daily at startup
                </div>
              </div>

              <div class="grid gap-2">
                <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                  Copies to keep
                </span>
                <div class="h-12 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-3 py-3 text-sm text-[var(--ledger-text)]">
                  7
                </div>
              </div>

              <div class="grid gap-2">
                <span class="text-[11px] font-semibold uppercase tracking-[0.28em] text-[var(--ledger-muted)]">
                  Destination Folder
                </span>
                <div class="rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(240,233,219,0.9)] px-3 py-3 font-mono text-xs text-[var(--ledger-text)] break-all">
                  {{ backupDir }}
                </div>
              </div>
            </div>
          </section>

          <section class="ledger-card rounded-[4px] p-5">
            <div class="flex items-center gap-2">
              <svg class="h-4 w-4 text-[var(--ledger-green)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M12 22s8-4 8-10V6l-8-3-8 3v6c0 6 8 10 8 10Z" />
                <path d="M9 12l2 2 4-4" />
              </svg>
              <h3 class="ledger-heading text-2xl text-[var(--ledger-text)]">
                Database health
              </h3>
            </div>

            <div class="mt-4 space-y-3 text-sm">
              <div class="flex items-center justify-between gap-3">
                <span class="text-[var(--ledger-muted)]">Integrity check</span>
                <span class="font-semibold text-[var(--ledger-green)]">
                  {{ health?.integrity_ok ? "Passed" : "Needs attention" }}
                </span>
              </div>
              <div class="flex items-center justify-between gap-3">
                <span class="text-[var(--ledger-muted)]">Last verified</span>
                <span class="font-mono text-[var(--ledger-text)]">
                  {{ health ? formatCheckedDate(health.checked_at) : "—" }}
                </span>
              </div>
              <div class="flex items-center justify-between gap-3">
                <span class="text-[var(--ledger-muted)]">Records</span>
                <span class="font-mono text-[var(--ledger-text)]">
                  {{ health ? health.record_count.toLocaleString() : "—" }}
                </span>
              </div>
            </div>

            <button
              type="button"
              class="mt-5 inline-flex h-12 w-full items-center justify-center gap-2 rounded-[4px] border border-[color:var(--ledger-line)] bg-[rgba(255,250,240,0.9)] px-4 text-sm font-semibold text-[var(--ledger-text)] transition hover:bg-white"
              @click="load"
            >
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M4 4v6h6" />
                <path d="M20 20v-6h-6" />
                <path d="M20 8a8 8 0 0 0-14.9-3" />
                <path d="M4 16a8 8 0 0 0 14.9 3" />
              </svg>
              <span>{{ loading ? "Checking..." : "Run integrity check" }}</span>
            </button>
          </section>
        </div>
      </div>

      <div class="border-t border-[color:var(--ledger-line)] px-6 py-3 text-xs text-[var(--ledger-muted)]">
        Entries are stored locally in SQLite. Latest backup count {{ retainedCopies }} and total archive size {{ formatBytes(totalSizeBytes) }}.
      </div>
    </section>
  </div>
</template>
