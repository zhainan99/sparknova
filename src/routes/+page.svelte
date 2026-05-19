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

    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }

    selectedIndex = 0;

    if (!query) {
      results = [];
      return;
    }

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
    if (!inputEl) return false;

    // 多次重试确保焦点生效
    const maxRetries = 8;
    for (let i = 0; i < maxRetries; i++) {
      try {
        // 先尝试直接聚焦
        (inputEl as HTMLInputElement).focus();
        // 聚焦后等待一小段时间让系统真正处理
        await new Promise(r => setTimeout(r, 30));

        // 验证焦点是否真的在 input 上
        if (document.activeElement === inputEl) {
          const len = (inputEl as HTMLInputElement).value.length;
          (inputEl as HTMLInputElement).setSelectionRange(len, len);
          return true;
        }
      } catch {}

      // 如果没成功，等待后重试
      await new Promise(r => setTimeout(r, 50));
    }

    // 最后尝试一次 window.focus 再聚焦
    try {
      window.focus();
      await new Promise(r => setTimeout(r, 100));
      inputEl.focus();
      await new Promise(r => setTimeout(r, 50));
      if (document.activeElement === inputEl) {
        const len = (inputEl as HTMLInputElement).value.length;
        inputEl.setSelectionRange(len, len);
        return true;
      }
    } catch {}

    return document.activeElement === inputEl;
  };

  onMount(async () => {
    log("onMount", "START");

    // 动态导入 Tauri API
    const tauriApi = await import("@tauri-apps/api/core");
    const tauriWindow = await import("@tauri-apps/api/window");
    const tauriEvent = await import("@tauri-apps/api/event");

    invoke = tauriApi.invoke;
    getCurrentWindow = tauriWindow.getCurrentWindow;
    listen = tauriEvent.listen;

    log("onMount", "Tauri APIs imported", "listen:", !!listen);

    let retries = 0;
    const tryFocus = async () => {
      const win = getCurrentWindow!();
      try {
        await win.setFocus();
      } catch {}

      // 延迟再聚焦，确保窗口完全就绪
      setTimeout(() => {
        win.setFocus().catch(() => {});
      }, 50);

      const ok = await focusInput();
      if (!ok && retries < 3) {
        retries += 1;
        setTimeout(tryFocus, 100);
      }
    };

    let unlisten: (() => void) | undefined;
    if (listen) {
      unlisten = await listen("focus-search-input", async () => {
        renderKey += 1;
        await tick();
        retries = 0;
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