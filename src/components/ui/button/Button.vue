<script setup lang="ts">
import { computed } from "vue";
import { cva } from "class-variance-authority";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-xl text-sm font-semibold transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-50",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground shadow-sm hover:opacity-95",
        outline: "border border-border bg-background text-foreground hover:bg-accent hover:text-accent-foreground",
        secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ghost: "hover:bg-accent hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default: "h-11 px-5 py-3",
        sm: "h-9 rounded-lg px-4",
        lg: "h-12 rounded-xl px-6",
        icon: "size-11",
        "icon-sm": "size-9",
        "icon-lg": "size-12",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
);

type ButtonVariant = "default" | "outline" | "secondary" | "ghost" | "link";
type ButtonSize = "default" | "sm" | "lg" | "icon" | "icon-sm" | "icon-lg";

type ButtonProps = {
  variant?: ButtonVariant;
  size?: ButtonSize;
  href?: string;
  target?: string;
  rel?: string;
  type?: "button" | "submit" | "reset";
};

const props = withDefaults(defineProps<ButtonProps>(), {
  variant: "default",
  size: "default",
  type: "button",
});

const isLink = computed(() => Boolean(props.href));
const tag = computed(() => (isLink.value ? "a" : "button"));
const classes = computed(() => buttonVariants({ variant: props.variant, size: props.size }));
</script>

<template>
  <component
    :is="tag"
    :href="href"
    :target="target"
    :rel="rel"
    :type="isLink ? undefined : type"
    :class="cn(classes)"
  >
    <slot />
  </component>
</template>
