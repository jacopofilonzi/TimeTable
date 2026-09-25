<script lang="ts" module>
  import type { Field } from '../lib/api';

  /** Static selects with up to this many options are rendered as chips instead of a list. */
  const MAX_CHIPS = 8;

  export function isChips(field: Field): boolean {
    return field.type === 'select' && field.options.length <= MAX_CHIPS;
  }
</script>

<script lang="ts">
  import { api, localize, type SelectOption } from '../lib/api';
  import { useT, type Lang } from '../lib/i18n';
  import OptionList from './OptionList.svelte';

  let {
    uniId,
    field,
    values,
    lang,
    column = false,
    onselect,
  }: {
    uniId: string;
    field: Field;
    values: Record<string, string>;
    lang: Lang;
    /** Lay chips out as a narrow side column (one per row on large screens). */
    column?: boolean;
    onselect: (option: SelectOption) => void;
  } = $props();

  const t = $derived(useT(lang));
  const label = $derived(localize(field.label, lang));

  let remote = $state<{ options: SelectOption[] | null; error: string | null }>({ options: null, error: null });
  let reload = $state(0);

  // Remote options: (re)load whenever the dependencies change.
  $effect(() => {
    if (field.type !== 'remote_select') return;
    reload;
    const deps = Object.fromEntries((field.depends_on ?? []).map((k) => [k, values[k] ?? '']));
    if (Object.values(deps).some((v) => !v)) {
      remote = { options: null, error: null };
      return;
    }
    const controller = new AbortController();
    remote = { options: null, error: null };
    api
      .options(uniId, field.key, deps, controller.signal)
      .then((options) => (remote = { options, error: null }))
      .catch((e) => {
        if (!controller.signal.aborted) remote = { options: null, error: e.message || t('error.load') };
      });
    return () => controller.abort();
  });
</script>

<fieldset class="space-y-2">
  <legend class="mb-2 text-sm font-medium text-neutral-700">{label}</legend>

  {#if field.type === 'select' && isChips(field)}
    <div class={column ? 'grid grid-cols-3 gap-2 lg:grid-cols-1' : 'flex flex-wrap gap-2'} role="radiogroup" aria-label={label}>
      {#each field.options as option (option.value)}
        {@const selected = values[field.key] === option.value}
        <button
          type="button"
          role="radio"
          aria-checked={selected}
          class="cursor-pointer rounded-lg px-4 py-2 text-sm transition-colors {selected
            ? 'border border-accent bg-accent/5 font-medium text-accent ring-1 ring-accent'
            : 'border border-neutral-200 bg-white hover:border-accent'}"
          onclick={() => onselect(option)}
        >
          {localize(option.label, lang)}
        </button>
      {/each}
    </div>
  {:else if field.type === 'select'}
    <OptionList options={field.options} value={values[field.key]} {lang} {label} {onselect} />
  {:else if remote.error}
    <div class="flex items-center justify-between gap-4 rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
      <span>{remote.error}</span>
      <button type="button" class="btn-secondary" onclick={() => reload++}>{t('field.retry')}</button>
    </div>
  {:else if remote.options}
    <OptionList
      options={remote.options}
      value={values[field.key]}
      searchable={field.searchable}
      {lang}
      {label}
      {onselect}
    />
  {:else}
    <div class="flex h-24 items-center justify-center rounded-lg border border-dashed border-neutral-300 text-sm text-neutral-500">
      <span class="mr-2 inline-block h-4 w-4 animate-spin rounded-full border-2 border-neutral-300 border-t-accent"></span>
      {t('field.loading')}
    </div>
  {/if}
</fieldset>
