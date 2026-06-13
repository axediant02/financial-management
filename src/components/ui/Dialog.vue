<script setup lang="ts">
import { onBeforeUnmount, onMounted, watch } from "vue";

const props = defineProps<{
  open: boolean;
  themeMode: "light" | "dark";
  title: string;
  description?: string;
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
}>();

function close() {
  emit("update:open", false);
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Escape" && props.open) {
    close();
  }
}

watch(
  () => props.open,
  (value) => {
    document.body.style.overflow = value ? "hidden" : "";
  },
  { immediate: true },
);

onMounted(() => {
  window.addEventListener("keydown", onKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeydown);
  document.body.style.overflow = "";
});
</script>

<template>
  <teleport to="body">
    <div v-if="open" class="fixed inset-0 z-50">
      <button type="button" class="absolute inset-0 h-full w-full cursor-default bg-black/60" @click="close" aria-label="Close dialog"></button>
      <div class="absolute inset-0 flex items-center justify-center p-4">
        <div
          role="dialog"
          aria-modal="true"
          :aria-label="title"
          class="w-full max-w-xl rounded-[2px] border shadow-2xl"
          :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-950 text-slate-100' : 'border-slate-200 bg-white text-slate-900'"
        >
          <div class="flex items-start justify-between gap-4 border-b px-5 py-4" :class="themeMode === 'dark' ? 'border-slate-700 bg-slate-900' : 'border-slate-100 bg-white'">
            <div>
              <h3 class="text-lg font-semibold">{{ title }}</h3>
              <p v-if="description" class="mt-1 text-sm" :class="themeMode === 'dark' ? 'text-slate-400' : 'text-slate-500'">
                {{ description }}
              </p>
            </div>
            <button
              type="button"
              class="rounded-[2px] border px-3 py-2 text-sm font-semibold"
              :class="themeMode === 'dark' ? 'border-slate-600 bg-slate-800 text-slate-100 hover:bg-slate-700' : 'border-slate-300 bg-white text-slate-900 hover:bg-slate-50'"
              @click="close"
            >
              Close
            </button>
          </div>

          <div class="p-5">
            <slot />
          </div>
        </div>
      </div>
    </div>
  </teleport>
</template>
