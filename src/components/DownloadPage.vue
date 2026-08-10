<template>
  <main class="min-h-screen bg-[radial-gradient(circle_at_top_left,_rgba(255,255,255,0.86),_transparent_34%),radial-gradient(circle_at_85%_10%,_rgba(221,189,125,0.45),_transparent_28%),linear-gradient(180deg,_#f4efe4_0%,_#e7dcc2_100%)] px-4 py-8 text-[#20324f]">
    <div class="mx-auto flex min-h-[calc(100vh-4rem)] w-full max-w-6xl items-center">
      <section class="grid w-full gap-6">
        <section class="relative overflow-hidden rounded-[28px] border border-[rgba(113,82,24,0.14)] bg-[rgba(255,252,244,0.92)] p-8 shadow-[0_26px_70px_rgba(69,52,17,0.18)] backdrop-blur-[16px] md:p-10">
          <div
            class="inline-flex items-center gap-2 rounded-full border border-[rgba(143,107,40,0.18)] bg-white/80 px-4 py-2 text-[12px] uppercase tracking-[0.18em] text-[#6f521c]"
          >
            Offline desktop app
          </div>
          <h1 class="mt-5 max-w-4xl text-4xl font-semibold tracking-[-0.05em] text-[#20324f] md:text-6xl">
            Download Financial System
          </h1>
          <p class="mt-4 max-w-3xl text-base leading-7 text-[#6b6a5d] md:text-lg md:leading-8">
            Install the Windows desktop app to manage church funds offline. It stores data locally on the
            device, so your team can keep working even without an internet connection.
          </p>

          <div class="mt-7 flex flex-col gap-3 sm:flex-row sm:flex-wrap">
            <a
              class="inline-flex items-center justify-center rounded-[16px] bg-gradient-to-b from-[#8f6b28] to-[#6f521c] px-5 py-4 font-bold text-white shadow-[0_16px_30px_rgba(111,82,28,0.22)] transition hover:-translate-y-[1px]"
              href="/download/financial-system_0.1.0_x64_en-US.msi"
            >
              Download MSI Installer
            </a>
            <a
              class="inline-flex items-center justify-center rounded-[16px] border border-[rgba(32,50,79,0.16)] bg-white/90 px-5 py-4 font-bold text-[#20324f] transition hover:-translate-y-[1px]"
              href="/download/financial-system_0.1.0_x64-setup.exe"
            >
              Download Setup EXE
            </a>
          </div>

          <div class="mt-6 flex flex-wrap gap-3 text-sm text-[#6b6a5d]">
            <span class="inline-flex items-center rounded-[14px] border border-[rgba(32,50,79,0.08)] bg-white/70 px-4 py-2">
              Windows 64-bit
            </span>
            <span class="inline-flex items-center rounded-[14px] border border-[rgba(32,50,79,0.08)] bg-white/70 px-4 py-2">
              Offline first
            </span>
            <span class="inline-flex items-center rounded-[14px] border border-[rgba(32,50,79,0.08)] bg-white/70 px-4 py-2">
              Local data storage
            </span>
            <span class="inline-flex items-center rounded-[14px] border border-[rgba(32,50,79,0.08)] bg-white/70 px-4 py-2">
              Version 0.1.0
            </span>
          </div>
        </section>

        <section class="grid gap-6 lg:grid-cols-12">
          <div class="rounded-[24px] border border-[rgba(113,82,24,0.14)] bg-[rgba(255,252,244,0.92)] p-6 shadow-[0_26px_70px_rgba(69,52,17,0.18)] backdrop-blur-[16px] lg:col-span-7">
            <h2 class="mb-4 text-xl font-semibold text-[#20324f]">What it does</h2>
            <ul class="grid gap-3">
              <li class="rounded-[16px] border border-[rgba(32,50,79,0.08)] bg-white/75 px-4 py-3 leading-6 text-[#20324f]">
                Tracks project funds, contributions, expenses, and balances in one place.
              </li>
              <li class="rounded-[16px] border border-[rgba(32,50,79,0.08)] bg-white/75 px-4 py-3 leading-6 text-[#20324f]">
                Records registration collections and deductions for post-event documentation.
              </li>
              <li class="rounded-[16px] border border-[rgba(32,50,79,0.08)] bg-white/75 px-4 py-3 leading-6 text-[#20324f]">
                Works as a local desktop app, so your records stay on the machine.
              </li>
              <li class="rounded-[16px] border border-[rgba(32,50,79,0.08)] bg-white/75 px-4 py-3 leading-6 text-[#20324f]">
                Designed for use without a browser or constant internet connection.
              </li>
            </ul>
          </div>

          <div class="rounded-[24px] border border-[rgba(113,82,24,0.14)] bg-[rgba(255,252,244,0.92)] p-6 shadow-[0_26px_70px_rgba(69,52,17,0.18)] backdrop-blur-[16px] lg:col-span-5">
            <h2 class="mb-4 text-xl font-semibold text-[#20324f]">Download options</h2>
            <div class="grid gap-3">
              <div class="rounded-[18px] border border-[rgba(32,50,79,0.08)] bg-white/80 p-4">
                <strong class="block text-[#20324f]">MSI installer</strong>
                <p class="mt-2 text-sm leading-6 text-[#6b6a5d]">
                  Best for standard Windows deployment. Upload this file to your public hosting or release
                  page.
                </p>
                <a
                  class="mt-4 inline-flex items-center justify-center rounded-[16px] bg-gradient-to-b from-[#8f6b28] to-[#6f521c] px-4 py-3 font-bold text-white"
                  href="/download/financial-system_0.1.0_x64_en-US.msi"
                >
                  Get MSI
                </a>
              </div>
              <div class="rounded-[18px] border border-[rgba(32,50,79,0.08)] bg-white/80 p-4">
                <strong class="block text-[#20324f]">Setup EXE</strong>
                <p class="mt-2 text-sm leading-6 text-[#6b6a5d]">
                  Alternative Windows installer for easy double-click installation.
                </p>
                <a
                  class="mt-4 inline-flex items-center justify-center rounded-[16px] border border-[rgba(32,50,79,0.16)] bg-white px-4 py-3 font-bold text-[#20324f]"
                  href="/download/financial-system_0.1.0_x64-setup.exe"
                >
                  Get EXE
                </a>
              </div>
            </div>
          </div>
        </section>

        <section class="rounded-[20px] border border-[rgba(113,82,24,0.14)] bg-[rgba(255,252,244,0.92)] px-5 py-4 text-sm leading-6 text-[#6b6a5d] shadow-[0_26px_70px_rgba(69,52,17,0.18)] backdrop-blur-[16px]">
          To make these buttons work on the public internet, upload the installer files next to this page or
          replace the links with your release URLs. If you are publishing with GitHub Releases, point the
          links at the release asset URLs for the same filenames.
        </section>
      </section>
    </div>
  </main>
</template>
