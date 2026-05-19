<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  export let value: string = "";
  export let placeholder: string = "输入命令或搜索...";

  const dispatch = createEventDispatcher();
  let inputEl: HTMLInputElement;

  interface ResultItem {
    name: string;
    path: string;
  }

  onMount(async () => {
    await tick();
    dispatch("ready", { inputEl });
  });

  // 暴露一个方法，父组件可调用以将焦点置于末尾
  export function focusToEnd(len: number) {
    try {
      inputEl?.focus({ preventScroll: true });
    } catch {
      inputEl?.focus();
    }
    try {
      inputEl?.setSelectionRange(len, len);
    } catch {}
  }

  const onInput = (e: Event) => {
    dispatch("input", e);
  };

  const onKeydown = async (e: KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      dispatch("nav", { direction: "down" });
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      dispatch("nav", { direction: "up" });
    } else if (e.key === "Enter") {
      e.preventDefault();
      dispatch("activate");
    }
  };

  // 调用 Rust query 命令获取搜索结果
  export async function query(q: string): Promise<ResultItem[]> {
    try {
      const results = await invoke<ResultItem[]>("query", { q });
      return results;
    } catch (e) {
      console.error("query error:", e);
      return [];
    }
  }
</script>

<input
  bind:value
  bind:this={inputEl}
  on:input={onInput}
  on:keydown={onKeydown}
  tabindex="0"
  type="text"
  class="search-input"
  {placeholder}
  autocomplete="off"
  autocorrect="off"
  autocapitalize="none"
  spellcheck="false"
  aria-label="命令或搜索"
/>

<style>
  .search-input {
    width: 100%;
    height: 80px;
    font-size: 18px;
    border: 0;
    outline: none;
    padding: 0 16px;
    background: rgba(255, 255, 255, 0.95);
    box-sizing: border-box;
  }
</style>
