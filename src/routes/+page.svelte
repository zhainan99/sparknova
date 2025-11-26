<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount, onDestroy } from "svelte";
  import { tick } from "svelte";
  import SearchInput from "../lib/SearchInput.svelte";

  let query = "";
  let inputEl: HTMLInputElement | null = null;
  let renderKey = 0;

  const onReady = (e: CustomEvent<{ inputEl: HTMLInputElement }>) => {
    inputEl = e.detail.inputEl;
  };

  const sanitize = (v: string) => v.replace(/[<>]/g, "").trim().slice(0, 256);
  const onInput = (e: Event) => {
    const t = e.target as HTMLInputElement;
    query = sanitize(t.value);
  };

  const focusInput = async () => {
    await tick();
    const exists = !!inputEl;
    console.log("focusInput:start", { exists });
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
      console.warn("focusInput:preventScroll error", e);
      try {
        inputEl?.focus();
        fallbackOk = true;
      } catch (e2) {
        console.error("focusInput:fallback error", e2);
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
      console.log("focusInput:retry", { d, activeIsInput, docHasFocus });
    }

    const len = query.length;
    let selectionOk = false;
    if (activeIsInput && docHasFocus) {
      try {
        inputEl!.setSelectionRange(len, len);
        selectionOk = true;
        console.log("focusInput:setSelectionRange", { len, selectionOk });
      } catch (e3) {
        console.warn("focusInput:setSelectionRange error", e3);
      }
    }

    console.log("focusInput:end", {
      preventOk,
      fallbackOk,
      activeIsInput,
      docHasFocus,
      selectionOk,
    });
    return activeIsInput && docHasFocus;
  };

  onMount(() => {
    let retries = 0;
    const tryFocus = async () => {
      console.log("tryFocus:start", { retries });
      const win = getCurrentWindow();
      try {
        const visible = await win.isVisible();
        console.log("tryFocus:window visibility", { visible });
      } catch (eVis) {
        console.warn("tryFocus:isVisible error", eVis);
      }
      try {
        await win.setFocus();
        setTimeout(() => {
          win
            .setFocus()
            .catch((e2) => console.warn("tryFocus:setFocus second error", e2));
        }, 50);
        console.log("tryFocus:setFocus ok");
      } catch (e) {
        console.warn("tryFocus:setFocus error", e);
      }
      const ok = await focusInput();
      console.log("tryFocus:result", { ok });
      if (!ok && retries < 5) {
        retries += 1;
        console.log("tryFocus:retry", { retries });
        setTimeout(tryFocus, 75);
      }
    };

    let unlisten: (() => void) | undefined;
    const isTauriEnv =
      typeof window !== "undefined" && (window as any).__TAURI_INTERNALS__;
    if (isTauriEnv) {
      listen("focus-search-input", async () => {
        console.log("listen: focus-search-input received");
        renderKey += 1;
        await tick();
        tryFocus();
      })
        .then((u) => {
          unlisten = u;
          console.log("listen: subscribed focus-search-input");
        })
        .catch((e) => {
          console.error("listen: subscribe error", e);
        });
    }

    onDestroy(() => {
      if (unlisten) {
        try {
          unlisten();
          console.log("listen: unsubscribed focus-search-input");
        } catch (e) {
          console.warn("listen: unsubscribe error", e);
        }
      }
    });
  });
</script>

<div class="app-container">
  <div class="search-input-container">
    {#key renderKey}
      <SearchInput
        bind:value={query}
        on:input={onInput}
        on:ready={onReady}
        placeholder="输入命令或搜索..."
      />
    {/key}
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
