<script lang="ts">
  import { icsUrl } from '../lib/api';
  import { useT, type Lang } from '../lib/i18n';
  import Icon, { type IconName } from './Icon.svelte';
  import Modal from './Modal.svelte';
  import Preview from './Preview.svelte';
  import QrCode from './QrCode.svelte';

  let {
    uniId,
    values,
    weeks,
    name,
    lang,
  }: {
    uniId: string;
    values: Record<string, string>;
    weeks: number;
    name: string;
    lang: Lang;
  } = $props();

  const t = $derived(useT(lang));

  const url = $derived(icsUrl(uniId, { ...values, weeks: String(weeks), name }));
  const webcal = $derived(url.replace(/^https?:/, 'webcal:'));
  const google = $derived(`https://calendar.google.com/calendar/render?cid=${encodeURIComponent(webcal)}`);
  const others = $derived<{ label: string; href: string; icon: IconName; download?: boolean }[]>([
    { label: t('result.webcal'), href: webcal, icon: 'link' },
    {
      label: t('result.outlook'),
      href: `https://outlook.live.com/calendar/0/addfromweb?url=${encodeURIComponent(url)}&name=${encodeURIComponent(name)}`,
      icon: 'mail',
    },
    { label: t('result.download'), href: url, icon: 'download', download: true },
  ]);

  let copied = $state(false);
  let qrOpen = $state(false);
  let previewOpen = $state(false);
  let urlBox: HTMLElement;

  async function copy() {
    try {
      await navigator.clipboard.writeText(url);
    } catch {
      // Clipboard API unavailable (e.g. plain http on a LAN address): select the text instead.
      getSelection()?.selectAllChildren(urlBox);
      return;
    }
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }
</script>

<div class="space-y-8">
  <div
    class="flex flex-col overflow-hidden rounded-lg bg-code shadow-[rgba(0,0,0,0.35)_0px_5px_15px] sm:flex-row sm:items-stretch"
  >
    <code bind:this={urlBox} class="flex-1 px-4 py-3 font-mono text-[0.85rem] font-semibold break-all text-code-text select-all">
      {url}
    </code>
    <button
      type="button"
      class="shrink-0 cursor-pointer bg-accent py-3 text-sm font-medium text-white hover:bg-accent-hover sm:w-24 sm:py-0"
      onclick={copy}
    >
      {copied ? t('result.copied') : t('result.copy')}
    </button>
  </div>

  <!-- Main calendar apps -->
  <div class="grid gap-3 sm:grid-cols-2">
    <a class="app-button" href={google} target="_blank" rel="noopener">
      <svg viewBox="0 0 32 32" class="h-10 w-10 shrink-0" aria-hidden="true">
        <rect x="3" y="3" width="26" height="26" rx="4" fill="#fff" />
        <path d="M7 3h18v4H7z" fill="#4285F4" />
        <path d="M29 7v18h-4V7z" fill="#FBBC04" />
        <path d="M25 29H7v-4h18z" fill="#34A853" />
        <path d="M3 25V7h4v18z" fill="#EA4335" />
        <path d="M3 7a4 4 0 0 1 4-4v4zM25 3a4 4 0 0 1 4 4h-4zM29 25a4 4 0 0 1-4 4v-4zM7 29a4 4 0 0 1-4-4h4z" fill="#1967D2" />
        <text x="16" y="20.5" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" font-weight="700" fill="#4285F4">31</text>
      </svg>
      <span class="min-w-0">
        <span class="block font-medium text-neutral-900">{t('result.google')}</span>
        <span class="block text-xs text-neutral-500">{t('result.googleHint')}</span>
      </span>
    </a>
    <a class="app-button" href={webcal}>
      <svg viewBox="0 0 32 32" class="h-10 w-10 shrink-0" aria-hidden="true">
        <rect x="3" y="3" width="26" height="26" rx="6" fill="#fff" stroke="#e5e5e5" />
        <text x="16" y="11" text-anchor="middle" font-family="-apple-system, Arial, sans-serif" font-size="5.5" font-weight="700" fill="#FF3B30">{new Date().toLocaleDateString(lang, { weekday: 'short' }).toUpperCase()}</text>
        <text x="16" y="24" text-anchor="middle" font-family="-apple-system, Arial, sans-serif" font-size="13" fill="#1d1d1f">{new Date().getDate()}</text>
      </svg>
      <span class="min-w-0">
        <span class="block font-medium text-neutral-900">{t('result.apple')}</span>
        <span class="block text-xs text-neutral-500">{t('result.appleHint')}</span>
      </span>
    </a>
  </div>

  <!-- Other apps and formats -->
  <div>
    <p class="mb-2 text-xs font-semibold tracking-wide text-neutral-400 uppercase">{t('result.more')}</p>
    <div class="flex flex-wrap gap-2">
      {#each others as other (other.label)}
        <a
          class="inline-flex items-center gap-2 rounded-full border border-neutral-200 px-4 py-1.5 text-sm text-neutral-700 transition-colors hover:border-accent hover:text-accent"
          href={other.href}
          target={other.download ? undefined : '_blank'}
          rel="noopener"
          download={other.download ? `${uniId}.ics` : undefined}
        >
          <Icon name={other.icon} />
          {other.label}
        </a>
      {/each}
    </div>
  </div>

  <!-- Tools -->
  <div class="grid grid-cols-2 gap-3 border-t border-neutral-100 pt-6">
    <button type="button" class="btn-secondary py-2.5" onclick={() => (qrOpen = true)}>
      <Icon name="qr" />
      {t('result.qr')}
    </button>
    <button type="button" class="btn-secondary py-2.5" onclick={() => (previewOpen = true)}>
      <Icon name="calendar" />
      {t('result.preview')}
    </button>
  </div>
</div>

<Modal bind:open={qrOpen} title={t('result.qrTitle')} closeLabel={t('result.close')}>
  <QrCode {url} hint={t('result.qrHint')} errorText={t('result.qrError')} />
</Modal>

<Modal bind:open={previewOpen} title={t('preview.title')} closeLabel={t('result.close')} wide>
  <Preview {uniId} params={values} {lang} />
</Modal>

<style>
  .app-button {
    display: flex;
    align-items: center;
    gap: 1rem;
    border-radius: 0.75rem;
    border: 1px solid var(--color-neutral-200);
    padding: 1rem 1.25rem;
    transition:
      border-color 150ms,
      box-shadow 150ms;
  }

  .app-button:hover {
    border-color: var(--color-accent);
    box-shadow: 0 4px 12px rgb(0 0 0 / 0.06);
  }
</style>
