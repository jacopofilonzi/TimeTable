<script lang="ts">
  import { onMount } from 'svelte';
  import { api, BASE, localize, type Field, type SelectOption, type Step, type University } from '../lib/api';
  import { useT, type Lang } from '../lib/i18n';
  import { defaults, EMPTY, fields as schemaFields, fromQuery, stepIds, toQuery, type Progress } from '../lib/wizard-url';
  import FieldInput, { isChips } from './FieldInput.svelte';
  import ResultStep from './ResultStep.svelte';
  import Stepper from './Stepper.svelte';

  let { lang }: { lang: Lang } = $props();
  const t = $derived(useT(lang));

  type WizardStep = { kind: 'university' } | { kind: 'fields'; step: Step } | { kind: 'options' } | { kind: 'result' };

  let universities = $state<University[] | null>(null);
  let loadError = $state<string | null>(null);
  /** Wizard progress, mirrored in the page URL (see `lib/wizard-url.ts`). */
  let p = $state<Progress>(structuredClone(EMPTY));
  /** The progress has been read from the URL: don't write the URL before, or it would be lost. */
  let ready = $state(false);
  /** Step of the last URL written: moving to another step adds a history entry. */
  let lastStep: string | undefined;
  /** Discards URL restores overtaken by a newer one. */
  let restoreToken = 0;

  const uni = $derived(universities?.find((u) => u.id === p.uniId) ?? null);
  const fields = $derived<Field[]>(uni ? schemaFields(uni) : []);
  const query = $derived(toQuery(uni, p));
  const otherLang = $derived({
    code: lang === 'it' ? 'EN' : 'IT',
    href: `${BASE}/${lang === 'it' ? 'en/' : ''}${query ? `?${query}` : ''}`,
  });
  const steps = $derived<WizardStep[]>([
    { kind: 'university' },
    ...(uni?.steps ?? []).map((step) => ({ kind: 'fields' as const, step })),
    { kind: 'options' },
    { kind: 'result' },
  ]);
  const optionsIndex = $derived(steps.findIndex((s) => s.kind === 'options'));
  const current = $derived(steps[p.index] ?? steps[0]);
  const stepLabels = $derived(steps.map((s) => stepTitle(s)));
  const canNext = $derived.by(() => {
    switch (current.kind) {
      case 'university':
        return !!uni;
      case 'fields':
        return current.step.fields.every((f) => !!p.values[f.key]);
      case 'options':
        return !!uni && p.weeks >= uni.weeks.min && p.weeks <= uni.weeks.max && p.name.trim() !== '';
      default:
        return false;
    }
  });

  const defaultName = $derived(uni ? nameFor(uni, p.labels) : '');

  function nameFor(u: University, labels: Record<string, string>): string {
    return [localize(u.name, lang), ...schemaFields(u).map((f) => labels[f.key])].filter(Boolean).join(' · ');
  }

  function stepTitle(s: WizardStep): string {
    switch (s.kind) {
      case 'university':
        return t('steps.university');
      case 'fields':
        return localize(s.step.title, lang);
      case 'options':
        return t('steps.options');
      case 'result':
        return t('steps.result');
    }
  }

  onMount(() => {
    const controller = new AbortController();
    api
      .universities(controller.signal)
      .then(async (list) => {
        universities = list;
        await restore(list);
        ready = true;
      })
      .catch((e) => {
        if (!controller.signal.aborted) loadError = e.message || t('error.load');
      });
    // Back/forward: the URL of that history entry becomes the progress.
    const onPopState = () => {
      if (universities && ready) restore(universities);
    };
    addEventListener('popstate', onPopState);
    return () => {
      controller.abort();
      removeEventListener('popstate', onPopState);
    };
  });

  // Mirror the progress in the URL: a new history entry per step, in-place updates otherwise.
  $effect(() => {
    const q = query;
    if (!ready) return;
    const step = stepIds(uni)[p.index];
    const href = `${location.pathname}${q ? `?${q}` : ''}`;
    if (href !== location.pathname + location.search) {
      if (lastStep !== undefined && step !== lastStep) history.pushState(null, '', href);
      else history.replaceState(null, '', href);
    }
    lastStep = step;
  });

  async function restore(list: University[]) {
    const token = ++restoreToken;
    const next = await fromQuery(location.search, list, lang, p, nameFor);
    if (token !== restoreToken) return;
    lastStep = stepIds(list.find((u) => u.id === next.uniId) ?? null)[next.index];
    p = next;
  }

  function selectUniversity(u: University) {
    if (p.uniId !== u.id) {
      p = { ...structuredClone(EMPTY), ...defaults(u, lang), uniId: u.id, weeks: u.weeks.default };
    }
    go(1);
  }

  function selectOption(key: string, option: SelectOption) {
    p.values[key] = option.value;
    p.labels[key] = localize(option.label, lang);
    // Clear fields that depend (directly or transitively) on the changed one.
    const changed = new Set([key]);
    for (let grew = true; grew; ) {
      grew = false;
      for (const f of fields) {
        if (!changed.has(f.key) && f.depends_on?.some((d) => changed.has(d))) {
          changed.add(f.key);
          delete p.values[f.key];
          delete p.labels[f.key];
          grew = true;
        }
      }
    }
  }

  function go(index: number) {
    p.index = Math.max(0, Math.min(index, steps.length - 1));
    if (steps[p.index]?.kind === 'options' && !p.nameEdited) p.name = defaultName;
  }

  function restart() {
    p = structuredClone(EMPTY);
  }

  /** A step mixing a list field with chip fields lays the chips out as a side column. */
  function sideColumn(step: Step): boolean {
    return step.fields.some(isChips) && step.fields.some((f) => !isChips(f));
  }
