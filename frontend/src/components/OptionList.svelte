<script lang="ts">
  import { localize, type SelectOption } from '../lib/api';
  import { useT, type Lang } from '../lib/i18n';

  let {
    options,
    value,
    lang,
    label,
    searchable = true,
    onselect,
  }: {
    options: SelectOption[];
    value: string | undefined;
    lang: Lang;
    label: string;
    searchable?: boolean;
    onselect: (option: SelectOption) => void;
  } = $props();

  const t = $derived(useT(lang));
  let query = $state('');

  const normalize = (s: string) =>
    s
      .normalize('NFD')
      .replace(/\p{Diacritic}/gu, '')
      .toLowerCase();

  const groups = $derived.by(() => {
    const q = normalize(query.trim());
    const matches = options.filter(
      (o) => !q || normalize(`${localize(o.label, lang)} ${o.hint ?? ''} ${o.group ?? ''}`).includes(q),
    );
    const byGroup = new Map<string, SelectOption[]>();
    for (const o of matches) {
      const g = o.group ?? '';
      if (!byGroup.has(g)) byGroup.set(g, []);
      byGroup.get(g)!.push(o);
    }
    return [...byGroup.entries()];
  });
</script>

<div class="overflow-hidden rounded-xl border border-neutral-200">
  {#if searchable}
    <div class="border-b border-neutral-200 p-2">
      <input
        type="search"
        class="w-full rounded-lg bg-neutral-100 px-3 py-2 text-sm outline-none focus:ring-2 focus:ring-accent/40"
        placeholder={t('field.search')}
        aria-label={`${label}: ${t('field.search')}`}
        bind:value={query}
      />
    </div>
  {/if}
  <div class="max-h-72 overflow-y-auto" role="listbox" aria-label={label}>
    {#each groups as [group, items] (group)}
      {#if group}
        <div class="sticky top-0 bg-neutral-50 px-3 py-1 text-[11px] font-semibold tracking-wide text-neutral-400 uppercase">
          {group}
        </div>
      {/if}
      {#each items as option (option.value)}
        {@const selected = option.value === value}
        <button
          type="button"
          role="option"
          aria-selected={selected}
          class="flex w-full cursor-pointer items-center gap-3 px-3 py-2.5 text-left text-sm transition-colors {selected
            ? 'bg-accent/5 font-medium text-accent'
            : 'hover:bg-neutral-50'}"
          onclick={() => onselect(option)}
        >
          {#if option.hint}
            <span class="w-20 shrink-0 font-mono text-xs {selected ? '' : 'text-neutral-400'}">{option.hint}</span>
          {/if}
          <span class="flex-1">{localize(option.label, lang)}</span>
          {#if selected}<span aria-hidden="true">✓</span>{/if}
        </button>
      {/each}
    {:else}
      <p class="px-3 py-6 text-center text-sm text-neutral-500">{t('field.noResults')}</p>
    {/each}
  </div>
</div>
