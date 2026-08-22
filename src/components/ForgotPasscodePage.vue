<script setup lang="ts">
import { ref } from "vue";
import { ShieldCheck, KeyRound, TimerReset, ArrowLeft, BadgeInfo } from "lucide-vue-next";
import { completeAdminPasswordReplace, requestAdminPasswordReplace } from "../lib/api";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

const emit = defineEmits<{
  (e: "back-to-login"): void;
  (e: "passcode-replaced"): void;
}>();

const replaceCode = ref("");
const challengeCode = ref("");
const challengeExpiresAt = ref("");
const newPasscode = ref("");
const confirmPasscode = ref("");
const requestingCode = ref(false);
const submitting = ref(false);
const errorMessage = ref<string | null>(null);
const successMessage = ref<string | null>(null);

async function handleRequestCode() {
  errorMessage.value = null;
  successMessage.value = null;
  requestingCode.value = true;
  try {
    const challenge = await requestAdminPasswordReplace();
    challengeCode.value = challenge.code;
    challengeExpiresAt.value = challenge.expires_at;
    replaceCode.value = challenge.code;
    successMessage.value = "Replacement code created.";
  } catch (e: any) {
    errorMessage.value = String(e);
  } finally {
    requestingCode.value = false;
  }
}

async function handleSubmit() {
  errorMessage.value = null;
  successMessage.value = null;

  if (!replaceCode.value.trim()) {
    errorMessage.value = "Enter the replacement code.";
    return;
  }

  if (newPasscode.value.trim().length < 8) {
    errorMessage.value = "New passcode must be at least 8 characters.";
    return;
  }

  if (newPasscode.value !== confirmPasscode.value) {
    errorMessage.value = "Passcodes do not match.";
    return;
  }

  submitting.value = true;
  try {
    await completeAdminPasswordReplace(replaceCode.value.trim(), newPasscode.value);
    replaceCode.value = "";
    challengeCode.value = "";
    challengeExpiresAt.value = "";
    newPasscode.value = "";
    confirmPasscode.value = "";
    successMessage.value = "Passcode replaced successfully.";
    emit("passcode-replaced");
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
              <TimerReset class="size-4 text-primary" />
              Passcode recovery
            </div>
            <div class="space-y-3">
              <CardTitle class="text-4xl leading-[0.95] tracking-[-0.06em] md:text-6xl">
                Replace the admin passcode
              </CardTitle>
              <CardDescription class="max-w-2xl text-base md:text-lg">
                Generate a one-time replacement code, enter it here, and set a new admin passcode.
              </CardDescription>
            </div>
          </CardHeader>

          <CardContent class="grid gap-5">
            <div class="grid gap-4 rounded-2xl border border-border bg-background/70 p-4 text-sm text-muted-foreground md:grid-cols-2">
              <div class="rounded-xl border border-border/70 bg-card/80 p-4">
                <div class="flex items-center gap-2 text-foreground">
                  <BadgeInfo class="size-4 text-primary" />
                  <span class="font-semibold">How it works</span>
                </div>
                <p class="mt-2 leading-6">
                  Generate a code, paste it below, then submit the new passcode. The code is valid for a
                  short time only.
                </p>
              </div>
              <div class="rounded-xl border border-border/70 bg-card/80 p-4">
                <div class="flex items-center gap-2 text-foreground">
                  <ShieldCheck class="size-4 text-primary" />
                  <span class="font-semibold">Security</span>
                </div>
                <p class="mt-2 leading-6">
                  This flow is meant for the local admin account on a trusted device.
                </p>
              </div>
            </div>

            <div class="flex flex-wrap gap-3">
              <Button :disabled="requestingCode" @click="handleRequestCode">
                {{ requestingCode ? "Generating code..." : "Generate replacement code" }}
              </Button>
              <Button variant="outline" @click="emit('back-to-login')">
                <ArrowLeft class="size-4" />
                Back to login
              </Button>
            </div>

            <div v-if="challengeCode" class="rounded-2xl border border-border bg-background/70 p-4">
              <div class="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                Replacement code
              </div>
              <div class="mt-2 break-all font-mono text-2xl tracking-[0.28em] text-foreground">
                {{ challengeCode }}
              </div>
              <div class="mt-2 text-xs text-muted-foreground">
                Expires at {{ challengeExpiresAt }}
              </div>
            </div>

            <form class="grid gap-4" @submit.prevent="handleSubmit">
              <label class="grid gap-2">
                <span class="text-sm font-medium text-card-foreground">Replacement code</span>
                <div class="relative">
                  <div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-4 text-muted-foreground">
                    <KeyRound class="size-4" />
                  </div>
                  <input
                    v-model="replaceCode"
                    type="text"
                    autocomplete="off"
                    placeholder="Enter the one-time code"
                    class="w-full rounded-2xl border border-border bg-background px-4 py-3 pl-11 font-mono tracking-[0.18em] text-foreground uppercase outline-none transition focus:ring-2 focus:ring-ring"
                  />
                </div>
              </label>

              <label class="grid gap-2">
                <span class="text-sm font-medium text-card-foreground">New passcode</span>
                <input
                  v-model="newPasscode"
                  type="password"
                  autocomplete="new-password"
                  placeholder="At least 8 characters"
                  class="w-full rounded-2xl border border-border bg-background px-4 py-3 text-foreground outline-none transition focus:ring-2 focus:ring-ring"
                />
              </label>

              <label class="grid gap-2">
                <span class="text-sm font-medium text-card-foreground">Confirm new passcode</span>
                <input
                  v-model="confirmPasscode"
                  type="password"
                  autocomplete="new-password"
                  placeholder="Repeat the new passcode"
                  class="w-full rounded-2xl border border-border bg-background px-4 py-3 text-foreground outline-none transition focus:ring-2 focus:ring-ring"
                />
              </label>

              <div
                v-if="errorMessage"
                class="rounded-2xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive"
              >
                {{ errorMessage }}
              </div>

              <div
                v-if="successMessage"
                class="rounded-2xl border border-green-600/30 bg-green-600/10 px-4 py-3 text-sm text-green-700 dark:text-green-300"
              >
                {{ successMessage }}
              </div>

              <Button type="submit" class="mt-1 w-full rounded-2xl" :disabled="submitting">
                {{ submitting ? "Replacing..." : "Replace admin passcode" }}
              </Button>
            </form>
          </CardContent>
        </Card>

        <Card class="border-border/80 bg-card/90 shadow-[0_24px_70px_rgba(32,50,79,0.10)] lg:col-span-5">
          <CardHeader>
            <CardTitle class="text-2xl">When to use this</CardTitle>
            <CardDescription>
              Use the recovery flow only when the current admin passcode is lost.
            </CardDescription>
          </CardHeader>
          <CardContent class="grid gap-3 text-sm text-muted-foreground">
            <div class="rounded-2xl border border-border bg-background/70 p-4">
              The replacement code is one-time use.
            </div>
            <div class="rounded-2xl border border-border bg-background/70 p-4">
              It expires after a short window.
            </div>
            <div class="rounded-2xl border border-border bg-background/70 p-4">
              After a successful reset, everyone must sign in again.
            </div>
            <div class="rounded-2xl border border-border bg-background/70 p-4">
              Keep the code private and use it on the trusted device only.
            </div>
          </CardContent>
        </Card>
      </section>
    </div>
  </main>
</template>
