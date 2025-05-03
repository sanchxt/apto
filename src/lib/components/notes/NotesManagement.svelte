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

  // New states for folder navigation
  let selectedFolderId = $state<number | null>(null);
  let selectedFolderName = $state<string>("All Notes");
  let includeSubfolders = $state(true);
  let folderHierarchy = $state<any[]>([]);

  // Right sidebar state
  let isRightSidebarVisible = $state(false);

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
    await buildFolderHierarchy();
    await loadNotesBasedOnSelection();
    isLoading = false;
  });

  // load notes based on folder selection
  async function loadNotesBasedOnSelection() {
    isLoading = true;
    try {
      let loadedNotes: any;

      if (selectedFolderId === null) {
        // when "All Notes" is selected
        if (includeSubfolders) {
          // load all notes across all folders
          loadedNotes = await invoke("get_notes");
        } else {
          // load only root notes (not in any folder)
          loadedNotes = await invoke("get_notes_by_folder", {
            folderId: null,
          });
        }
      } else {
        // when a specific folder is selected
        loadedNotes = await invoke("get_notes_by_folder_recursive", {
          folderId: selectedFolderId,
          includeSubfolders: includeSubfolders,
        });
      }

      console.log(
        `Loaded notes for folder ${selectedFolderName}:`,
        loadedNotes
      );

      // add tag colors to the notes
      const notesWithColors = prepareNotesWithTagColors(loadedNotes, tags);
      notes = notesWithColors;

      // if a note is selected, update it
      if (selectedNote) {
        const updatedNote = notes.find((n) => n.id === selectedNote.id);
        if (updatedNote) {
          selectedNote = updatedNote;
        } else {
          // selected note is not in the current folder view
          selectedNote = null;
        }
      }
    } catch (error) {
      console.error("Failed to load notes for folder:", error);
      notes = [];
    } finally {
      isLoading = false;
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

  // build folder hierarchy for the sidebar
  async function buildFolderHierarchy() {
    try {
      // start with "All Notes" option
      const hierarchy = [
        {
          id: null,
          name: "All Notes",
          level: 0,
          hasChildren: false,
          parent_id: null,
        },
      ];

      // add root folders first
      const rootFolders = folders.filter((f) => f.parent_id === null);

      for (const folder of rootFolders) {
        hierarchy.push({
          ...folder,
          level: 1,
          hasChildren: folders.some((f) => f.parent_id === folder.id),
        });

        // add subfolders recursively
        addSubfoldersToHierarchy(folder.id, 2, hierarchy);
      }

      folderHierarchy = hierarchy;
      console.log("Built folder hierarchy:", folderHierarchy);
    } catch (error) {
      console.error("Failed to build folder hierarchy:", error);
    }
  }

  // ghelper function to recursively add subfolders to hierarchy
  function addSubfoldersToHierarchy(
    parentId: number,
    level: number,
    hierarchy: any[]
  ) {
    const subfolders = folders.filter((f) => f.parent_id === parentId);

    for (const folder of subfolders) {
      hierarchy.push({
        ...folder,
        level,
        hasChildren: folders.some((f) => f.parent_id === folder.id),
      });

      // recursively add children
      addSubfoldersToHierarchy(folder.id, level + 1, hierarchy);
    }
  }

  // select a folder and load its notes
  function selectFolder(folderId: number | null, folderName: string) {
    selectedFolderId = folderId;
    selectedFolderName = folderName;

    // reset search when changing folders
    if (searchQuery) {
      searchQuery = "";
    }

    loadNotesBasedOnSelection();
  }

  // toggle inclusion of subfolders
  function toggleIncludeSubfolders() {
    includeSubfolders = !includeSubfolders;
    loadNotesBasedOnSelection();
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
      // if search is empty, reload based on folder selection
      loadNotesBasedOnSelection();
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
      loadNotesBasedOnSelection();
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
        folderId: noteData.folderId || selectedFolderId,
        tags: noteData.tags,
        isPinned: noteData.isPinned || false,
        isArchived: noteData.isArchived || false,
        color: noteData.color,
      });

      console.log("Created note with ID:", newNoteId);
      await loadNotesBasedOnSelection();

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
      await loadNotesBasedOnSelection();

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

      await loadNotesBasedOnSelection();

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
      await loadNotesBasedOnSelection();

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
    loadFolders().then(() => {
      buildFolderHierarchy();
      loadNotesBasedOnSelection();
    });
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
      await loadNotesBasedOnSelection();

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

  // toggle left sidebar
  function toggleSidebar() {
    isSidebarCollapsed = !isSidebarCollapsed;
  }

  // toggle right sidebar (folders)
  function toggleRightSidebar() {
    isRightSidebarVisible = !isRightSidebarVisible;
  }

  // handle keydown for sidebar toggle
  function handleSidebarKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      toggleSidebar();
    }
  }

  // handle keydown for right sidebar toggle
  function handleRightSidebarKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      toggleRightSidebar();
    }
  }

  // handle tag updates
  function handleTagsUpdated() {
    loadTags().then(() => loadNotesBasedOnSelection());
  }

  // toggle tag modal visibility
  function toggleTagModal() {
    showTagModal = !showTagModal;
  }
