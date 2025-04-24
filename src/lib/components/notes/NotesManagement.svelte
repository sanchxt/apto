<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import NotesList from "./NotesList.svelte";
  import NoteEditor from "./NoteEditor.svelte";
  import DeleteConfirmationModal from "./DeleteConfirmationModal.svelte";

  // states
  let notes = $state<any[]>([]);
  let folders = $state<any[]>([]);
  let tags = $state<any[]>([]);
  let selectedNote = $state<any>(null);
  let isLoading = $state(true);
  let isCreatingNew = $state(false);
  let isSidebarCollapsed = $state(false);

  // modal state
  let showDeleteModal = $state(false);
  let noteToDelete = $state<{ id: number; title: string } | null>(null);

  // sort notes - pinned first, then updated_at (descending)
  function sortNotes(notesToSort: any[]): any[] {
    return [...notesToSort].sort((a, b) => {
      // sort by pin status
      if (a.is_pinned && !b.is_pinned) return -1;
      if (!a.is_pinned && b.is_pinned) return 1;

      // sort by date (newest first)
      const dateA = new Date(a.updated_at).getTime();
      const dateB = new Date(b.updated_at).getTime();
      return dateB - dateA;
    });
  }

  // load data on component mount
  onMount(async () => {
    await Promise.all([loadNotes(), loadFolders(), loadTags()]);
    isLoading = false;
  });

  // load notes from backend
  async function loadNotes() {
    try {
      notes = await invoke("get_notes");
      console.log("Loaded notes:", notes);
    } catch (error) {
      console.error("Failed to load notes:", error);
    }
  }

  // load folders from backend
  async function loadFolders() {
    try {
      folders = await invoke("get_folders");
      console.log("Loaded folders:", folders);
    } catch (error) {
      console.error("Failed to load folders:", error);
    }
  }

  // load tags from backend
  async function loadTags() {
    try {
      tags = await invoke("get_all_note_tags");
      console.log("Loaded tags:", tags);
    } catch (error) {
      console.error("Failed to load tags:", error);
    }
  }

  // handle note selection
  function selectNote(note: any) {
    selectedNote = note;
    isCreatingNew = false;
  }

  // create a new note
  function createNewNote() {
    selectedNote = null;
    isCreatingNew = true;
  }

  // save a new note
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

      // select the newly created note
      const newNote = notes.find((note) => note.id === newNoteId);
      if (newNote) {
        selectNote(newNote);
      }

      isCreatingNew = false;
    } catch (error) {
      console.error("Failed to create note:", error);
    }
  }

  // update an existing note
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

      // refresh the selected note
      const updatedNote = notes.find((note) => note.id === noteData.id);
      if (updatedNote) {
        selectedNote = updatedNote;
      }
    } catch (error) {
      console.error("Failed to update note:", error);
    }
  }

  // show the confirmation modal
  function confirmDeleteNote(note: any) {
    noteToDelete = {
      id: note.id,
      title: note.title || "Untitled",
    };
    showDeleteModal = true;
  }

  // actually delete the note after confirmation
  async function deleteNote() {
    if (!noteToDelete) return;

    try {
      await invoke("delete_note", { id: noteToDelete.id });
      console.log("Deleted note with ID:", noteToDelete.id);

      // reset selected note if the deleted one was selected
      if (selectedNote && selectedNote.id === noteToDelete.id) {
        selectedNote = null;
      }

      await loadNotes();

      // close the modal
      showDeleteModal = false;
      noteToDelete = null;
    } catch (error) {
      console.error("Failed to delete note:", error);
    }
  }

  // cancel note deletion
  function cancelDeleteNote() {
    showDeleteModal = false;
    noteToDelete = null;
  }

  // toggle pin status for a note
  async function toggleNotePin(note: any) {
    try {
      await invoke("toggle_note_pin", { id: note.id });
      console.log(
        `${note.is_pinned ? "Unpinned" : "Pinned"} note with ID:`,
        note.id
      );
      await loadNotes();

      // update selected note if it was the pinned/unpinned one
      if (selectedNote && selectedNote.id === note.id) {
        const updatedNote = notes.find((n) => n.id === note.id);
        if (updatedNote) {
          selectedNote = updatedNote;
        }
      }
    } catch (error) {
      console.error("Failed to toggle pin status:", error);
    }
  }

  // toggle sidebar collapse state
  function toggleSidebar() {
    isSidebarCollapsed = !isSidebarCollapsed;
  }
</script>

<div class="notes-management">
  <div class="notes-sidebar" class:collapsed={isSidebarCollapsed}>
    <div class="sidebar-header">
      <h2>Notes</h2>
      <div class="sidebar-controls">
        <button class="new-note-btn" onclick={createNewNote}>
          <span>+</span> New
        </button>
      </div>
    </div>

    {#if isLoading}
      <div class="loading">Loading notes...</div>
    {:else}
      <NotesList
        notes={sortNotes(notes)}
        selectedNoteId={selectedNote?.id}
        onSelectNote={selectNote}
        onDeleteNote={confirmDeleteNote}
        onTogglePin={toggleNotePin}
      />
    {/if}
  </div>

  <button
    type="button"
    class="sidebar-toggle"
    onclick={toggleSidebar}
    onkeydown={(e) => e.key === "Enter" && toggleSidebar()}
    title={isSidebarCollapsed ? "Expand Sidebar" : "Collapse Sidebar"}
  >
    <span class="toggle-icon">{isSidebarCollapsed ? "›" : "‹"}</span>
  </button>

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

  {#if showDeleteModal}
    <DeleteConfirmationModal
      noteTitle={noteToDelete?.title || "this note"}
      onConfirm={deleteNote}
      onCancel={cancelDeleteNote}
    />
  {/if}
</div>

<style>
  .notes-management {
    display: flex;
    height: 100%;
    position: relative;
  }

  .notes-sidebar {
    width: 250px;
    border-right: 1px solid rgba(128, 128, 128, 0.2);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transition:
      transform 0.3s ease,
      width 0.3s ease;
    flex-shrink: 0;
  }

  .notes-sidebar.collapsed {
    width: 0;
    transform: translateX(-100%);
  }

  .sidebar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 16px 8px 16px;
    border-bottom: 1px solid rgba(128, 128, 128, 0.2);
  }

  .sidebar-controls {
    display: flex;
    align-items: center;
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

  .sidebar-toggle {
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 20px;
    height: 40px;
    background: rgba(128, 128, 128, 0.1);
    border-radius: 0 4px 4px 0;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    z-index: 10;
    transition: all 0.3s ease;
    border: none;
  }

  .sidebar-toggle:hover {
    background: rgba(128, 128, 128, 0.2);
  }

  .notes-sidebar:not(.collapsed) + .sidebar-toggle {
    left: 250px;
  }

  .toggle-icon {
    font-size: 16px;
    line-height: 1;
    font-weight: bold;
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
