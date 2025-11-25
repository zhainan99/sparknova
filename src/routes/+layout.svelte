<script lang="ts">
  // 全局布局组件
</script>

<div>
  <slot />
</div>

<style>
  * {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  /* 确保页面不出现滚动条 */
  :global(html),
  :global(body) {
    font-family:
      -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu,
      Cantarell, sans-serif;
    background: transparent;
    overflow: hidden;
    width: 100%;
    height: 100%;
  }

  /* 确保根容器不出现滚动条 */
  div {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }
</style>
