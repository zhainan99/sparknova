<script setup lang="ts">
import { SearchOutline } from "@vicons/ionicons5";
import { NInput } from "naive-ui";
import { ref } from "vue";
import { useWindowControl } from "../composables/useWindowControl";
import { useSearchStore } from "../stores/search";

// 输入框引用
const inputRef = ref<HTMLElement | null>(null);

// 使用 Pinia store
const searchStore = useSearchStore();

// 使用窗口控制 composable
const { hideWindow } = useWindowControl(inputRef);

/**
 * 处理搜索
 */
const handleSearch = async () => {
  if (searchStore.hasQuery) {
    await searchStore.search();
    console.log("Searching for:", searchStore.query);
    // TODO: 显示搜索结果
  }
};

/**
 * 处理 Esc 键
 */
const handleEscape = () => {
  searchStore.clearQuery();
  hideWindow();
};
</script>

<template>
  <div class="search-input-container">
    <NInput
      ref="inputRef"
      v-model:value="searchStore.query"
      type="text"
      size="large"
      placeholder="输入命令或搜索..."
      clearable
      @keyup.enter="handleSearch"
      @keyup.esc="handleEscape"
    >
      <template #prefix>
        <n-icon :component="SearchOutline" />
      </template>
    </NInput>
  </div>
</template>

<style scoped>
.search-input-container {
  width: 100%;
}

/* 自定义 Naive UI 输入框样式 - 无边框设计 */
:deep(.n-input) {
  /* 尺寸和字体 */
  --n-border-radius: 0;
  --n-height: 60px;
  --n-font-size: 18px;

  /* 背景色 */
  --n-color: rgba(255, 255, 255, 0.95);
  --n-color-focus: rgba(255, 255, 255, 1);
  --n-color-disabled: rgba(255, 255, 255, 0.5);

  /* 移除所有边框 */
  --n-border: 0 solid transparent;
  --n-border-hover: 0 solid transparent;
  --n-border-focus: 0 solid transparent;
  --n-border-disabled: 0 solid transparent;

  /* 移除所有阴影 */
  --n-box-shadow-focus: none;

  /* 移除边框警告色 */
  --n-border-warning: 0 solid transparent;
  --n-border-warning-hover: 0 solid transparent;
  --n-border-warning-focus: 0 solid transparent;

  /* 移除边框错误色 */
  --n-border-error: 0 solid transparent;
  --n-border-error-hover: 0 solid transparent;
  --n-border-error-focus: 0 solid transparent;
}

/* 直接隐藏边框元素 */
:deep(.n-input__border),
:deep(.n-input__state-border) {
  display: none !important;
}

:deep(.n-input__placeholder) {
  color: #999;
}

:deep(.n-input__prefix) {
  margin-left: 16px;
  color: #666;
}

:deep(.n-input__suffix) {
  margin-right: 16px;
}
</style>
