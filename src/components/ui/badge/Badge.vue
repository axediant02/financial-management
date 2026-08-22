<script setup lang="ts">
import { computed } from "vue";
import { cva } from "class-variance-authority";
import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center gap-1 rounded-full border px-3 py-1 text-xs font-semibold whitespace-nowrap transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-background",
  {
    variants: {
      variant: {
        default: "border-transparent bg-primary text-primary-foreground",
        secondary: "border-transparent bg-secondary text-secondary-foreground",
        outline: "border-border bg-transparent text-foreground",
        destructive: "border-transparent bg-destructive text-destructive-foreground",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
);

type BadgeVariant = "default" | "secondary" | "outline" | "destructive";

type BadgeProps = {
  variant?: BadgeVariant;
  href?: string;
  target?: string;
  rel?: string;
};

const props = withDefaults(defineProps<BadgeProps>(), {
  variant: "default",
});

const tag = computed(() => (props.href ? "a" : "span"));
const classes = computed(() => badgeVariants({ variant: props.variant }));
</script>

<template>
  <component
    :is="tag"
    :href="href"
    :target="target"
    :rel="rel"
    :class="cn(classes)"
  >
    <slot />
  </component>
</template>
