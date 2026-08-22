---
title: 'Remove alternate download option'
type: 'feature'
created: '2026-08-22'
status: 'done'
route: 'one-shot'
---

# Remove alternate download option

## Intent

**Problem:** The download page presented an alternate installer button alongside the normal download, creating an unnecessary choice for users.

**Approach:** Remove the alternate installer links and related copy from both download-page implementations while preserving the normal MSI download flow and its single-card layout.

## Suggested Review Order

**Download entry points**

- The primary CTA now exposes only the normal MSI download.
  [`DownloadPage.vue:75`](../../src/components/DownloadPage.vue#L75)

- The standalone download page mirrors the same single normal download CTA.
  [`index.html:145`](../../public/download/index.html#L145)

**Single-installer presentation**

- The Vue page keeps one recommended installer card and updated supporting copy.
  [`DownloadPage.vue:128`](../../src/components/DownloadPage.vue#L128)

- The standalone page uses a single-card layout after removing the alternate option.
  [`index.html:93`](../../public/download/index.html#L93)
