<script setup lang="ts">
import { Download, ShieldCheck, HardDrive, Monitor, Package, ArrowRight } from "@lucide/vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";

const version = "0.1.0";
const files = [
  {
    label: "MSI installer",
    description: "Best for standard Windows deployment and managed installs.",
    href: "/download/financial-system_0.1.0_x64_en-US.msi",
    note: "Recommended for IT-managed installs.",
  },
  {
    label: "Setup EXE",
    description: "A double-click installer for direct desktop installation.",
    href: "/download/financial-system_0.1.0_x64-setup.exe",
    note: "Useful for quick local installs.",
  },
];

const highlights = [
  "Windows 64-bit desktop app",
  "Offline first operation",
  "Local data storage on the machine",
  `Version ${version}`,
];
</script>

<template>
  <main class="min-h-screen bg-[radial-gradient(circle_at_top_left,rgba(255,255,255,0.76),transparent_34%),radial-gradient(circle_at_85%_10%,rgba(221,189,125,0.34),transparent_28%),linear-gradient(180deg,#f6f0e4_0%,#ead9b8_100%)] px-4 py-6 text-foreground md:px-6 md:py-8">
    <div class="mx-auto flex min-h-[calc(100vh-3rem)] w-full max-w-6xl items-center">
      <section class="grid w-full gap-6">
        <Card class="overflow-hidden border-border/80 bg-card/95 shadow-[0_24px_70px_rgba(32,50,79,0.14)] backdrop-blur">
          <CardHeader class="gap-5 md:gap-6">
            <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
              <div class="max-w-3xl space-y-4">
                <Badge variant="secondary" class="border border-border/60 bg-secondary/80 px-4 py-1.5 text-[11px] uppercase tracking-[0.18em] text-secondary-foreground">
                  Offline desktop app
                </Badge>
                <div class="space-y-3">
                  <CardTitle class="text-4xl leading-[0.95] tracking-[-0.06em] md:text-6xl">
                    Download Financial System
                  </CardTitle>
                  <CardDescription class="max-w-2xl text-base md:text-lg">
                    Install the Windows desktop app to manage church funds offline. Data stays on the device,
                    so the team can keep working even without a constant internet connection.
                  </CardDescription>
                </div>
              </div>

              <div class="grid gap-3 rounded-2xl border border-border bg-background/70 p-4 text-sm text-muted-foreground md:min-w-72">
                <div class="flex items-center gap-3 text-foreground">
                  <Monitor class="size-5 text-primary" />
                  <span class="font-semibold">Desktop release</span>
                </div>
                <Separator />
                <div class="flex items-center gap-3">
                  <ShieldCheck class="size-4 text-primary" />
                  <span>Built for offline-first use</span>
                </div>
                <div class="flex items-center gap-3">
                  <HardDrive class="size-4 text-primary" />
                  <span>Stores records locally</span>
                </div>
                <div class="flex items-center gap-3">
                  <Package class="size-4 text-primary" />
                  <span>Release version {{ version }}</span>
                </div>
              </div>
            </div>
          </CardHeader>

          <CardContent>
            <div class="grid gap-6 lg:grid-cols-12">
              <div class="lg:col-span-7">
                <div class="grid gap-4 sm:grid-cols-2">
                  <Button
                    href="/download/financial-system_0.1.0_x64_en-US.msi"
                    class="h-auto justify-start rounded-2xl px-5 py-4 text-left shadow-sm"
                  >
                    <span class="flex size-11 shrink-0 items-center justify-center rounded-full bg-primary-foreground/15">
                      <Download class="size-5" />
                    </span>
                    <span class="flex flex-col items-start">
                      <span class="text-sm font-semibold">Download MSI Installer</span>
                      <span class="text-xs font-normal text-primary-foreground/80">Recommended for managed deployment</span>
                    </span>
                  </Button>

                  <Button
                    variant="outline"
                    href="/download/financial-system_0.1.0_x64-setup.exe"
                    class="h-auto justify-start rounded-2xl border-border bg-background px-5 py-4 text-left"
                  >
                    <span class="flex size-11 shrink-0 items-center justify-center rounded-full bg-muted text-foreground">
                      <ArrowRight class="size-5" />
                    </span>
                    <span class="flex flex-col items-start">
                      <span class="text-sm font-semibold">Download Setup EXE</span>
                      <span class="text-xs font-normal text-muted-foreground">Fast local install for Windows</span>
                    </span>
                  </Button>
                </div>

                <div class="mt-4 flex flex-wrap gap-2">
                  <Badge v-for="item in highlights" :key="item" variant="outline" class="border-border/70 bg-background/80 px-3 py-1.5 text-xs text-foreground">
                    {{ item }}
                  </Badge>
                </div>
              </div>

              <div class="lg:col-span-5">
                <Card class="h-full border-border/70 bg-background/70">
                  <CardHeader class="pb-4">
                    <CardTitle class="text-2xl">Download options</CardTitle>
                    <CardDescription>
                      Choose the installer that matches how you deploy Windows apps.
                    </CardDescription>
                  </CardHeader>
                  <CardContent class="grid gap-4">
                    <div
                      v-for="file in files"
                      :key="file.href"
                      class="rounded-2xl border border-border bg-card/90 p-4"
                    >
                      <div class="flex items-start justify-between gap-4">
                        <div>
                          <h4 class="text-base font-semibold text-card-foreground">
                            {{ file.label }}
                          </h4>
                          <p class="mt-2 text-sm leading-6 text-muted-foreground">
                            {{ file.description }}
                          </p>
                        </div>
                        <Badge variant="secondary" class="shrink-0">
                          v{{ version }}
                        </Badge>
                      </div>
                      <div class="mt-4 flex flex-wrap items-center gap-3">
                        <Button :href="file.href" size="sm" class="rounded-xl">
                          {{ file.label }}
                        </Button>
                        <span class="text-xs text-muted-foreground">{{ file.note }}</span>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </div>
            </div>
          </CardContent>

          <CardFooter class="flex-col items-start gap-4 pt-0">
            <Separator />
            <p class="max-w-4xl text-sm leading-6 text-muted-foreground">
              If you publish these installers on the public internet, replace the local `/download/...`
              links with the actual release asset URLs. The page is already wired for direct downloads.
            </p>
          </CardFooter>
        </Card>
      </section>
    </div>
  </main>
</template>
