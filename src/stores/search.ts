import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { SearchResultItem } from "../types";

/**
 * 搜索状态管理 Store
 */
export const useSearchStore = defineStore(
  "search",
  () => {
    // 状态
    const query = ref<string>("");
    const results = ref<SearchResultItem[]>([]);
    const isSearching = ref<boolean>(false);
    const history = ref<string[]>([]);

    // 计算属性
    const hasQuery = computed(() => query.value.trim().length > 0);
    const hasResults = computed(() => results.value.length > 0);

    // 动作
    /**
     * 设置搜索查询
     */
    function setQuery(value: string) {
      query.value = value;
    }

    /**
     * 清空搜索查询
     */
    function clearQuery() {
      query.value = "";
      results.value = [];
    }

    /**
     * 执行搜索
     */
    async function search(searchQuery?: string) {
      const q = searchQuery || query.value;
      if (!q.trim()) {
        results.value = [];
        return;
      }

      isSearching.value = true;

      try {
        // TODO: 实现实际的搜索逻辑
        // 这里可以集成 Fuse.js 进行模糊搜索
        console.log("Searching for:", q);

        // 模拟搜索结果
        await new Promise((resolve) => setTimeout(resolve, 100));
        results.value = [];

        // 添加到搜索历史
        addToHistory(q);
      } finally {
        isSearching.value = false;
      }
    }

    /**
     * 添加到搜索历史
     */
    function addToHistory(searchQuery: string) {
      const trimmed = searchQuery.trim();
      if (!trimmed) return;

      // 移除重复项
      history.value = history.value.filter((item) => item !== trimmed);

      // 添加到开头
      history.value.unshift(trimmed);

      // 限制历史记录数量
      if (history.value.length > 50) {
        history.value = history.value.slice(0, 50);
      }
    }

    /**
     * 清空搜索历史
     */
    function clearHistory() {
      history.value = [];
    }

    return {
      // 状态
      query,
      results,
      isSearching,
      history,

      // 计算属性
      hasQuery,
      hasResults,

      // 动作
      setQuery,
      clearQuery,
      search,
      addToHistory,
      clearHistory,
    };
  },
  {
    // 持久化配置
    persist: {
      key: "sparknova-search",
      storage: localStorage,
    },
  },
);
