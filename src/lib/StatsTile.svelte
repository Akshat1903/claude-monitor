<script lang="ts">
  import type { StatsSnapshot } from "./types";
  import { shortTokens } from "./format";

  let { stats }: { stats: StatsSnapshot } = $props();
</script>

<div
  class="flex items-stretch rounded-lg bg-black/5 dark:bg-white/5 overflow-hidden"
>
  <div class="flex-1 flex flex-col items-center py-2">
    <span class="text-[15px] font-semibold tabular-nums"
      >{shortTokens(stats.today_tokens)}</span
    >
    <span class="text-[10px] text-black/60 dark:text-white/60">Today</span>
  </div>
  <div class="w-px bg-black/10 dark:bg-white/10"></div>
  <div class="flex-1 flex flex-col items-center py-2">
    <span class="text-[15px] font-semibold tabular-nums"
      >{shortTokens(stats.week_tokens)}</span
    >
    <span class="text-[10px] text-black/60 dark:text-white/60">Week</span>
  </div>
  <div class="w-px bg-black/10 dark:bg-white/10"></div>
  <div class="flex-1 flex flex-col items-center py-2">
    <span class="text-[14px] font-semibold truncate max-w-[100px]"
      >{stats.favorite_model ?? "—"}</span
    >
    <span class="text-[10px] text-black/60 dark:text-white/60">Favorite</span>
  </div>
</div>
