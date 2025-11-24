/**
 * TypeScript 类型定义
 */

/**
 * 搜索结果项
 */
export interface SearchResultItem {
  id: string;
  title: string;
  description?: string;
  icon?: string;
  type: 'app' | 'file' | 'command' | 'plugin';
  path?: string;
  score?: number;
}

/**
 * 搜索选项
 */
export interface SearchOptions {
  query: string;
  limit?: number;
  threshold?: number;
}

/**
 * 窗口控制选项
 */
export interface WindowControlOptions {
  autoFocusDelay?: number;
}
