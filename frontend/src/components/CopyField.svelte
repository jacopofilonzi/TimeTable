<script lang="ts">
  let { value, copyLabel, copiedLabel }: { value: string; copyLabel: string; copiedLabel: string } = $props();

  let copied = $state(false);
  let box: HTMLElement;

  async function copy() {
    try {
      await navigator.clipboard.writeText(value);
    } catch {
      // Clipboard API unavailable (e.g. plain http on a LAN address): select the text instead.
      getSelection()?.selectAllChildren(box);
      return;
    }
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }
</script>

<div class="flex flex-col overflow-hidden rounded-lg bg-code shadow-[rgba(0,0,0,0.35)_0px_5px_15px] sm:flex-row sm:items-stretch">
  <code bind:this={box} class="flex-1 px-4 py-3 font-mono text-[0.85rem] font-semibold break-all text-code-text select-all">
    {value}
  </code>
  <button
    type="button"
    class="shrink-0 cursor-pointer bg-accent py-3 text-sm font-medium text-white hover:bg-accent-hover sm:w-24 sm:py-0"
    onclick={copy}
  >
    {copied ? copiedLabel : copyLabel}
  </button>
</div>
