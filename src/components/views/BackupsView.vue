<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { backupCreate, backupList, backupRestore } from "../../lib/api";
import { notify } from "../../lib/feedback";
import type { BackupInfo } from "../../lib/types";

const props = defineProps<{ sessionToken: string }>();

const loading = ref(true);
const errorMessage = ref<string | null>(null);
const items = ref<BackupInfo[]>([]);

async function load() {
  loading.value = true;
  errorMessage.value = null;
  try {
    items.value = await backupList(props.sessionToken);
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function create() {
  errorMessage.value = null;
  if (!confirm("Create a new backup now?")) return;
  try {
    await backupCreate(props.sessionToken);
    await load();
    notify("Backup created.");
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

async function restore() {
  errorMessage.value = null;
  const path = await open({
    multiple: false,
    title: "Select a backup file to restore",
    filters: [{ name: "SQLite Backup", extensions: ["sqlite3", "db"] }],
  });
  if (!path) return;
  if (!confirm("Restore this backup? The current database will be replaced.")) return;
  try {
    await backupRestore(props.sessionToken, String(path));
    await load();
    notify("Restore complete. The app is using the restored database.");
  } catch (e: any) {
    errorMessage.value = String(e);
  }
}

load();
</script>

<template>
  <div class="space-y-6">
    <div v-if="errorMessage" class="rounded-xl border border-rose-500/40 bg-rose-500/10 p-4 text-rose-200">
      {{ errorMessage }}
    </div>

    <div class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
      <div class="flex items-center justify-between gap-3">
        <div>
          <div class="font-semibold">Backups</div>
          <div class="text-sm text-slate-400">Auto backups rotate (latest 7) + manual backup/restore</div>
        </div>
        <div class="flex gap-2">
          <button class="rounded-xl bg-slate-800 hover:bg-slate-700 px-4 py-2 font-semibold" @click="restore">Restore…</button>
          <button class="rounded-xl bg-indigo-600 hover:bg-indigo-500 px-4 py-2 font-semibold" @click="create">Create Backup</button>
        </div>
      </div>
    </div>

    <div class="rounded-2xl border border-slate-800 bg-slate-900/40 overflow-hidden">
      <div class="p-5 flex items-center justify-between">
        <div class="font-semibold">Backup Files</div>
        <button class="rounded-lg bg-slate-800 hover:bg-slate-700 px-3 py-2 text-sm font-semibold" @click="load">
          {{ loading ? "Loading…" : "Refresh" }}
        </button>
      </div>
      <div class="border-t border-slate-800">
        <table class="w-full text-sm">
          <thead class="bg-slate-950/40 text-slate-300">
            <tr>
              <th class="text-left p-3 font-medium">File</th>
              <th class="text-left p-3 font-medium">Created</th>
              <th class="text-right p-3 font-medium">Size</th>
            </tr>
          </thead>
          <tbody>
            <tr v-if="loading" class="border-t border-slate-800">
              <td class="p-3 text-slate-400" colspan="3">Loading…</td>
            </tr>
            <tr v-for="b in items" :key="b.full_path" class="border-t border-slate-800">
              <td class="p-3 font-mono text-xs break-all">{{ b.file_name }}</td>
              <td class="p-3 text-slate-300">{{ b.created_at }}</td>
              <td class="p-3 text-right text-slate-300">{{ (b.bytes / 1024).toFixed(1) }} KB</td>
            </tr>
            <tr v-if="!loading && items.length === 0" class="border-t border-slate-800">
              <td class="p-3 text-slate-400" colspan="3">No backups found yet.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
