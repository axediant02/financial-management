<script setup lang="ts">
interface SeparatorProps {
  orientation?: "horizontal" | "vertical";
}

const props = withDefaults(defineProps<SeparatorProps>(), {
  orientation: "horizontal",
});
</script>

<template>
  <div
    role="separator"
    :aria-orientation="props.orientation"
    :class="props.orientation === 'vertical' ? 'h-full w-px shrink-0 bg-border' : 'h-px w-full bg-border'"
  />
</template>
