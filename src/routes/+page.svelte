<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { tick } from "svelte";
  import SearchInput from "../lib/SearchInput.svelte";
  import ResultList from "../lib/components/ResultList.svelte";

  // 日志工具函数
  const log = (prefix: string, ...args: any[]) => {
    const timestamp = new Date().toISOString().substr(11, 12);
    const msg = `[${timestamp}] [${prefix}]`;
    console.log(msg, ...args);
    // 同时写入页面上的日志区域，方便查看
    const logEl = document.getElementById("debug-log");
    if (logEl) {
      logEl.textContent += msg + " " + args.map(a => typeof a === 'object' ? JSON.stringify(a) : a).join(" ") + "\n";
      logEl.scrollTop = logEl.scrollHeight;
    }
  };

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
    log("onReady", "inputEl set", !!inputEl, "renderKey:", renderKey);
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
    log("focusInput", "START", "inputEl:", !!inputEl, "renderKey:", renderKey);

    await tick();
    const exists = !!inputEl;
    log("focusInput", "after tick", "exists:", exists);

    if (!exists) {
      log("focusInput", "FAIL - no inputEl");
      return false;
    }

    if (!document.hasFocus()) {
      log("focusInput", "document doesn't have focus, calling window.focus()");
      try {
        window.focus();
        await new Promise((r) => setTimeout(r, 20));
      } catch (eWin) {
        log("focusInput", "window.focus error", eWin);
      }
    }

    let preventOk = false;
    let fallbackOk = false;
    try {
      log("focusInput", "calling inputEl.focus({preventScroll:true})");
      (inputEl as any).focus({ preventScroll: true });
      preventOk = true;
      log("focusInput", "preventScroll focus ok");
    } catch (e) {
      log("focusInput", "preventScroll failed, trying regular focus", e);
      try {
        inputEl?.focus();
        fallbackOk = true;
        log("focusInput", "fallback focus ok");
      } catch (e2) {
        log("focusInput", "fallback failed", e2);
        return false;
      }
    }

    const delays = [0, 10, 50, 100, 200];
    let activeIsInput = document.activeElement === inputEl;
    let docHasFocus = document.hasFocus();
    log("focusInput", "initial state", "activeIsInput:", activeIsInput, "docHasFocus:", docHasFocus);

    for (const d of delays) {
      if (activeIsInput && docHasFocus) break;
      await new Promise((r) => setTimeout(r, d));
      try {
        inputEl?.focus();
      } catch {}
      activeIsInput = document.activeElement === inputEl;
      docHasFocus = document.hasFocus();
      log("focusInput", `delay ${d}`, "activeIsInput:", activeIsInput, "docHasFocus:", docHasFocus);
    }

    const len = query.length;
    if (activeIsInput && docHasFocus) {
      try {
        inputEl!.setSelectionRange(len, len);
        log("focusInput", "setSelectionRange ok", "len:", len);
      } catch (e3) {
        log("focusInput", "setSelectionRange error", e3);
      }
    }

    const result = activeIsInput && docHasFocus;
    log("focusInput", "END", "result:", result);
    return result;
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
      log("tryFocus", "START", "retries:", retries, "renderKey:", renderKey, "inputEl:", !!inputEl);
      const win = getCurrentWindow!();
      log("tryFocus", "calling win.setFocus()");
      try {
        await win.setFocus();
        log("tryFocus", "win.setFocus() ok");
      } catch (e) {
        log("tryFocus", "win.setFocus() error", e);
      }
      setTimeout(() => {
        log("tryFocus", "delayed win.setFocus()");
        win.setFocus().catch(() => {});
      }, 50);

      log("tryFocus", "calling focusInput()");
      const ok = await focusInput();
      log("tryFocus", "focusInput result:", ok, "retries:", retries);

      if (!ok && retries < 5) {
        retries += 1;
        log("tryFocus", "retrying in 75ms, retry:", retries);
        setTimeout(tryFocus, 75);
      } else if (ok) {
        log("tryFocus", "SUCCESS");
      } else {
        log("tryFocus", "GIVE UP after max retries");
      }
    };

    let unlisten: (() => void) | undefined;
    if (listen) {
      log("onMount", "setting up event listener");
      unlisten = await listen("focus-search-input", async () => {
        log("EVENT", "focus-search-input received", "renderKey before:", renderKey);
        renderKey += 1;
        log("EVENT", "renderKey incremented to:", renderKey);
        await tick();
        log("EVENT", "tick complete, calling tryFocus()");
        tryFocus();
      });
      log("onMount", "event listener set up");
    }

    onDestroy(() => {
      if (unlisten) unlisten();
      if (debounceTimer) clearTimeout(debounceTimer);
    });
  });
</script>

<!-- 调试日志区域 -->
<div id="debug-log" style="
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0,0,0,0.9);
  color: #0f0;
  font-family: monospace;
  font-size: 12px;
  padding: 10px;
  overflow: auto;
  z-index: 9999;
  display: none;
"></div>

<!-- 点击任意位置隐藏日志，按 L 键显示日志 -->
<svelte:window on:keydown={(e) => {
  if (e.key === 'l' || e.key === 'L') {
    const logEl = document.getElementById('debug-log');
    if (logEl) {
      logEl.style.display = logEl.style.display === 'none' ? 'block' : 'none';
    }
  }
}} />

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