</script>

<div class="flex min-h-144 flex-col md:flex-row">
  <!-- Sidebar -->
  <aside class="flex flex-col gap-8 bg-neutral-50 p-6 md:w-72 md:shrink-0 md:border-r md:border-neutral-200">
    <div class="flex items-center gap-3">
      <img src={`${BASE}/logo.png`} alt="" class="h-10 w-10 shrink-0" width="40" height="40" />
      <div class="min-w-0 flex-1">
        <p class="font-semibold">TimeTable</p>
        <p class="text-xs text-neutral-500">{t('sidebar.tagline')}</p>
      </div>
      <a href={otherLang.href} class="btn-ghost px-2 py-1 text-xs md:hidden">{otherLang.code}</a>
    </div>

    {#if universities}
      <div class="hidden md:block">
        <Stepper labels={stepLabels} current={p.index} ongo={go} />
      </div>
    {/if}

    {#if uni}
      <div class="mt-auto hidden space-y-2 rounded-xl border border-neutral-200 bg-white p-4 text-sm md:block">
        <p class="text-xs font-semibold tracking-wide text-neutral-400 uppercase">{t('summary.title')}</p>
        <dl class="space-y-1.5">
          <div>
            <dt class="text-xs text-neutral-400">{t('summary.university')}</dt>
            <dd class="font-medium">{localize(uni.name, lang)}</dd>
          </div>
          {#each fields.filter((f) => p.labels[f.key]) as field (field.key)}
            <div>
              <dt class="text-xs text-neutral-400">{localize(field.label, lang)}</dt>
              <dd class="font-medium">{p.labels[field.key]}</dd>
            </div>
          {/each}
          {#if p.index >= optionsIndex}
            <div>
              <dt class="text-xs text-neutral-400">{t('summary.weeks')}</dt>
              <dd class="font-medium">{p.weeks}</dd>
            </div>
          {/if}
        </dl>
      </div>
    {/if}

    <div class="hidden gap-2 text-xs md:flex {uni ? '' : 'mt-auto'}">
      <span class="rounded-md bg-neutral-900 px-2 py-1 text-white">{lang.toUpperCase()}</span>
      <a class="rounded-md px-2 py-1 text-neutral-500 hover:bg-neutral-200" href={otherLang.href}>{otherLang.code}</a>
    </div>
  </aside>

  <!-- Content -->
  <section class="flex min-w-0 flex-1 flex-col p-6 sm:p-10">
    {#if loadError}
      <div class="rounded-lg border border-red-200 bg-red-50 px-4 py-6 text-center text-sm text-red-700">
        <p>{t('error.load')}</p>
        <p class="mt-1 text-xs opacity-75">{loadError}</p>
      </div>
    {:else if !universities || !ready}
      <div class="space-y-3">
        <div class="h-4 w-24 animate-pulse rounded bg-neutral-100"></div>
        <div class="h-8 w-2/3 animate-pulse rounded bg-neutral-100"></div>
        <div class="h-24 animate-pulse rounded-xl bg-neutral-100"></div>
      </div>
    {:else}
      <header class="mb-6">
        <p class="text-xs font-semibold tracking-wide text-accent uppercase">
          {current.kind === 'result' ? t('result.done') : t('steps.counter', { n: p.index + 1, total: steps.length })}
        </p>
        <h1 class="mt-1 text-2xl font-semibold">
          {current.kind === 'university'
            ? t('university.title')
            : current.kind === 'options'
              ? t('options.title')
              : current.kind === 'result'
                ? t('result.title')
                : stepTitle(current)}
        </h1>
        {#if current.kind === 'university'}
          <p class="mt-1 text-sm text-neutral-500">{t('university.subtitle')}</p>
        {:else if current.kind === 'fields' && current.step.description}
          <p class="mt-1 text-sm text-neutral-500">{localize(current.step.description, lang)}</p>
        {:else if current.kind === 'result'}
          <p class="mt-1 text-sm text-neutral-500">{t('result.subtitle')}</p>
        {/if}
      </header>

      <div class="flex-1">
      {#if current.kind === 'university'}
        <div class="grid gap-3 sm:grid-cols-2">
          {#each universities as u (u.id)}
            <button
              type="button"
              class="flex cursor-pointer items-center gap-4 rounded-xl border-2 p-4 text-left transition-colors {p.uniId === u.id
                ? 'border-accent bg-accent/5'
                : 'border-neutral-200 hover:border-accent'}"
              onclick={() => selectUniversity(u)}
            >
              <span class="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg bg-white font-mono text-sm font-bold text-accent uppercase shadow-sm">
                {u.id.slice(0, 2)}
              </span>
              <span class="min-w-0">
                <span class="block font-medium">{localize(u.name, lang)}</span>
                {#if u.website}
                  <span class="block truncate text-xs text-neutral-500">{u.website.replace(/^https?:\/\//, '')}</span>
                {/if}
              </span>
            </button>
          {/each}
          <div class="flex items-center gap-4 rounded-xl border-2 border-dashed border-neutral-200 p-4 text-sm text-neutral-500">
            <span class="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg bg-neutral-100 text-xl text-neutral-400">+</span>
            <span>
              {t('university.missing')}
              <a class="text-accent underline" href="https://github.com/jacopofilonzi/TimeTable/issues" target="_blank" rel="noopener">
                {t('university.missingLink')}
              </a>
            </span>
          </div>
        </div>
      {:else if current.kind === 'fields' && uni}
        {@const side = sideColumn(current.step)}
        <div class={side ? 'grid gap-6 lg:grid-cols-[1fr_12rem]' : 'space-y-6'}>
          {#each current.step.fields as field (field.key)}
            <FieldInput
              uniId={uni.id}
              {field}
              values={p.values}
              {lang}
              column={side && isChips(field)}
              onselect={(o) => selectOption(field.key, o)}
            />
          {/each}
        </div>
      {:else if current.kind === 'options' && uni}
        <div class="space-y-6">
          <fieldset>
            <legend class="mb-2 text-sm font-medium text-neutral-700">{t('options.weeks')}</legend>
            <div class="flex flex-wrap gap-2" role="radiogroup">
              {#each Array.from({ length: uni.weeks.max - uni.weeks.min + 1 }, (_, i) => uni.weeks.min + i) as n (n)}
                <button
                  type="button"
                  role="radio"
                  aria-checked={p.weeks === n}
                  class="h-10 w-10 cursor-pointer rounded-lg border text-sm transition-colors {p.weeks === n
                    ? 'border border-accent bg-accent/5 font-medium text-accent ring-1 ring-accent'
                    : 'border-neutral-200 hover:border-accent'}"
                  onclick={() => (p.weeks = n)}
                >
                  {n}
                </button>
              {/each}
            </div>
            <p class="mt-2 text-xs text-neutral-500">{t('options.weeksHint')}</p>
          </fieldset>
          <div>
            <label for="name" class="mb-2 block text-sm font-medium text-neutral-700">{t('options.name')}</label>
            <input
              id="name"
              class="input max-w-md"
              maxlength="100"
              bind:value={p.name}
              oninput={() => (p.nameEdited = true)}
            />
            <p class="mt-1 text-xs text-neutral-500">{t('options.nameHint')}</p>
          </div>
        </div>
      {:else if current.kind === 'result' && uni}
        <ResultStep uniId={uni.id} values={p.values} weeks={p.weeks} name={p.name.trim()} {lang} />
      {/if}
      </div>

      <nav class="mt-8 flex items-center justify-between border-t border-neutral-100 pt-6">
        {#if p.index > 0}
          <button type="button" class="btn-ghost" onclick={() => go(p.index - 1)}>← {t('nav.back')}</button>
        {:else}
          <span></span>
        {/if}

        {#if current.kind === 'result'}
          <button type="button" class="btn-secondary" onclick={restart}>{t('result.restart')}</button>
        {:else if current.kind !== 'university'}
          <button type="button" class="btn-primary px-5" disabled={!canNext} onclick={() => go(p.index + 1)}>
            {current.kind === 'options' ? t('nav.create') : t('nav.next')} →
          </button>
        {/if}
      </nav>
    {/if}
  </section>
</div>
