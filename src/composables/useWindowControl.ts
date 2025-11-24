import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { nextTick, onMounted, onUnmounted, type Ref } from "vue";
import type { WindowControlOptions } from "../types";

/**
 * 窗口控制 Composable
 *
 * 功能：
 * - 监听 spark_focus_input 事件并自动聚焦输入框
 * - 提供隐藏窗口的方法
 * - 自动管理事件监听的注册和清理
 *
 * @param inputRef 需要自动聚焦的输入框引用
 * @param options 配置选项
 * @returns 窗口控制方法
 */
export function useWindowControl(
  inputRef: Ref<HTMLElement | null>,
  options: WindowControlOptions = {},
) {
  const { autoFocusDelay = 150 } = options;

  let unlistenFn: UnlistenFn | null = null;

  /**
   * 聚焦输入框
   */
  const focusInput = async () => {
    await nextTick();
    inputRef.value?.focus();
  };

  /**
   * 隐藏主窗口
   */
  const hideWindow = async () => {
    try {
      await invoke("hide_main_window");
      console.log("Window hidden");
    } catch (error) {
      console.error("Failed to hide window:", error);
    }
  };

  /**
   * 初始化事件监听
   */
  const initEventListeners = async () => {
    try {
      // 监听 spark_focus_input 事件
      unlistenFn = await listen("spark_focus_input", () => {
        focusInput();
      });
    } catch (error) {
      console.error("Failed to setup event listeners:", error);
    }
  };

  /**
   * 清理事件监听
   */
  const cleanupEventListeners = () => {
    if (unlistenFn) {
      unlistenFn();
      unlistenFn = null;
    }
  };

  // 组件挂载时初始化
  onMounted(async () => {
    await initEventListeners();

    // 延迟自动聚焦
    setTimeout(() => {
      focusInput();
    }, autoFocusDelay);
  });

  // 组件卸载时清理
  onUnmounted(() => {
    cleanupEventListeners();
  });

  return {
    focusInput,
    hideWindow,
  };
}
