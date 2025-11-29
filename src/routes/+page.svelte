<script lang="ts">
  import { onMount } from 'svelte';
  import SearchInput from '../components/SearchInput.svelte';
  import { listen } from '@tauri-apps/api/event';
  import { formatTime } from '$utils/time';

  // 定义组件实例的类型
  let searchInputComponent: SearchInput | null = null;

  onMount(async () => {
    console.log(`[${formatTime(new Date())}] +page 组件挂载`);

    // 监听激活输入框事件
    await listen('activate-input', () => {
      console.log(`[${formatTime(new Date())}] activate-input event received`);
      // 激活搜索框
      searchInputComponent?.focus();
    });

    // 监听窗口隐藏事件
    await listen('window-hidden', () => {
      console.log(`[${formatTime(new Date())}] window-hidden event received`);
      // 调用搜索框失焦方法
      searchInputComponent?.blur();
    });
  });
</script>

<main>
  <!-- 只保留搜索框组件 -->
  <SearchInput bind:this={searchInputComponent} />
</main>

<style>
  /* 重置默认样式 */
  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  main {
    width: 100%;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>
