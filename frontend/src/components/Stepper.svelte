<script lang="ts">
  let {
    labels,
    current,
    ongo,
  }: {
    labels: string[];
    current: number;
    ongo: (index: number) => void;
  } = $props();
</script>

<ol class="space-y-1 text-sm">
  {#each labels as label, i (i)}
    {@const done = i < current}
    {@const active = i === current}
    <li>
      <button
        type="button"
        class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left transition-colors {active
          ? 'bg-white font-medium shadow-sm'
          : done
            ? 'cursor-pointer text-neutral-600 hover:bg-white/60'
            : 'cursor-default text-neutral-400'}"
        disabled={!done}
        aria-current={active ? 'step' : undefined}
        onclick={() => ongo(i)}
      >
        <span
          class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full text-xs {active
            ? 'bg-accent text-white'
            : done
              ? 'bg-accent/15 text-accent'
              : 'bg-neutral-200 text-neutral-500'}"
        >
          {#if done}✓{:else}{i + 1}{/if}
        </span>
        {label}
      </button>
    </li>
  {/each}
</ol>
