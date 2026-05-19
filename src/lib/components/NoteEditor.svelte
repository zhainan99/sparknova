<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  export let note: {
    id: number;
    content: string;
    kind: string;
    done: boolean;
    created_at: string;
  };

  const dispatch = createEventDispatcher();

  let content = note.content;
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let textareaEl: HTMLTextAreaElement;

  const saveNote = async () => {
    if (content === note.content) return;
    try {
      const updatedNote = await invoke<typeof note>("create_note", {
        content,
        kind: note.kind,
      });
      updatedNote.id = note.id;
      dispatch("update", { note: updatedNote });
    } catch (e) {
      console.error("save_note error:", e);
    }
  };

  const scheduleAutoSave = () => {
    if (saveTimer) {
      clearTimeout(saveTimer);
    }
    saveTimer = setTimeout(() => {
      saveNote();
    }, 1000);
  };

  const handleInput = () => {
    scheduleAutoSave();
  };

  const handleKeydown = (e: KeyboardEvent) => {
    if (e.key === "Tab") {
      e.preventDefault();
      const start = textareaEl.selectionStart;
      const end = textareaEl.selectionEnd;
      content = content.substring(0, start) + "  " + content.substring(end);
      // Move cursor
      setTimeout(() => {
        textareaEl.selectionStart = textareaEl.selectionEnd = start + 2;
      }, 0);
    }
  };

  const handleInsertMarkdown = (syntax: string) => {
    const start = textareaEl.selectionStart;
    const end = textareaEl.selectionEnd;
    const selected = content.substring(start, end);
    const before = content.substring(0, start);
    const after = content.substring(end);

    if (syntax === "# ") {
      // Find line start
      const lineStart = before.lastIndexOf("\n") + 1;
      content =
        content.substring(0, lineStart) +
        "# " +
        content.substring(lineStart);
    } else if (syntax === "**") {
      content = before + "**" + selected + "**" + after;
    } else if (syntax === "*") {
      content = before + "*" + selected + "*" + after;
    } else if (syntax === "```") {
      content = before + "```\n" + selected + "\n```" + after;
    } else if (syntax === "- ") {
      const lineStart = before.lastIndexOf("\n") + 1;
      content =
        content.substring(0, lineStart) + "- " + content.substring(lineStart);
    } else {
      content = before + syntax + selected + syntax + after;
    }

    scheduleAutoSave();
    setTimeout(() => {
      textareaEl.focus();
    }, 0);
  };

  onMount(() => {
    textareaEl.focus();
  });
</script>

<div class="editor-container">
  <div class="toolbar">
    <button type="button" on:click={() => handleInsertMarkdown("# ")} title="标题">
      H
    </button>
    <button type="button" on:click={() => handleInsertMarkdown("**")} title="粗体">
      B
    </button>
    <button type="button" on:click={() => handleInsertMarkdown("*")} title="斜体">
      I
    </button>
    <button type="button" on:click={() => handleInsertMarkdown("```")} title="代码">
      {"</>"}
    </button>
    <button type="button" on:click={() => handleInsertMarkdown("- ")} title="列表">
      -
    </button>
  </div>
  <textarea
    bind:this={textareaEl}
    bind:value={content}
    on:input={handleInput}
    on:keydown={handleKeydown}
    class="editor-textarea"
    placeholder="开始编写笔记..."
    spellcheck="false"
  ></textarea>
</div>

<style>
  .editor-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .toolbar {
    display: flex;
    gap: 4px;
    padding: 8px 12px;
    border-bottom: 1px solid #e5e5e5;
    background: #fafafa;
  }

  .toolbar button {
    width: 32px;
    height: 32px;
    border: 1px solid #e5e5e5;
    border-radius: 4px;
    background: white;
    cursor: pointer;
    font-size: 14px;
    font-weight: 600;
    color: #333;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .toolbar button:hover {
    background: #f0f0f0;
    border-color: #007aff;
  }

  .editor-textarea {
    flex: 1;
    width: 100%;
    padding: 16px;
    border: none;
    outline: none;
    resize: none;
    font-size: 15px;
    font-family: inherit;
    line-height: 1.6;
    box-sizing: border-box;
  }

  .editor-textarea::placeholder {
    color: #aaa;
  }
</style>