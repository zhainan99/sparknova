<script lang="ts">
  let query = "";
</script>

<div class="app-container">
  <div class="search-input-container">
    <input
      bind:value={query}
      type="text"
      class="search-input"
      placeholder="输入命令或搜索..."
    />
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
    /* 宽度由后端根据屏幕分辨率动态计算 */
    width: 100%;
    /* 确保容器不会超出视口 */
    max-width: 90vw;
  }

  .search-input {
    /* 宽度占满容器，高度固定为80px */
    width: 100%;
    height: 80px;
    font-size: 18px;
    border: 0;
    outline: none;
    padding: 0 16px;
    background: rgba(255, 255, 255, 0.95);
    /* 确保输入框占满容器 */
    box-sizing: border-box;
  }
</style>
