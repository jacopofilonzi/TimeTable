<script lang="ts">
  import { ApiError, clearCache, type ClearReport, type RedisClear } from '../lib/api';
  import { useT, type Lang } from '../lib/i18n';

  let { lang }: { lang: Lang } = $props();
  const t = $derived(useT(lang));

  let token = $state('');
  let running = $state(false);
  let report = $state<ClearReport | null>(null);
  let error = $state<string | null>(null);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    running = true;
    report = null;
    error = null;
    try {
      report = await clearCache(token.trim());
    } catch (e) {
      error =
        e instanceof ApiError && e.status === 401
          ? t('admin.unauthorized')
          : e instanceof ApiError && e.status === 404
            ? t('admin.disabled')
            : e instanceof Error
              ? e.message
              : t('error.generic');
    } finally {
      running = false;
    }
  }

  function redisText(redis: RedisClear): string {
    switch (redis.status) {
      case 'disabled':
        return t('admin.redisDisabled');
      case 'not_connected':
        return t('admin.redisNotConnected');
      case 'cleared':
        return t('admin.redisCleared', { n: redis.keys });
      case 'failed':
        return t('admin.redisFailed', { error: redis.error });
    }
  }
</script>

<div class="p-6 sm:p-10">
  <p class="text-xs font-semibold tracking-wide text-accent uppercase">{t('admin.title')}</p>
  <h1 class="mt-1 text-2xl font-semibold">{t('admin.cacheTitle')}</h1>
  <!-- <p class="mt-1 max-w-xl text-sm text-neutral-500">{t('admin.cacheDescription')}</p> -->

  <form class="mt-6 max-w-md space-y-4" onsubmit={submit}>
    <div>
      <label for="token" class="mb-2 block text-sm font-medium text-neutral-700">{t('admin.token')}</label>
      <input
        id="token"
        type="password"
        class="input font-mono"
        autocomplete="off"
        spellcheck="false"
        required
        bind:value={token}
      />
      <!-- <p class="mt-1 text-xs text-neutral-500">{t('admin.tokenHint')}</p> -->
    </div>
    <button type="submit" class="btn-primary px-5" disabled={running || !token.trim()}>
      {running ? t('admin.running') : t('admin.submit')}
    </button>
  </form>

  {#if error}
    <p class="mt-6 max-w-md rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700" role="alert">
      {error}
    </p>
  {:else if report}
    <div class="mt-6 max-w-md rounded-lg border-l-4 border-accent bg-neutral-50 px-4 py-3 text-sm" role="status">
      <p class="font-medium text-accent">✓ {t('admin.done')}</p>
      <dl class="mt-2 grid grid-cols-[auto_1fr] gap-x-4 gap-y-1">
        <dt class="text-neutral-500">{t('admin.memory')}</dt>
        <dd class="font-mono">{report.memory}</dd>
        <dt class="text-neutral-500">{t('admin.redis')}</dt>
        <dd class={report.redis.status === 'failed' ? 'text-red-700' : ''}>{redisText(report.redis)}</dd>
      </dl>
    </div>
  {/if}
</div>
