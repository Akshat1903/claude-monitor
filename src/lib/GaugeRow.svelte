<script lang="ts">
  import type { PaceZone, UsageBlock } from "./types";
  import { secondsUntil, shortDuration } from "./format";

  let {
    title,
    block,
    pace = null,
  }: { title: string; block: UsageBlock | null; pace?: PaceZone | null } = $props();

  const pct = $derived(block?.utilization ?? null);
  const remaining = $derived(secondsUntil(block?.resets_at ?? null));

  function barColor(p: number): string {
    if (p >= 90) return "bg-red-500";
    if (p >= 70) return "bg-orange-500";
    return "bg-blue-500";
  }

  const paceMeta: Record<PaceZone, { label: string; tone: string }> = {
    chill: { label: "Chill", tone: "text-green-600 bg-green-500/15" },
    "on-track": { label: "On track", tone: "text-blue-600 bg-blue-500/15" },
    hot: { label: "Hot", tone: "text-red-600 bg-red-500/15" },
  };
</script>

<div class="flex flex-col gap-1">
  <div class="flex items-center gap-2">
    <span class="text-sm font-medium">{title}</span>
    {#if pace}
      <span
        class="text-[10px] px-1.5 py-0.5 rounded-full font-medium {paceMeta[pace].tone}"
        >{paceMeta[pace].label}</span
      >
    {/if}
    <span class="flex-1"></span>
    {#if pct !== null}
      <span class="text-sm tabular-nums font-semibold">{Math.round(pct)}%</span>
    {/if}
  </div>
  <div class="h-2 rounded-full bg-black/10 dark:bg-white/10 overflow-hidden">
    <div
      class="h-full rounded-full transition-all {barColor(pct ?? 0)}"
      style="width: {Math.min(100, Math.max(0, pct ?? 0))}%"
    ></div>
  </div>
  {#if remaining !== null}
    <div class="text-[11px] text-black/60 dark:text-white/60">
      Resets in {shortDuration(remaining)}
    </div>
  {/if}
</div>
