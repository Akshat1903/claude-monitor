<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount, onDestroy } from "svelte";
  import type { AppSnapshot } from "$lib/types";
  import { fiveHourPace, relativeFromNow } from "$lib/format";
  import GaugeRow from "$lib/GaugeRow.svelte";
  import ProfileHeader from "$lib/ProfileHeader.svelte";
  import StatsTile from "$lib/StatsTile.svelte";
  import ErrorBanner from "$lib/ErrorBanner.svelte";

  let snapshot = $state<AppSnapshot | null>(null);
  let loading = $state(false);
  let clockTick = $state(0);
  let tickTimer: ReturnType<typeof setInterval> | null = null;

  async function loadCached() {
    snapshot = await invoke<AppSnapshot>("get_snapshot");
  }

  async function refresh() {
    if (loading) return;
    loading = true;
    try {
      snapshot = await invoke<AppSnapshot>("refresh");
    } catch (e) {
      console.error(e);
    } finally {
      loading = false;
    }
  }

  function hide() {
    getCurrentWindow().hide();
  }

  onMount(async () => {
    await loadCached();

    const unlisten = await listen<AppSnapshot>("snapshot-updated", (e) => {
      snapshot = e.payload;
    });
    tickTimer = setInterval(() => {
      clockTick++;
    }, 30_000);

    onDestroy(() => {
      unlisten();
      if (tickTimer) clearInterval(tickTimer);
    });
  });

  const pace = $derived(fiveHourPace(snapshot?.usage?.five_hour ?? null));
  const updatedLabel = $derived(
    snapshot?.fetched_at
      ? `Updated ${relativeFromNow(snapshot.fetched_at)}`
      : ""
  );
  $effect(() => {
    void clockTick;
  });
</script>

<main
  class="p-4 flex flex-col gap-3 rounded-[14px] bg-white dark:bg-zinc-900
         text-black dark:text-white
         border border-black/10 dark:border-white/10"
>
  <header class="flex items-center">
    <h1 class="text-sm font-semibold">Claude Usage</h1>
    <span class="flex-1"></span>
    <button
      aria-label="Refresh"
      class="w-6 h-6 grid place-items-center rounded
             text-black/70 dark:text-white/70 hover:bg-black/5 dark:hover:bg-white/10
             disabled:opacity-40"
      disabled={loading}
      onclick={refresh}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        fill="currentColor"
        class={"w-4 h-4 " + (loading ? "animate-spin" : "")}
      >
        <path d="M17.65 6.35A8 8 0 1 0 19.73 14h-2.13a6 6 0 1 1-1.42-6.35L13 11h7V4l-2.35 2.35z" />
      </svg>
    </button>
  </header>

  {#if snapshot?.profile}
    <ProfileHeader profile={snapshot.profile} />
  {/if}

  {#if snapshot?.stats}
    <StatsTile stats={snapshot.stats} />
  {/if}

  {#if snapshot?.usage}
    {#key clockTick}
      <GaugeRow
        title="5-hour window"
        block={snapshot.usage.five_hour}
        {pace}
      />
      <GaugeRow title="Weekly" block={snapshot.usage.seven_day} />
      {#if snapshot.usage.seven_day_omelette}
        <GaugeRow
          title="Claude Design · weekly"
          block={snapshot.usage.seven_day_omelette}
        />
      {/if}
    {/key}
  {:else if !snapshot?.error}
    <div class="py-6 flex items-center justify-center text-sm opacity-60">
      Loading…
    </div>
  {/if}

  {#if snapshot?.error}
    <ErrorBanner message={snapshot.error} />
  {/if}

  <footer
    class="flex items-center pt-1 border-t border-black/10 dark:border-white/10"
  >
    <span class="text-[11px] text-black/50 dark:text-white/50 flex-1"
      >{updatedLabel}</span
    >
    <button
      class="text-[12px] text-black/60 dark:text-white/60 hover:underline"
      onclick={hide}
    >
      Close
    </button>
  </footer>
</main>

<style>
  :global(html),
  :global(body) {
    background: transparent;
  }
</style>
