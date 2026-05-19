<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import NoteList from "$lib/components/NoteList.svelte";
  import NoteEditor from "$lib/components/NoteEditor.svelte";

  interface Note {
    id: number;
    content: string;
    kind: string;
    done: boolean;
    created_at: string;
  }

  let notes: Note[] = [];
  let selectedNote: Note | null = null;
  let searchQuery = "";

  const loadNotes = async () => {
    try {
      notes = await invoke<Note[]>("list_notes");
    } catch (e) {
      console.error("list_notes error:", e);
    }
  };

  const handleCreateNote = async () => {
    try {
      const newNote = await invoke<Note>("create_note", {
        content: "",
        kind: "Memo",
      });
      notes = [newNote, ...notes];
      selectedNote = newNote;
    } catch (e) {
      console.error("create_note error:", e);
    }
  };

  const handleDeleteNote = async (e: CustomEvent<{ id: number }>) => {
    try {
      await invoke("delete_note", { id: e.detail.id });
      notes = notes.filter((n) => n.id !== e.detail.id);
      if (selectedNote?.id === e.detail.id) {
        selectedNote = null;
      }
    } catch (e) {
      console.error("delete_note error:", e);
    }
  };

  const handleSelectNote = (e: CustomEvent<{ note: Note }>) => {
    selectedNote = e.detail.note;
  };

  const handleUpdateNote = (e: CustomEvent<{ note: Note }>) => {
    const updatedNote = e.detail.note;
    notes = notes.map((n) => (n.id === updatedNote.id ? updatedNote : n));
    if (selectedNote?.id === updatedNote.id) {
      selectedNote = updatedNote;
    }
  };

  $: filteredNotes = searchQuery
    ? notes.filter((n) =>
        n.content.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : notes;

  onMount(() => {
    loadNotes();
  });
</script>

<div class="notes-page">
  <div class="notes-sidebar">
    <div class="sidebar-header">
      <input
        type="text"
        class="search-input"
        placeholder="搜索笔记..."
        bind:value={searchQuery}
      />
      <button class="new-btn" on:click={handleCreateNote}>+</button>
    </div>
    <NoteList
      notes={filteredNotes}
      selectedId={selectedNote?.id ?? null}
      on:select={handleSelectNote}
      on:delete={handleDeleteNote}
    />
  </div>
  <div class="notes-main">
    {#if selectedNote}
      <NoteEditor note={selectedNote} on:update={handleUpdateNote} />
    {:else}
      <div class="empty-state">
        <p>选择或创建一个笔记</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .notes-page {
    width: 100%;
    height: 100%;
    display: flex;
    background: rgba(255, 255, 255, 0.98);
  }

  .notes-sidebar {
    width: 280px;
    min-width: 280px;
    height: 100%;
    display: flex;
    flex-direction: column;
    border-right: 1px solid #e5e5e5;
  }

  .sidebar-header {
    display: flex;
    gap: 8px;
    padding: 12px;
    border-bottom: 1px solid #f0f0f0;
  }

  .search-input {
    flex: 1;
    height: 36px;
    padding: 0 12px;
    border: 1px solid #e5e5e5;
    border-radius: 6px;
    font-size: 14px;
    outline: none;
  }

  .search-input:focus {
    border-color: #007aff;
  }

  .new-btn {
    width: 36px;
    height: 36px;
    border: none;
    border-radius: 6px;
    background: #007aff;
    color: white;
    font-size: 20px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .new-btn:hover {
    background: #0066dd;
  }

  .notes-main {
    flex: 1;
    height: 100%;
    overflow: hidden;
  }

  .empty-state {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #888;
    font-size: 16px;
  }
</style>