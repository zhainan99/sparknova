<script lang="ts">
  import { onMount } from 'svelte';
  import SearchInput from '../components/SearchInput.svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';

  // 定义组件实例的类型
  let searchInputComponent: SearchInput | null = null;

  onMount(async () => {
    console.log('+page 组件挂载');

    // 监听激活输入框事件
    await listen('activate-input', () => {
      console.log('activate-input event received');
      searchInputComponent?.focus();
    });

    // 监听窗口隐藏事件
    await listen('window-hidden', () => {
      console.log('window-hidden event received');
      searchInputComponent?.blur();
    });

    // 按 Escape 键隐藏窗口
    const onKeyDown = async (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        console.log('Escape pressed, hiding window');
        await invoke('hide_main_window');
      }
    };
    document.addEventListener('keydown', onKeyDown);

    return () => {
      document.removeEventListener('keydown', onKeyDown);
    };
  });
</script>

<main>
  <SearchInput bind:this={searchInputComponent} />
</main>

<style>
  main {
    width: 100%;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>