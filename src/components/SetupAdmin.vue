<script setup lang="ts">
import { ref } from "vue";
import { ShieldCheck, Database, HardDrive, LockKeyhole } from "lucide-vue-next";
import { bootstrapAdmin, login } from "../lib/api";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

const props = defineProps<{
  dbPath: string | null;
  appDataDir: string | null;
}>();

const emit = defineEmits<{
  (e: "bootstrap-success", sessionToken: string): void;
}>();

const adminPasscode = ref("");
const confirmPasscode = ref("");
const submitting = ref(false);
const errorMessage = ref<string | null>(null);

async function submit() {
  errorMessage.value = null;

  if (adminPasscode.value.trim().length < 8) {
    errorMessage.value = "Passcode must be at least 8 characters.";
    return;
  }

  if (adminPasscode.value !== confirmPasscode.value) {
    errorMessage.value = "Passcodes do not match.";
    return;
  }

  submitting.value = true;
  try {
    await bootstrapAdmin(adminPasscode.value);
    const res = await login(adminPasscode.value);
    emit("bootstrap-success", res.session_token);
    adminPasscode.value = "";
    confirmPasscode.value = "";
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <main class="min-h-screen overflow-hidden bg-[radial-gradient(circle_at_top_left,rgba(255,255,255,0.76),transparent_34%),radial-gradient(circle_at_85%_10%,rgba(221,189,125,0.34),transparent_28%),linear-gradient(180deg,#f6f0e4_0%,#ead9b8_100%)] px-4 py-6 text-foreground md:px-6 md:py-8">
    <div class="mx-auto flex min-h-[calc(100vh-3rem)] w-full max-w-6xl items-center">
      <section class="grid w-full gap-6 lg:grid-cols-12">
        <Card class="overflow-hidden border-border/80 bg-card/95 shadow-[0_24px_70px_rgba(32,50,79,0.14)] backdrop-blur lg:col-span-7">
          <CardHeader class="gap-4 md:gap-5">
            <div class="inline-flex w-fit items-center gap-2 rounded-full border border-border/60 bg-secondary/80 px-4 py-1.5 text-[11px] uppercase tracking-[0.18em] text-secondary-foreground">
              <ShieldCheck class="size-4 text-primary" />
              First run setup
            </div>
            <div class="space-y-3">
              <CardTitle class="text-4xl leading-[0.95] tracking-[-0.06em] md:text-6xl">
                Create the admin passcode
              </CardTitle>
              <CardDescription class="max-w-2xl text-base md:text-lg">
                Set the first admin passcode for this device. It unlocks the local ledger and protects
                access to the database on this machine.
              </CardDescription>
            </div>
          </CardHeader>

          <CardContent class="grid gap-5">
            <div class="grid gap-4 rounded-2xl border border-border bg-background/70 p-4 text-sm text-muted-foreground md:grid-cols-2">
              <div class="rounded-xl border border-border/70 bg-card/80 p-4">
                <div class="flex items-center gap-2 text-foreground">
                  <Database class="size-4 text-primary" />
                  <span class="font-semibold">Database</span>
                </div>
                <p class="mt-2 break-all text-xs leading-5 text-muted-foreground">
                  {{ props.dbPath || "-" }}
                </p>
              </div>
              <div class="rounded-xl border border-border/70 bg-card/80 p-4">
                <div class="flex items-center gap-2 text-foreground">
                  <HardDrive class="size-4 text-primary" />
                  <span class="font-semibold">App data</span>
                </div>
                <p class="mt-2 break-all text-xs leading-5 text-muted-foreground">
                  {{ props.appDataDir || "-" }}
                </p>
              </div>
            </div>

            <form class="grid gap-4" @submit.prevent="submit">
              <label class="grid gap-2">
                <span class="text-sm font-medium text-card-foreground">Admin passcode</span>
                <div class="relative">
                  <div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-4 text-muted-foreground">
                    <LockKeyhole class="size-4" />
                  </div>
                  <input
                    v-model="adminPasscode"
                    type="password"
                    class="w-full rounded-2xl border border-border bg-background px-4 py-3 pl-11 text-foreground outline-none transition focus:ring-2 focus:ring-ring"
                    placeholder="At least 8 characters"
                    autocomplete="new-password"
                  />
                </div>
              </label>

              <label class="grid gap-2">
                <span class="text-sm font-medium text-card-foreground">Confirm passcode</span>
                <input
                  v-model="confirmPasscode"
                  type="password"
                  class="w-full rounded-2xl border border-border bg-background px-4 py-3 text-foreground outline-none transition focus:ring-2 focus:ring-ring"
                  placeholder="Repeat the passcode"
                  autocomplete="new-password"
                />
              </label>

              <div
                v-if="errorMessage"
                class="rounded-2xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive"
              >
                {{ errorMessage }}
              </div>

              <Button type="submit" class="mt-1 w-full rounded-2xl" :disabled="submitting">
                {{ submitting ? "Creating..." : "Create admin passcode" }}
              </Button>
            </form>
          </CardContent>
        </Card>

        <div class="lg:col-span-5"></div>
      </section>
    </div>
  </main>
</template>
