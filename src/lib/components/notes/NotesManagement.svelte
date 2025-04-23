<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import NotesList from "./NotesList.svelte";
  import NoteEditor from "./NoteEditor.svelte";

  // States
  let notes = $state<any[]>([]);
  let folders = $state<any[]>([]);
  let tags = $state<any[]>([]);
  let selectedNote = $state<any>(null);
  let isLoading = $state(true);
  let isCreatingNew = $state(false);

  // Load data on component mount
  onMount(async () => {
    await Promise.all([loadNotes(), loadFolders(), loadTags()]);
    isLoading = false;
  });

  // Load notes from backend
  async function loadNotes() {
    try {
      notes = await invoke("get_notes");
      console.log("Loaded notes:", notes);
    } catch (error) {
      console.error("Failed to load notes:", error);
    }
  }

  // Load folders from backend
  async function loadFolders() {
    try {
      folders = await invoke("get_folders");
      console.log("Loaded folders:", folders);
    } catch (error) {
      console.error("Failed to load folders:", error);
    }
  }

  // Load tags from backend
  async function loadTags() {
    try {
      tags = await invoke("get_all_note_tags");
      console.log("Loaded tags:", tags);
    } catch (error) {
      console.error("Failed to load tags:", error);
    }
  }

  // Handle note selection
  function selectNote(note: any) {
    selectedNote = note;
    isCreatingNew = false;
  }

  // Create a new note
  function createNewNote() {
    selectedNote = null;
    isCreatingNew = true;
  }

  // Save a new note
  async function saveNewNote(noteData: any) {
    try {
      const newNoteId = await invoke("create_note", {
        title: noteData.title,
        content: noteData.content,
        folderId: noteData.folderId,
        tags: noteData.tags,
        isPinned: noteData.isPinned || false,
        isArchived: noteData.isArchived || false,
        color: noteData.color,
      });

      console.log("Created note with ID:", newNoteId);
      await loadNotes();

      // Select the newly created note
      const newNote = notes.find((note) => note.id === newNoteId);
      if (newNote) {
        selectNote(newNote);
      }

      isCreatingNew = false;
    } catch (error) {
      console.error("Failed to create note:", error);
    }
  }

  // Update an existing note
  async function updateNote(noteData: any) {
    try {
      await invoke("update_note", {
        id: noteData.id,
        title: noteData.title,
        content: noteData.content,
        folderId: noteData.folderId,
        tags: noteData.tags,
        isPinned: noteData.isPinned,
        isArchived: noteData.isArchived,
        color: noteData.color,
        createRevision: true,
      });

      console.log("Updated note with ID:", noteData.id);
      await loadNotes();

      // Refresh the selected note
      const updatedNote = notes.find((note) => note.id === noteData.id);
      if (updatedNote) {
        selectedNote = updatedNote;
      }
    } catch (error) {
      console.error("Failed to update note:", error);
    }
  }

  // Delete a note
  async function deleteNote(noteId: number) {
    if (!confirm("Are you sure you want to delete this note?")) {
      return;
    }

    try {
      await invoke("delete_note", { id: noteId });
      console.log("Deleted note with ID:", noteId);

      // Reset selected note if the deleted one was selected
      if (selectedNote && selectedNote.id === noteId) {
        selectedNote = null;
      }

      await loadNotes();
    } catch (error) {
      console.error("Failed to delete note:", error);
    }
  }
</script>

<div class="notes-management">
  <div class="notes-sidebar">
    <div class="sidebar-header">
      <h2>Notes</h2>
      <button class="new-note-btn" onclick={createNewNote}>
        <span>+</span> New
      </button>
    </div>

    {#if isLoading}
      <div class="loading">Loading notes...</div>
    {:else}
      <NotesList
        {notes}
        selectedNoteId={selectedNote?.id}
        onSelectNote={selectNote}
        onDeleteNote={deleteNote}
      />
    {/if}
  </div>

  <div class="notes-content">
    {#if isCreatingNew}
      <NoteEditor
        isNew={true}
        {folders}
        {tags}
        onSave={saveNewNote}
        onCancel={() => (isCreatingNew = false)}
      />
    {:else if selectedNote}
      <NoteEditor
        isNew={false}
        note={selectedNote}
        {folders}
        {tags}
        onSave={updateNote}
        onCancel={() => (selectedNote = null)}
      />
    {:else}
      <div class="no-selection">
        <p>Select a note or create a new one</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .notes-management {
    display: flex;
    height: 100%;
  }

  .notes-sidebar {
    width: 250px;
    border-right: 1px solid rgba(128, 128, 128, 0.2);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sidebar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 16px 8px 16px;
    border-bottom: 1px solid rgba(128, 128, 128, 0.2);
  }

  .sidebar-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }

  .new-note-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(128, 128, 128, 0.1);
    border: none;
    border-radius: 4px;
    padding: 6px 12px;
    font-size: 14px;
    cursor: pointer;
    color: inherit;
    transition: background 0.2s;
  }

  .new-note-btn:hover {
    background: rgba(128, 128, 128, 0.2);
  }

  .new-note-btn span {
    font-size: 18px;
    margin-right: 4px;
    line-height: 1;
  }

  .loading {
    padding: 16px;
    text-align: center;
    color: rgba(128, 128, 128, 0.8);
  }

  .notes-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .no-selection {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: rgba(128, 128, 128, 0.6);
  }
</style>
