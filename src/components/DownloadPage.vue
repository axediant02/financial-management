<script setup lang="ts">
import {
  ArrowDownToLine,
  Check,
  LockKeyhole,
  Package,
  ShieldCheck,
} from "lucide-vue-next";

const version = "0.1.0";
const msiHref = "/download/financial-system_0.1.0_x64_en-US.msi";

const features = [
  {
    number: "01",
    title: "Everything in one place",
    description: "Keep track of projects, donations, expenses, and balances in one simple app.",
  },
  {
    number: "02",
    title: "Works without internet",
    description: "Open the app and keep working even when there is no internet connection.",
  },
  {
    number: "03",
    title: "Easy to understand",
    description: "Keep clear records of collections, expenses, documents, and reports.",
  },
];
</script>

<template>
  <main class="relative min-h-screen overflow-hidden bg-[#060d1b] px-4 py-5 text-[#edf2f7] sm:px-6 lg:px-10">
    <div class="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_16%_0%,rgba(48,76,116,0.42),transparent_32%),radial-gradient(circle_at_90%_22%,rgba(185,143,67,0.16),transparent_25%),linear-gradient(135deg,#060d1b_0%,#0b1628_54%,#091221_100%)]" />
    <div class="pointer-events-none absolute inset-0 opacity-30 [background-image:linear-gradient(rgba(218,190,123,0.07)_1px,transparent_1px),linear-gradient(90deg,rgba(218,190,123,0.07)_1px,transparent_1px)] [background-size:56px_56px] [mask-image:linear-gradient(180deg,black,transparent_88%)]" />
    <div class="pointer-events-none absolute -right-44 top-24 size-[34rem] rounded-full border border-[#d9b866]/10 shadow-[0_0_0_32px_rgba(217,184,102,0.025),0_0_0_64px_rgba(217,184,102,0.02)]" />

    <div class="relative mx-auto w-full max-w-7xl">
      <header class="flex items-center justify-between border-b border-white/10 pb-5">
        <a href="/" class="group inline-flex items-center gap-3" aria-label="Church Ledger home">
          <span class="flex size-9 items-center justify-center rounded-xl border border-[#d9b866]/50 bg-[#d9b866]/10 text-[#f0ce86] transition group-hover:bg-[#d9b866]/20">
            <LockKeyhole class="size-4" />
          </span>
          <span>
            <span class="block text-[11px] font-semibold uppercase tracking-[0.32em] text-[#f0ce86]">Church Ledger</span>
            <span class="mt-0.5 block text-[10px] uppercase tracking-[0.2em] text-[#8b9ab0]">Money records made simple</span>
          </span>
        </a>
        <div class="hidden items-center gap-3 text-[10px] font-semibold uppercase tracking-[0.26em] text-[#8b9ab0] sm:flex">
          <span class="size-1.5 rounded-full bg-[#73b587] shadow-[0_0_12px_rgba(115,181,135,0.85)]" />
          Version {{ version }}
        </div>
      </header>

      <section class="grid gap-12 pb-20 pt-16 lg:grid-cols-[minmax(0,1fr)_360px] lg:items-end lg:gap-20 lg:pb-24 lg:pt-24">
        <div class="max-w-3xl">
          <div class="mb-7 inline-flex items-center gap-3 text-[11px] font-semibold uppercase tracking-[0.3em] text-[#d9b866]">
            <span class="h-px w-8 bg-[#d9b866]" />
            For Windows computers
          </div>
          <h1 class="max-w-3xl font-serif text-5xl leading-[0.98] tracking-[-0.055em] text-[#f5f2eb] sm:text-7xl lg:text-[6.6rem]">
            Your money records,
            <span class="block text-[#d9b866]">kept close.</span>
          </h1>
          <p class="mt-8 max-w-xl text-base leading-8 text-[#aab6c6] sm:text-lg">
            A simple app for keeping track of church funds. It works without the internet and keeps your information on your computer.
          </p>
          <div class="mt-10 flex flex-col gap-3 sm:flex-row">
            <a
              :href="msiHref"
              class="group inline-flex h-14 items-center justify-center gap-3 rounded-xl bg-[#d9b866] px-6 text-sm font-bold text-[#091221] shadow-[0_14px_32px_rgba(217,184,102,0.2)] transition hover:-translate-y-0.5 hover:bg-[#edd08d] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#f5d994] focus-visible:ring-offset-2 focus-visible:ring-offset-[#060d1b]"
            >
              <ArrowDownToLine class="size-4 transition-transform group-hover:translate-y-0.5" />
              Download the app
            </a>
          </div>
          <div class="mt-6 flex flex-wrap gap-x-6 gap-y-2 text-[11px] font-semibold uppercase tracking-[0.17em] text-[#718197]">
            <span class="inline-flex items-center gap-2"><Check class="size-3 text-[#73b587]" /> For Windows 10 and 11</span>
            <span class="inline-flex items-center gap-2"><Check class="size-3 text-[#73b587]" /> Version {{ version }}</span>
            <span class="inline-flex items-center gap-2"><Check class="size-3 text-[#73b587]" /> Works without internet</span>
          </div>
        </div>

        <aside class="relative overflow-hidden rounded-2xl border border-white/12 bg-[#0e1a2c]/85 p-6 shadow-[0_28px_80px_rgba(0,0,0,0.32)] backdrop-blur-xl">
          <div class="absolute right-0 top-0 size-40 rounded-full bg-[#d9b866]/10 blur-3xl" />
          <div class="relative">
            <div class="flex items-center justify-between border-b border-white/10 pb-5">
              <span class="text-[10px] font-semibold uppercase tracking-[0.28em] text-[#8b9ab0]">This download</span>
              <span class="inline-flex items-center gap-2 text-[10px] font-bold uppercase tracking-[0.2em] text-[#73b587]"><span class="size-1.5 rounded-full bg-[#73b587]" /> Available now</span>
            </div>
            <div class="border-b border-white/10 py-7">
              <div class="flex items-start justify-between gap-4">
                <div>
                  <p class="text-3xl font-semibold tracking-[-0.04em] text-[#f5f2eb]">Version {{ version }}</p>
                  <p class="mt-2 text-sm text-[#8b9ab0]">Ready to install</p>
                </div>
                <span class="flex size-11 items-center justify-center rounded-full border border-[#d9b866]/30 bg-[#d9b866]/10 text-[#f0ce86]"><Package class="size-5" /></span>
              </div>
            </div>
            <dl class="grid gap-5 py-6 text-sm">
              <div class="flex items-center justify-between gap-4"><dt class="text-[#8b9ab0]">Works on</dt><dd class="font-medium text-[#e4eaf1]">Windows 10 or 11</dd></div>
              <div class="flex items-center justify-between gap-4"><dt class="text-[#8b9ab0]">Internet needed</dt><dd class="font-medium text-[#e4eaf1]">No</dd></div>
              <div class="flex items-center justify-between gap-4"><dt class="text-[#8b9ab0]">Your information</dt><dd class="font-medium text-[#73b587]">Stays on your computer</dd></div>
            </dl>
            <div class="flex items-center gap-3 border-t border-white/10 pt-5 text-xs leading-5 text-[#8b9ab0]">
              <ShieldCheck class="size-5 shrink-0 text-[#d9b866]" />
              Your information stays on this computer.
            </div>
          </div>
        </aside>
      </section>

      <section class="border-y border-white/10 py-8 lg:py-9">
        <div class="grid gap-8 md:grid-cols-3 md:gap-6">
          <article v-for="feature in features" :key="feature.number" class="relative md:border-l md:border-white/10 md:pl-6 first:md:border-l-0 first:md:pl-0">
            <p class="text-[10px] font-bold tracking-[0.24em] text-[#d9b866]">{{ feature.number }}</p>
            <h2 class="mt-3 text-base font-semibold tracking-[-0.015em] text-[#eef2f6]">{{ feature.title }}</h2>
            <p class="mt-2 max-w-sm text-sm leading-6 text-[#8795a9]">{{ feature.description }}</p>
          </article>
        </div>
      </section>

      <footer class="flex flex-col gap-4 border-t border-white/10 py-7 text-xs leading-6 text-[#718197] sm:flex-row sm:items-center sm:justify-between">
        <p>Church Ledger <span class="px-2 text-[#d9b866]">/</span> Version {{ version }}</p>
        <p class="inline-flex items-center gap-2"><LockKeyhole class="size-3 text-[#d9b866]" /> Your information stays on your computer</p>
      </footer>
    </div>
  </main>
</template>
