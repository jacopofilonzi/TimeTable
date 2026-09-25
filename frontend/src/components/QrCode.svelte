<script lang="ts">
  let { url, hint, errorText }: { url: string; hint: string; errorText: string } = $props();

  let src = $state<string | null>(null);
  let failed = $state(false);

  $effect(() => {
    let cancelled = false;
    src = null;
    failed = false;
    // Loaded lazily: only needed when the dialog is opened.
    import('qrcode')
      .then(({ default: QRCode }) => QRCode.toDataURL(url, { width: 560, margin: 1, errorCorrectionLevel: 'M' }))
      .then((data) => {
        if (!cancelled) src = data;
      })
      .catch((err) => {
        console.error('QR code generation failed', err);
        if (!cancelled) failed = true;
      });
    return () => (cancelled = true);
  });
</script>

<p class="text-center text-sm text-neutral-500">{hint}</p>
<div class="mx-auto mt-4 aspect-square w-full max-w-72">
  {#if src}
    <img {src} alt="QR code" class="h-full w-full [image-rendering:pixelated]" />
  {:else if failed}
    <div class="flex h-full w-full items-center justify-center rounded-lg bg-red-50 p-6 text-center text-sm text-red-700">
      {errorText}
    </div>
  {:else}
    <div class="h-full w-full animate-pulse rounded-lg bg-neutral-100"></div>
  {/if}
</div>
