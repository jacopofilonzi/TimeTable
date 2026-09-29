<script lang="ts">
  import { icsUrl } from '../lib/api';
  import { useT, type Lang } from '../lib/i18n';
  import CopyField from './CopyField.svelte';
  import Icon, { type IconName } from './Icon.svelte';
  import Modal from './Modal.svelte';
  import Preview from './Preview.svelte';
  import ShareLink from './ShareLink.svelte';

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
  const others = $derived<{ label: string; href: string; icon: IconName }[]>([
    { label: t('result.webcal'), href: webcal, icon: 'link' },
    {
      label: t('result.outlook'),
      href: `https://outlook.live.com/calendar/0/addfromweb?url=${encodeURIComponent(url)}&name=${encodeURIComponent(name)}`,
      icon: 'mail',
    },
  ]);
  const chip =
    'inline-flex cursor-pointer items-center gap-2 rounded-full border border-neutral-200 px-4 py-1.5 text-sm text-neutral-700 transition-colors hover:border-accent hover:text-accent';

  let shareOpen = $state(false);
  let icsOpen = $state(false);
  let previewOpen = $state(false);
</script>

<div class="space-y-8">
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
        <a class={chip} href={other.href} target="_blank" rel="noopener">
          <Icon name={other.icon} />
          {other.label}
        </a>
      {/each}
      <button type="button" class={chip} onclick={() => (icsOpen = true)}>
        <Icon name="fileCode" />
        {t('result.ics')}
      </button>
    </div>
  </div>

  <!-- Tools -->
  <div class="grid grid-cols-2 gap-3 border-t border-neutral-100 pt-6">
    <button type="button" class="btn-secondary py-2.5" onclick={() => (shareOpen = true)}>
      <Icon name="share" />
      {t('result.share')}
    </button>
    <button type="button" class="btn-secondary py-2.5" onclick={() => (previewOpen = true)}>
      <Icon name="calendar" />
      {t('result.preview')}
    </button>
  </div>
</div>

<Modal bind:open={shareOpen} title={t('result.share')} closeLabel={t('result.close')}>
  <ShareLink {uniId} {values} {weeks} {lang} />
</Modal>

<Modal bind:open={icsOpen} title={t('result.icsTitle')} closeLabel={t('result.close')}>
  <div class="space-y-4">
    <p class="text-sm text-neutral-500">{t('result.icsHint')}</p>
    <CopyField value={url} copyLabel={t('result.copy')} copiedLabel={t('result.copied')} />
    <a class="btn-secondary w-full py-2.5" href={url} download={`${uniId}.ics`}>
      <Icon name="download" />
      {t('result.download')}
    </a>
  </div>
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
