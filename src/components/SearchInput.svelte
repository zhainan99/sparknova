<script lang="ts">
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  // 使用组件引用获取输入框元素
  let inputElement: HTMLInputElement;

  // 焦点到输入框的函数
  async function focusInput() {
    try {
      if (inputElement) {
        // 使用 requestAnimationFrame 确保 DOM 更新完成
        requestAnimationFrame(async () => {
          await getCurrentWebview().setFocus();
          inputElement.focus();
          console.log('输入框焦点设置成功');

          // 设置光标到末尾
          if (
            inputElement.selectionStart !== null &&
            inputElement.selectionEnd !== null
          ) {
            inputElement.setSelectionRange(
              inputElement.value.length,
              inputElement.value.length
            );
          }
        });
      }
    } catch (error) {
      console.error('聚焦输入框失败:', error);
    }
  }

  // 暴露focus方法给外部调用（调用Rust后端的ensure_window_focus）
  export async function focus() {
    try {
      focusInput();
    } catch (error) {
      console.error('聚焦失败:', error);
    }
  }

  // 暴露blur方法给外部调用
  export function blur() {
    if (inputElement) {
      document.body.blur();
      console.log('输入框已失焦');
    }
  }
</script>

<div class="search-container">
  <input
    type="text"
    placeholder="搜索..."
    id="search-input"
    class="search-input"
    bind:this={inputElement}
  />
</div>

<style>
  .search-container {
    position: relative;
    width: 100%;
    height: 100%;
    margin: 0;
    padding: 0;
  }

  .search-input {
    width: 100%;
    height: 100%;
    padding: 0 8px;
    font-size: 20px;
    border: 1px solid #e1e5e9;
    border-radius: 8px;
    outline: none;
    transition: all 0.2s ease;
    box-sizing: border-box;
    background-color: #ffffff;
    color: #333333;
  }

  .search-input:hover {
    border: 1px solid #e1e5e9;
  }

  .search-input:focus {
    border: 1px solid #e1e5e9;
  }

  .search-input::placeholder {
    color: #999999;
    opacity: 1;
  }
</style>
