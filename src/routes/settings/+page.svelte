<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import SettingsForm from "$lib/components/SettingsForm.svelte";

  let config = {
    scan_dirs: [] as string[],
    theme_mode: "dark",
    show_main_on_start: false,
    show_notes_on_start: false,
  };
  let loading = true;

  async function loadConfig() {
    try {
      const result = await invoke<typeof config>("get_config");
      config = result;
    } catch (e) {
      console.error("get_config error:", e);
    } finally {
      loading = false;
    }
  }

  loadConfig();
</script>

<div class="settings-page">
  {#if loading}
    <div class="loading">加载中...</div>
  {:else}
    <SettingsForm {config} />
  {/if}
</div>

<style>
  .settings-page {
    width: 100%;
    height: 100%;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    font-size: 14px;
    color: #666;
  }
</style>