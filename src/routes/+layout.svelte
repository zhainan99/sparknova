<script lang="ts">
  // 全局布局组件
</script>

<div>
  <slot />
</div>

<style>
  :global(html),
  :global(body) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
    font-family:
      -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu,
      Cantarell, sans-serif;
    background: transparent;
    overflow: hidden;
    width: 100%;
    height: 100%;
  }
</style>