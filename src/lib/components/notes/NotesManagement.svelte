<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  import NotesList from "./NotesList.svelte";
  import NoteEditor from "./NoteEditor.svelte";
  import TagModal from "./tags/TagModal.svelte";
  import FolderModal from "./folders/FolderModal.svelte";
  import DeleteConfirmationModal from "./DeleteConfirmationModal.svelte";

  // states
  let notes = $state<any[]>([]);
  let allNotes = $state<any[]>([]);
  let folders = $state<any[]>([]);
  let tags = $state<any[]>([]);
  let selectedNote = $state<any>(null);
  let isLoading = $state(true);
  let isCreatingNew = $state(false);
  let isSidebarCollapsed = $state(false);
  let showTagModal = $state(false);
  let searchQuery = $state("");
  let searchTimeout = $state<number | null>(null);
  let isSearching = $state(false);
  let showFolderModal = $state(false);

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

  // prepare notes with tag colors
  function prepareNotesWithTagColors(
    loadedNotes: any[],
    tagsList: any[]
  ): any[] {
    return loadedNotes.map((note) => {
      // create a color mapping for each tag
      const tagColors: Record<string, string> = {};

      // for each tag in the note, find its color from the tags list
      note.tags.forEach((tagName: string) => {
        const tag = tagsList.find((t) => t.name === tagName);
        if (tag && tag.color) {
          tagColors[tagName] = tag.color;
        }
      });

      // add the tag colors to the note object
      return {
        ...note,
        tag_colors: tagColors,
      };
    });
  }

  // load data on component mount
  onMount(async () => {
    await Promise.all([loadFolders(), loadTags()]);
    await loadNotes(); // load notes after tags to ensure we have tag colors
    isLoading = false;
  });

  // load notes from backend
  async function loadNotes() {
    try {
      const loadedNotes: any = await invoke("get_notes");
      console.log("Loaded notes:", loadedNotes);

      // add tag colors to the notes
      const notesWithColors = prepareNotesWithTagColors(loadedNotes, tags);
      allNotes = notesWithColors; // store all notes
      notes = notesWithColors; // initially show all notes

      // if a note is selected, update it with the latest data
      if (selectedNote) {
        const updatedNote = notes.find((n) => n.id === selectedNote.id);
        if (updatedNote) {
          selectedNote = updatedNote;
        }
      }
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

  // handle search input with debounce
  function handleSearchInput(event: Event) {
    const target = event.target as HTMLInputElement;
    searchQuery = target.value;

    // clear previous timeout
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }

    // set new timeout (300ms debounce)
    searchTimeout = setTimeout(() => {
      performSearch();
    }, 300);
  }

  // perform the search
  async function performSearch() {
    if (!searchQuery.trim()) {
      // if search is empty, show all notes
      notes = allNotes;
      return;
    }

    isSearching = true;
    try {
      const searchResults: any = await invoke("search_notes", {
        query: searchQuery,
      });
      notes = prepareNotesWithTagColors(searchResults, tags);
    } catch (error) {
      console.error("Failed to search notes:", error);
      notes = allNotes.filter(
        (note) =>
          note.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
          note.content.toLowerCase().includes(searchQuery.toLowerCase())
      );
    } finally {
      isSearching = false;
    }
  }

  // clear search
  function clearSearch() {
    if (searchQuery) {
      searchQuery = "";
      notes = allNotes;
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

  function toggleFolderModal() {
    showFolderModal = !showFolderModal;
  }

  function handleFoldersUpdated() {
    loadFolders();
  }

  async function moveNoteToFolder(note: any, folderId: number | null) {
    try {
      await invoke("update_note", {
        id: note.id,
        title: note.title,
        content: note.content,
        folderId: folderId,
        tags: note.tags,
        isPinned: note.is_pinned,
        isArchived: note.is_archived,
        color: note.color,
        createRevision: false,
      });

      console.log(
        `Moved note ID ${note.id} to folder ID ${folderId || "null"}`
      );
      await loadNotes();

      // if the selected note was moved, update its data
      if (selectedNote && selectedNote.id === note.id) {
        const updatedNote = notes.find((n) => n.id === note.id);
        if (updatedNote) {
          selectedNote = updatedNote;
        }
      }
    } catch (error) {
      console.error("Failed to move note:", error);
    }
  }

  // toggle sidebar collapse state
  function toggleSidebar() {
    isSidebarCollapsed = !isSidebarCollapsed;
  }

  // handle keydown for sidebar toggle
  function handleSidebarKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      toggleSidebar();
    }
  }

  // handle tag updates
  function handleTagsUpdated() {
    loadTags().then(() => loadNotes());
  }

  // toggle tag modal visibility
  function toggleTagModal() {
    showTagModal = !showTagModal;
  }
</script>

<div class="notes-management">
  <div class="notes-sidebar" class:collapsed={isSidebarCollapsed}>
    <div class="sidebar-header">
      <h2>Notes</h2>
      <div class="sidebar-controls">
        <button
          class="folder-manager-btn"
          onclick={toggleFolderModal}
          title="Manage Folders"
        >
          📁
        </button>
        <button
          class="tag-manager-btn"
          onclick={toggleTagModal}
          title="Manage Tags"
        >
          🏷️
        </button>
        <button class="new-note-btn" onclick={createNewNote}>
          <span>+</span> New
        </button>
      </div>
    </div>

    <div class="search-container">
      <input
        type="text"
        class="search-input"
        placeholder="Search notes..."
        bind:value={searchQuery}
        oninput={handleSearchInput}
        aria-label="Search notes"
      />
      {#if searchQuery}
        <button
          class="clear-search-btn"
          onclick={clearSearch}
          aria-label="Clear search"
          title="Clear search"
        >
          ×
        </button>
      {/if}
    </div>

    {#if isLoading || isSearching}
      <div class="loading">
        {isSearching ? "Searching..." : "Loading notes..."}
      </div>
    {:else if notes.length === 0 && searchQuery}
      <div class="empty-list">
        <p>No notes match your search</p>
      </div>
    {:else}
      <NotesList
        notes={sortNotes(notes)}
        {folders}
        selectedNoteId={selectedNote?.id}
        onSelectNote={selectNote}
        onDeleteNote={confirmDeleteNote}
        onTogglePin={toggleNotePin}
        onMoveNote={moveNoteToFolder}
      />
    {/if}
  </div>

  <button
    type="button"
    class="sidebar-toggle"
    onclick={toggleSidebar}
    onkeydown={handleSidebarKeydown}
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

  {#if showTagModal}
    <TagModal onClose={toggleTagModal} onTagsUpdated={handleTagsUpdated} />
  {/if}

  {#if showFolderModal}
    <FolderModal
      onClose={toggleFolderModal}
      onFoldersUpdated={handleFoldersUpdated}
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
    gap: 8px;
  }

  .sidebar-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }

  .search-container {
    position: relative;
    padding: 8px 16px;
  }

  .search-input {
    width: 100%;
    padding: 6px 30px 6px 10px;
    border-radius: 4px;
    border: 1px solid rgba(128, 128, 128, 0.3);
    background: rgba(128, 128, 128, 0.05);
    font-size: 14px;
    outline: none;
    transition: all 0.2s;
  }

  .search-input:focus {
    border-color: rgba(128, 128, 128, 0.5);
    background: rgba(255, 255, 255, 0.1);
  }

  .clear-search-btn {
    position: absolute;
    right: 22px;
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    cursor: pointer;
    font-size: 18px;
    color: rgba(128, 128, 128, 0.6);
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    border-radius: 50%;
  }

  .clear-search-btn:hover {
    color: rgba(128, 128, 128, 0.9);
    background: rgba(128, 128, 128, 0.1);
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

  .tag-manager-btn {
    background: rgba(128, 128, 128, 0.1);
    border: none;
    border-radius: 4px;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    font-size: 16px;
    color: inherit;
    transition: background 0.2s;
  }

  .tag-manager-btn:hover {
    background: rgba(128, 128, 128, 0.2);
  }

  .folder-manager-btn {
    background: rgba(128, 128, 128, 0.1);
    border: none;
    border-radius: 4px;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    font-size: 16px;
    color: inherit;
    transition: background 0.2s;
  }

  .folder-manager-btn:hover {
    background: rgba(128, 128, 128, 0.2);
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

  .empty-list {
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
