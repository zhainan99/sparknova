<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let notes: {
    id: number;
    content: string;
    kind: string;
    done: boolean;
    created_at: string;
  }[] = [];
  export let selectedId: number | null = null;

  const dispatch = createEventDispatcher();

  const handleClick = (note: (typeof notes)[0]) => {
    dispatch("select", { note });
  };

  const handleDelete = (e: MouseEvent, id: number) => {
    e.stopPropagation();
    dispatch("delete", { id });
  };

  const formatDate = (dateStr: string) => {
    try {
      const date = new Date(dateStr);
      return date.toLocaleDateString("zh-CN", {
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return dateStr;
    }
  };

  const getPreview = (content: string) => {
    if (!content) return "空白笔记";
    const text = content.replace(/[#*`_\[\]]/g, "").trim();
    return text.slice(0, 50) + (text.length > 50 ? "..." : "");
  };
</script>

<div class="note-list">
  {#if notes.length === 0}
    <div class="empty">暂无笔记</div>
  {:else}
    {#each notes as note (note.id)}
      <div
        role="button"
        class="note-item"
        class:selected={note.id === selectedId}
        on:click={() => handleClick(note)}
        on:keydown={(e) => e.key === "Enter" && handleClick(note)}
        tabindex="0"
      >
        <div class="note-header">
          <span class="note-kind">{note.kind}</span>
          <button
            type="button"
            class="delete-btn"
            on:click={(e) => handleDelete(e, note.id)}
            title="删除笔记"
          >
            ×
          </button>
        </div>
        <div class="note-preview">{getPreview(note.content)}</div>
        <div class="note-date">{formatDate(note.created_at)}</div>
      </div>
    {/each}
  {/if}
</div>

<style>
  .note-list {
    flex: 1;
    overflow-y: auto;
  }

  .empty {
    padding: 20px;
    text-align: center;
    color: #888;
    font-size: 14px;
  }

  .note-item {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 12px 16px;
    border: none;
    background: transparent;
    cursor: pointer;
    text-align: left;
    font-size: 14px;
    border-bottom: 1px solid #f0f0f0;
    transition: background-color 0.1s;
  }

  .note-item:hover {
    background: #f5f5f5;
  }

  .note-item.selected {
    background: #e6f0ff;
  }

  .note-header {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
  }

  .note-kind {
    font-size: 11px;
    color: #007aff;
    text-transform: uppercase;
    font-weight: 500;
  }

  .delete-btn {
    width: 20px;
    height: 20px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: #888;
    font-size: 16px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: opacity 0.1s, background-color 0.1s;
  }

  .note-item:hover .delete-btn {
    opacity: 1;
  }

  .delete-btn:hover {
    background: #ff3b30;
    color: white;
  }

  .note-preview {
    width: 100%;
    color: #333;
    line-height: 1.4;
    margin-bottom: 4px;
  }

  .note-date {
    font-size: 12px;
    color: #888;
  }
</style>