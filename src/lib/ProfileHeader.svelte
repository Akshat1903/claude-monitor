<script lang="ts">
  import type { ProfileResponse } from "./types";
  import { planLabel, shortName } from "./format";

  let { profile }: { profile: ProfileResponse } = $props();

  const status = $derived(profile.organization.subscription_status);
  const nonActive = $derived(
    status && status.toLowerCase() !== "active" ? status : null
  );
</script>

<div class="flex items-center gap-2 text-[12px]">
  <span class="text-black/60 dark:text-white/60 truncate">
    {shortName(profile)}
  </span>
  <span class="text-black/30 dark:text-white/30">·</span>
  <span
    class="px-2 py-0.5 rounded-full text-[11px] font-medium bg-blue-500/15 text-blue-700 dark:text-blue-300"
    >{planLabel(profile)}</span
  >
  {#if nonActive}
    <span
      class="px-2 py-0.5 rounded-full text-[11px] bg-orange-500/20 text-orange-700 dark:text-orange-300 capitalize"
      >{nonActive}</span
    >
  {/if}
</div>
