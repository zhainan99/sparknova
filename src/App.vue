<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

const searchInput = ref("");
const inputRef = ref<HTMLInputElement | null>(null);

// 组件挂载后自动聚焦输入框
const customUnlistenPromise = listen("spark_focus_input", () => {
  nextTick().then(() => {
    inputRef.value?.focus();
  });
});

onUnmounted(() => {
  customUnlistenPromise.then((fn) => fn());
});

onMounted(() => {
  // 初次挂载后延迟聚焦
  setTimeout(() => {
    nextTick().then(() => {
      inputRef.value?.focus();
    });
  }, 150);
});

// 处理搜索
function handleSearch() {
  if (searchInput.value.trim()) {
    console.log("Searching for:", searchInput.value);
    // TODO: 实现搜索逻辑
  }
}

// Esc 键隐藏窗口
function handleEscape() {
  searchInput.value = "";
  invoke("hide_main_window");
}
</script>

<template>
  <div class="spark-container">
    <div class="search-box">
      <input
        ref="inputRef"
        v-model="searchInput"
        type="text"
        class="search-input"
        placeholder="输入命令或搜索..."
        @keyup.enter="handleSearch"
        @keyup.esc="handleEscape"
        autofocus
      />
    </div>
  </div>
</template>

<style scoped>
.spark-container {
  width: 100%;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
}

.search-box {
  width: 100%;
  padding: 0 10px;
}

.search-input {
  width: 100%;
  height: 50px;
  padding: 0 20px;
  font-size: 18px;
  border: none;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.95);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  outline: none;
  transition: all 0.3s ease;
}

.search-input:focus {
  box-shadow: 0 6px 30px rgba(0, 0, 0, 0.25);
  background: rgba(255, 255, 255, 1);
}

.search-input::placeholder {
  color: #999;
}
</style>
<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  font-family:
    -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue",
    Arial, sans-serif;
}

body {
  background: transparent;
  overflow: hidden;
}

#app {
  width: 100%;
  height: 60px;
}
</style>
