<script lang="ts">
  let { message }: { message: string } = $props();
</script>

<div
  class="flex items-start gap-1.5 px-2 py-1.5 rounded-md bg-orange-500/15 text-[12px]"
>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 24 24"
    fill="currentColor"
    class="w-3.5 h-3.5 text-orange-500 mt-[1px] shrink-0"
  >
    <path
      d="M12 2L1 21h22L12 2zm0 6l7.53 11H4.47L12 8zm-1 3v4h2v-4h-2zm0 5v2h2v-2h-2z"
    />
  </svg>
  <span class="text-black/80 dark:text-white/80">{message}</span>
</div>
