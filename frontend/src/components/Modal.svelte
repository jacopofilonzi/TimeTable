<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    open = $bindable(false),
    title,
    closeLabel,
    wide = false,
    children,
  }: {
    open?: boolean;
    title: string;
    closeLabel: string;
    /** Wider dialog, for content like the calendar grid. */
    wide?: boolean;
    children: Snippet;
  } = $props();

  let dialog: HTMLDialogElement;

  $effect(() => {
    if (open && !dialog.open) dialog.showModal();
    if (!open && dialog.open) dialog.close();
  });
</script>

<dialog
  bind:this={dialog}
  class="m-auto max-h-[calc(100dvh-2rem)] rounded-2xl bg-white p-0 shadow-2xl backdrop:bg-black/50 {wide
    ? 'w-[min(56rem,calc(100vw-2rem))]'
    : 'w-[min(24rem,calc(100vw-2rem))]'}"
  onclose={() => (open = false)}
  onclick={(e) => e.target === dialog && (open = false)}
>
  <div class="flex items-center justify-between gap-4 border-b border-neutral-100 px-5 py-4">
    <h2 class="font-semibold">{title}</h2>
    <button
      type="button"
      class="-mr-2 flex h-8 w-8 cursor-pointer items-center justify-center rounded-full text-neutral-400 hover:bg-neutral-100 hover:text-neutral-700"
      aria-label={closeLabel}
      onclick={() => (open = false)}
    >
      ✕
    </button>
  </div>
  <div class="p-5">
    <!-- Content is mounted only while open, so it loads lazily. -->
    {#if open}{@render children()}{/if}
  </div>
</dialog>
