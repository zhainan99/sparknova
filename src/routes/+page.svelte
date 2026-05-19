<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { tick } from "svelte";
  import SearchInput from "../lib/SearchInput.svelte";
  import ResultList from "../lib/components/ResultList.svelte";

  // 动态导入 Tauri API，避免 SSR 问题
  let listen: ((event: string, callback: () => void) => Promise<() => void>) | null = null;
  let getCurrentWindow: (() => any) | null = null;
  let invoke: ((cmd: string, args?: any) => Promise<any>) | null = null;

  let query = "";
  let results: { name: string; path: string }[] = [];
  let selectedIndex = 0;
  let inputEl: HTMLInputElement | null = null;
  let renderKey = 0;
  let searchInputComponent: SearchInput;

  const sanitize = (v: string) => v.replace(/[<>]/g, "").trim().slice(0, 256);

  // debounce timer
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  const onReady = (e: CustomEvent<{ inputEl: HTMLInputElement }>) => {
    inputEl = e.detail.inputEl;
  };

  const onInput = async (e: Event) => {
    const t = e.target as HTMLInputElement;
    query = sanitize(t.value);

    // Clear previous timer
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }

    // Reset selected index when query changes
    selectedIndex = 0;

    if (!query) {
      results = [];
      return;
    }

    // Debounce 100ms
    debounceTimer = setTimeout(async () => {
      if (searchInputComponent) {
        results = await searchInputComponent.query(query);
        selectedIndex = 0;
      }
    }, 100);
  };

  // Keyboard navigation
  const onNav = (e: CustomEvent<{ direction: "up" | "down" }>) => {
    const { direction } = e.detail;
    if (results.length === 0) return;

    if (direction === "down") {
      selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
    } else {
      selectedIndex = Math.max(selectedIndex - 1, 0);
    }
  };

  // Activate selected item on Enter
  const onActivate = async () => {
    if (results.length === 0 || selectedIndex < 0) return;
    if (!invoke) return;

    const selected = results[selectedIndex];
    if (selected) {
      try {
        await invoke("activate", { path: selected.path });
      } catch (e) {
        console.error("activate error:", e);
      }
    }
  };

  // Handle result item click
  const onSelect = async (e: CustomEvent<{ index: number }>) => {
    selectedIndex = e.detail.index;
    if (!invoke) return;

    const selected = results[selectedIndex];
    if (selected) {
      try {
        await invoke("activate", { path: selected.path });
      } catch (e) {
        console.error("activate error:", e);
      }
    }
  };

  // Handle mouse hover on result items
  const onHover = (e: CustomEvent<{ index: number }>) => {
    selectedIndex = e.detail.index;
  };

  const focusInput = async () => {
    await tick();
    const exists = !!inputEl;
    if (!exists) return false;

    if (!document.hasFocus()) {
      try {
        window.focus();
        await new Promise((r) => setTimeout(r, 20));
      } catch (eWin) {
        console.warn("focusInput: window.focus error", eWin);
      }
    }

    let preventOk = false;
    let fallbackOk = false;
    try {
      (inputEl as any).focus({ preventScroll: true });
      preventOk = true;
    } catch (e) {
      try {
        inputEl?.focus();
        fallbackOk = true;
      } catch (e2) {
        return false;
      }
    }

    const delays = [0, 10, 50, 100, 200];
    let activeIsInput = document.activeElement === inputEl;
    let docHasFocus = document.hasFocus();
    for (const d of delays) {
      if (activeIsInput && docHasFocus) break;
      await new Promise((r) => setTimeout(r, d));
      try {
        inputEl?.focus();
      } catch {}
      activeIsInput = document.activeElement === inputEl;
      docHasFocus = document.hasFocus();
    }

    const len = query.length;
    if (activeIsInput && docHasFocus) {
      try {
        inputEl!.setSelectionRange(len, len);
      } catch (e3) {
        // ignore
      }
    }
    return activeIsInput && docHasFocus;
  };

  onMount(async () => {
    // 动态导入 Tauri API
    const tauriApi = await import("@tauri-apps/api/core");
    const tauriWindow = await import("@tauri-apps/api/window");
    const tauriEvent = await import("@tauri-apps/api/event");

    invoke = tauriApi.invoke;
    getCurrentWindow = tauriWindow.getCurrentWindow;
    listen = tauriEvent.listen;

    let retries = 0;
    const tryFocus = async () => {
      const win = getCurrentWindow!();
      try {
        await win.setFocus();
        setTimeout(() => {
          win.setFocus().catch(() => {});
        }, 50);
      } catch (e) {
        // ignore
      }
      const ok = await focusInput();
      if (!ok && retries < 5) {
        retries += 1;
        setTimeout(tryFocus, 75);
      }
    };

    let unlisten: (() => void) | undefined;
    if (listen) {
      unlisten = await listen("focus-search-input", async () => {
        renderKey += 1;
        await tick();
        tryFocus();
      });
    }

    onDestroy(() => {
      if (unlisten) unlisten();
      if (debounceTimer) clearTimeout(debounceTimer);
    });
  });
</script>

<div class="app-container">
  <div class="search-input-container">
    {#key renderKey}
      <SearchInput
        bind:this={searchInputComponent}
        bind:value={query}
        on:input={onInput}
        on:ready={onReady}
        on:nav={onNav}
        on:activate={onActivate}
        placeholder="输入命令或搜索..."
      />
    {/key}
    <ResultList
      {results}
      {selectedIndex}
      on:select={onSelect}
      on:hover={onHover}
    />
  </div>
</div>

<style>
  /* 确保页面不出现滚动条 */
  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    width: 100%;
    height: 100%;
  }

  .app-container {
    width: 100%;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    overflow: hidden;
  }

  .search-input-container {
    width: 100%;
    max-width: 100vw;
  }
</style>
