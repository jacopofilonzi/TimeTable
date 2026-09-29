<script lang="ts">
  import { createShortLink, shortUrl, shortUrlForQr } from '../lib/api';
  import { useT, type Lang } from '../lib/i18n';
  import CopyField from './CopyField.svelte';
  import QrCode from './QrCode.svelte';

  let {
    uniId,
    values,
    weeks,
    lang,
  }: {
    uniId: string;
    values: Record<string, string>;
    weeks: number;
    lang: Lang;
  } = $props();

  const t = $derived(useT(lang));

  /** `null` while the short link is being created; the page URL if that fails. */
  let link = $state<{ url: string; qr: string; short: boolean } | null>(null);

  $effect(() => {
    let cancelled = false;
    link = null;
    createShortLink(uniId, values, weeks)
      .then((code) => {
        if (!cancelled) link = { url: shortUrl(code), qr: shortUrlForQr(code), short: true };
      })
      .catch((err) => {
        // The reason is in the server logs; the UI only offers the full link instead.
        console.error('short link creation failed', err);
        if (!cancelled) link = { url: location.href, qr: location.href, short: false };
      });
    return () => (cancelled = true);
  });
</script>

<div class="space-y-4">
  {#if link && !link.short}
    <p class="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">{t('result.shareError')}</p>
  {/if}
  <p class="text-center text-sm text-neutral-500">{t('result.shareHint')}</p>
  {#if link}
    <QrCode url={link.qr} errorText={t('result.qrError')} />
    <CopyField value={link.url} copyLabel={t('result.copy')} copiedLabel={t('result.copied')} />
  {:else}
    <div class="mx-auto aspect-square w-full max-w-72 animate-pulse rounded-lg bg-neutral-100"></div>
    <div class="h-11 animate-pulse rounded-lg bg-neutral-100"></div>
  {/if}
</div>