</script>

<div class="notes-management">
  <!-- left sidebar - notes list -->
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

    <!-- display current folder -->
    <div class="current-folder">
      <h3>{selectedFolderName}</h3>
      <div class="folder-actions">
        {#if searchQuery}
          <span class="search-indicator">🔍 Search results</span>
        {/if}
        <button
          class="toggle-folders-btn"
          onclick={toggleRightSidebar}
          title={isRightSidebarVisible ? "Hide Folders" : "Show Folders"}
        >
          {isRightSidebarVisible ? "Hide Folders" : "Show Folders"}
        </button>
      </div>
    </div>

    {#if isLoading || isSearching}
      <div class="loading">
        {isSearching ? "Searching..." : "Loading notes..."}
      </div>
    {:else if notes.length === 0 && searchQuery}
      <div class="empty-list">
        <p>No notes match your search</p>
      </div>
    {:else if notes.length === 0}
      <div class="empty-list">
        <p>No notes in this folder</p>
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

  <!-- left sidebar toggle -->
  <button
    type="button"
    class="sidebar-toggle left-toggle"
    onclick={toggleSidebar}
    onkeydown={handleSidebarKeydown}
    title={isSidebarCollapsed ? "Expand Sidebar" : "Collapse Sidebar"}
  >
    <span class="toggle-icon">{isSidebarCollapsed ? "›" : "‹"}</span>
  </button>

  <!-- main content - note Editor -->
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

  <!-- right sidebar - folders -->
  <div class="folders-sidebar" class:visible={isRightSidebarVisible}>
    <div class="folders-header">
      <h3>Folders</h3>
      <button
        class="close-folders-btn"
        onclick={toggleRightSidebar}
        title="Hide Folders"
      >
        <!-- Replace "×" with more subtle SVG icon -->
        <svg
          width="16"
          height="16"
          viewBox="0 0 16 16"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M4 12L12 4M4 4L12 12"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
    </div>

    <label class="subfolder-toggle">
      <input
        type="checkbox"
        checked={includeSubfolders}
        onchange={toggleIncludeSubfolders}
      />
      <span>Include subfolders</span>
    </label>

    <div class="folders-list">
      {#each folderHierarchy as folder}
        <button
          class="folder-item"
          class:active={selectedFolderId === folder.id}
          style={`padding-left: ${16 + folder.level * 12}px;`}
          onclick={() => selectFolder(folder.id, folder.name)}
        >
          {#if folder.color}
            <span
              class="folder-color-indicator"
              style={`background-color: ${folder.color};`}
            ></span>
          {/if}
          <span class="folder-icon">
            {#if folder.id === null}
              <svg
                width="16"
                height="16"
                viewBox="0 0 16 16"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <rect
                  x="2"
                  y="3"
                  width="12"
                  height="11"
                  rx="1"
                  stroke="currentColor"
                  stroke-width="1.5"
                />
                <path d="M2 6H14" stroke="currentColor" stroke-width="1.5" />
              </svg>
            {:else if folder.hasChildren}
              <svg
                width="16"
                height="16"
                viewBox="0 0 16 16"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M2 4C2 3.44772 2.44772 3 3 3H7L8.5 5H13C13.5523 5 14 5.44772 14 6V12C14 12.5523 13.5523 13 13 13H3C2.44772 13 2 12.5523 2 12V4Z"
                  stroke="currentColor"
                  stroke-width="1.5"
                />
              </svg>
            {:else}
              <svg
                width="16"
                height="16"
                viewBox="0 0 16 16"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M2 4C2 3.44772 2.44772 3 3 3H7L8.5 5H13C13.5523 5 14 5.44772 14 6V12C14 12.5523 13.5523 13 13 13H3C2.44772 13 2 12.5523 2 12V4Z"
                  stroke="currentColor"
                  stroke-width="1.5"
                />
                <path
                  d="M6 9H10"
                  stroke="currentColor"
                  stroke-width="1.5"
                  stroke-linecap="round"
                />
              </svg>
            {/if}
          </span>
          <span class="folder-name">{folder.name}</span>
          {#if selectedFolderId === folder.id}
            <span class="active-indicator">
              <svg
                width="12"
                height="12"
                viewBox="0 0 12 12"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M2.5 6L4.5 8L9.5 3"
                  stroke="currentColor"
                  stroke-width="1.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            </span>
          {/if}
        </button>
      {/each}
    </div>
  </div>

  <!-- right sidebar toggle (mobile) -->
  <button
    type="button"
    class="sidebar-toggle right-toggle"
    onclick={toggleRightSidebar}
    onkeydown={handleRightSidebarKeydown}
    title={isRightSidebarVisible ? "Hide Folders" : "Show Folders"}
    class:visible={isRightSidebarVisible}
    aria-label={isRightSidebarVisible ? "Hide Folders" : "Show Folders"}
  >
    <svg
      width="20"
      height="20"
      viewBox="0 0 20 20"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M7 5L12 10L7 15"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </button>

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
    overflow: visible;
    min-height: 0;
  }

  /* left sidebar (notes list) */
  .notes-sidebar {
    width: 280px;
    border-right: 1px solid rgba(128, 128, 128, 0.2);
    display: flex;
    flex-direction: column;
    transition:
      transform 0.3s ease,
      width 0.3s ease;
    flex-shrink: 0;
    height: 100%;
    overflow: hidden;
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

  /* current folder indicator */
  .current-folder {
    padding: 8px 16px;
    border-bottom: 1px solid rgba(128, 128, 128, 0.2);
  }

  .current-folder h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 500;
  }

  .folder-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 8px;
  }

  .toggle-folders-btn {
    font-size: 12px;
    padding: 4px 8px;
    border: none;
    background: rgba(128, 128, 128, 0.1);
    border-radius: 4px;
    cursor: pointer;
    color: inherit;
  }

  .toggle-folders-btn:hover {
    background: rgba(128, 128, 128, 0.2);
  }

  .search-indicator {
    font-size: 12px;
    color: rgba(128, 128, 128, 0.8);
  }

  .search-container {
    position: relative;
    padding: 8px 16px;
    border-bottom: 1px solid rgba(128, 128, 128, 0.2);
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

  .tag-manager-btn,
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

  .tag-manager-btn:hover,
  .folder-manager-btn:hover {
    background: rgba(128, 128, 128, 0.2);
  }

  /* left sidebar toggle */
  .sidebar-toggle {
    position: absolute;
    width: 24px;
    height: 48px;
    background: rgba(255, 255, 255, 0.95);
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 0 6px 6px 0;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    z-index: 10;
    transition: all 0.15s ease;
    color: #6b7280;
  }

  .sidebar-toggle.left-toggle {
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    border-radius: 0 4px 4px 0;
  }

  .sidebar-toggle.right-toggle {
    right: 0;
    top: 50%;
    transform: translateY(-50%) rotate(180deg);
    border-radius: 6px 0 0 6px;
    border-right: none;
  }

  .sidebar-toggle:hover {
    background: #f9fafb;
    color: #374151;
  }

  .notes-sidebar:not(.collapsed) + .sidebar-toggle.left-toggle {
    left: 280px;
  }

  .folders-sidebar.visible + .sidebar-toggle.right-toggle {
    right: 260px;
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

  /* main content */
  .notes-content {
    flex: 1;
    overflow: auto;
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .no-selection {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: rgba(128, 128, 128, 0.6);
  }

  /* right sidebar (folders) */
  .folders-sidebar {
    width: 260px;
    border-left: 1px solid rgba(128, 128, 128, 0.2);
    display: flex;
    flex-direction: column;
    transition:
      transform 0.3s ease,
      width 0.3s ease,
      opacity 0.3s ease;
    flex-shrink: 0;
    height: 100%;
    overflow: hidden;
    transform: translateX(100%);
    position: absolute;
    right: 0;
    top: 0;
    background: transparent;
    z-index: 5;
    opacity: 0;
  }

  .folders-sidebar.visible {
    transform: translateX(0);
    opacity: 1;
  }

  .folders-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid rgba(128, 128, 128, 0.2);
  }

  .folders-header h3 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: #1a1a1a;
    letter-spacing: -0.01em;
  }

  .close-folders-btn {
    background: none;
    border: none;
    cursor: pointer;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    color: #6b7280;
    transition: all 0.15s ease;
  }

  .close-folders-btn:hover {
    background: rgba(128, 128, 128, 0.1);
    color: #374151;
  }

  .subfolder-toggle {
    display: flex;
    align-items: center;
    font-size: 13px;
    padding: 12px 20px;
    color: #4b5563;
    cursor: pointer;
    border-bottom: 1px solid rgba(128, 128, 128, 0.2);
    transition: background-color 0.15s ease;
  }

  .subfolder-toggle:hover {
    background-color: rgba(128, 128, 128, 0.05);
  }

  .subfolder-toggle input {
    margin-right: 8px;
    width: 14px;
    height: 14px;
    cursor: pointer;
  }

  .folders-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }

  .folder-item {
    display: flex;
    align-items: center;
    padding: 6px 20px;
    cursor: pointer;
    font-size: 13px;
    color: #374151;
    transition: all 0.15s ease;
    position: relative;
    background: none;
    border: none;
    width: 100%;
    text-align: left;
  }

  .folder-item:hover {
    background: rgba(128, 128, 128, 0.05);
    color: #111827;
  }

  .folder-item.active {
    background: rgba(128, 128, 128, 0.1);
    color: #111827;
    font-weight: 500;
  }

  .folder-color-indicator {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    margin-right: 8px;
    flex-shrink: 0;
  }

  .folder-icon {
    margin-right: 8px;
    color: #6b7280;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .folder-item.active .folder-icon {
    color: #374151;
  }

  .folder-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    line-height: 20px;
  }

  .active-indicator {
    margin-left: auto;
    color: #3b82f6;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  /* scrollbar styling for folders list */
  .folders-list::-webkit-scrollbar {
    width: 6px;
  }

  .folders-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .folders-list::-webkit-scrollbar-thumb {
    background: rgba(0, 0, 0, 0.1);
    border-radius: 3px;
  }

  .folders-list::-webkit-scrollbar-thumb:hover {
    background: rgba(0, 0, 0, 0.15);
  }

  @media (min-width: 1024px) {
    .folders-sidebar {
      position: relative;
      transform: translateX(0);
      width: 0;
      opacity: 1;
    }

    .folders-sidebar.visible {
      width: 250px;
      transform: translateX(0);
    }

    .sidebar-toggle.right-toggle {
      display: none;
    }
  }

  @media (max-width: 1023px) {
    .folders-sidebar.visible {
      background: rgba(255, 255, 255, 0.8);
      backdrop-filter: blur(10px);
    }

    :global(html.dark) .folders-sidebar.visible {
      background: rgba(26, 26, 26, 0.8);
      backdrop-filter: blur(10px);
    }
  }

  /* dark mode */
  :global(html.dark) .sidebar-toggle {
    background: rgba(26, 26, 26, 0.95);
    border-color: rgba(255, 255, 255, 0.08);
    color: #9ca3af;
  }

  :global(html.dark) .sidebar-toggle:hover {
    background: #333;
    color: #d1d5db;
  }

  :global(html.dark) .folders-sidebar {
    background: transparent;
    border-left-color: rgba(255, 255, 255, 0.1);
  }

  :global(html.dark) .folders-header {
    border-bottom-color: rgba(255, 255, 255, 0.08);
  }

  :global(html.dark) .folders-header h3 {
    color: #f9fafb;
  }

  :global(html.dark) .close-folders-btn {
    color: #9ca3af;
  }

  :global(html.dark) .close-folders-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #d1d5db;
  }

  :global(html.dark) .subfolder-toggle {
    color: #d1d5db;
    border-bottom-color: rgba(255, 255, 255, 0.08);
  }

  :global(html.dark) .subfolder-toggle:hover {
    background-color: rgba(255, 255, 255, 0.02);
  }

  :global(html.dark) .folder-item {
    color: #d1d5db;
  }

  :global(html.dark) .folder-item:hover {
    background: rgba(255, 255, 255, 0.05);
    color: #f3f4f6;
  }

  :global(html.dark) .folder-item.active {
    background: rgba(255, 255, 255, 0.1);
    color: #f3f4f6;
  }

  :global(html.dark) .folder-icon {
    color: #9ca3af;
  }

  :global(html.dark) .folder-item.active .folder-icon {
    color: #d1d5db;
  }

  :global(html.dark) .active-indicator {
    color: #60a5fa;
  }
</style>
