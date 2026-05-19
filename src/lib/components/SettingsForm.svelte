<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";

  export let config: {
    scan_dirs: string[];
    theme_mode: string;
    show_main_on_start: boolean;
    show_notes_on_start: boolean;
  };

  const themeOptions = [
    { value: "light", label: "浅色" },
    { value: "dark", label: "深色" },
    { value: "system", label: "跟随系统" },
  ];

  let localConfig = { ...config };

  async function handleThemeChange(e: Event) {
    const select = e.target as HTMLSelectElement;
    localConfig.theme_mode = select.value;
    await saveConfig();
  }

  async function handleStartupChange(e: Event) {
    const checkbox = e.target as HTMLInputElement;
    localConfig.show_main_on_start = checkbox.checked;
    await saveConfig();
  }

  async function handleNotesStartupChange(e: Event) {
    const checkbox = e.target as HTMLInputElement;
    localConfig.show_notes_on_start = checkbox.checked;
    await saveConfig();
  }

  async function saveConfig() {
    try {
      await invoke("save_config", { config: localConfig });
    } catch (e) {
      console.error("save_config error:", e);
    }
  }

  async function addScanDir() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "选择扫描目录",
      });
      if (selected && typeof selected === "string") {
        if (!localConfig.scan_dirs.includes(selected)) {
          localConfig.scan_dirs = [...localConfig.scan_dirs, selected];
          await saveConfig();
        }
      }
    } catch (e) {
      console.error("open directory error:", e);
    }
  }

  async function removeScanDir(index: number) {
    localConfig.scan_dirs = localConfig.scan_dirs.filter((_, i) => i !== index);
    await saveConfig();
  }
</script>

<div class="settings-form">
  <section class="settings-section">
    <h3>主题</h3>
    <select value={localConfig.theme_mode} on:change={handleThemeChange}>
      {#each themeOptions as option}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
  </section>

  <section class="settings-section">
    <h3>启动</h3>
    <label class="checkbox-label">
      <input
        type="checkbox"
        checked={localConfig.show_main_on_start}
        on:change={handleStartupChange}
      />
      开机自启
    </label>
    <label class="checkbox-label" style="margin-top: 8px;">
      <input
        type="checkbox"
        checked={localConfig.show_notes_on_start}
        on:change={handleNotesStartupChange}
      />
      启动时显示笔记
    </label>
  </section>

  <section class="settings-section">
    <h3>扫描目录</h3>
    <div class="scan-dirs">
      {#each localConfig.scan_dirs as dir, index}
        <div class="dir-item">
          <span class="dir-path">{dir}</span>
          <button class="remove-btn" on:click={() => removeScanDir(index)}>
            删除
          </button>
        </div>
      {/each}
      <button class="add-btn" on:click={addScanDir}>添加目录</button>
    </div>
  </section>

  <section class="settings-section">
    <h3>关于</h3>
    <p class="about-text">SparkNova v0.1.0</p>
    <p class="about-text">轻量级插件启动器</p>
  </section>
</div>

<style>
  .settings-form {
    padding: 16px;
    height: 100%;
    overflow-y: auto;
    background: var(--bg-color, rgba(255, 255, 255, 0.98));
    color: var(--text-color, #333);
  }

  .settings-section {
    margin-bottom: 24px;
  }

  .settings-section h3 {
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 12px;
    color: var(--text-color, #333);
  }

  select {
    width: 100%;
    padding: 8px 12px;
    font-size: 14px;
    border: 1px solid #ddd;
    border-radius: 6px;
    background: white;
    cursor: pointer;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    cursor: pointer;
  }

  .checkbox-label input {
    width: 16px;
    height: 16px;
  }

  .scan-dirs {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .dir-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: #f5f5f5;
    border-radius: 6px;
  }

  .dir-path {
    font-size: 13px;
    word-break: break-all;
  }

  .remove-btn,
  .add-btn {
    padding: 6px 12px;
    font-size: 13px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .remove-btn {
    background: #ff4d4f;
    color: white;
  }

  .remove-btn:hover {
    background: #ff7875;
  }

  .add-btn {
    background: #1890ff;
    color: white;
  }

  .add-btn:hover {
    background: #40a9ff;
  }

  .about-text {
    font-size: 13px;
    color: #666;
    margin-bottom: 4px;
  }
</style>