<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import SearchInput from "$lib/SearchInput.svelte";
  import ResultList from "$lib/components/ResultList.svelte";
  import { sanitize } from "$lib/utils";

  let query = "";
  let results: { name: string; path: string }[] = [];
  let selectedIndex = 0;
  let searchInputComponent: SearchInput;
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  const onInput = async () => {
    const sanitized = sanitize(query);
    if (debounceTimer) clearTimeout(debounceTimer);
    selectedIndex = 0;
    if (!sanitized) { results = []; return; }

    debounceTimer = setTimeout(async () => {
      if (searchInputComponent) {
        results = await searchInputComponent.query(sanitized);
      }
    }, 100);
  };

  const onNav = (e: CustomEvent<{ direction: "up" | "down" }>) => {
    if (results.length === 0) return;
    if (e.detail.direction === "down") {
      selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
    } else {
      selectedIndex = Math.max(selectedIndex - 1, 0);
    }
  };

  const onActivate = async () => {
    if (results.length === 0 || selectedIndex < 0) return;
    const selected = results[selectedIndex];
    if (selected) {
      try {
        await invoke("activate", { path: selected.path });
      } catch (e) {
        console.error("activate error:", e);
      }
    }
  };

  const onSelect = async (e: CustomEvent<{ index: number }>) => {
    selectedIndex = e.detail.index;
    await onActivate();
  };

  const onHover = (e: CustomEvent<{ index: number }>) => {
    selectedIndex = e.detail.index;
  };

  onMount(async () => {
    // 备用：用原生 window focus 事件兜底
    const onWindowFocus = () => {
      setTimeout(() => {
        searchInputComponent?.doFocus();
      }, 30);
    };
    window.addEventListener("focus", onWindowFocus);

    // Tauri 窗口焦点监听 —— 方案二的核心
    const win = getCurrentWindow();
    const unlistenFocus = await win.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        // 小延迟等 WebView2 内部焦点管理完成
        setTimeout(() => {
          searchInputComponent?.doFocus();
        }, 30);
      }
    });

    return () => {
      unlistenFocus();
      window.removeEventListener("focus", onWindowFocus);
      if (debounceTimer) clearTimeout(debounceTimer);
    };
  });
</script>

<div class="launcher-container">
  <div class="search-area">
    <SearchInput
      bind:this={searchInputComponent}
      bind:value={query}
      on:input={onInput}
      on:nav={onNav}
      on:activate={onActivate}
      placeholder="输入命令或搜索..."
    />
    <ResultList
      {results}
      {selectedIndex}
      on:select={onSelect}
      on:hover={onHover}
    />
  </div>
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    width: 100%;
    height: 100%;
  }

  .launcher-container {
    width: 100%;
    height: 100vh;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    background: transparent;
    overflow: hidden;
  }

  .search-area {
    width: 100%;
    max-width: 100vw;
  }
</style>
