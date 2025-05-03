<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  // props
  let { onClose, onFoldersUpdated } = $props();

  // state
  let folders = $state<any[]>([]);
  let isLoading = $state(true);
  let errorMessage = $state("");
  let view = $state<"menu" | "create" | "manage">("menu");

  // folder creation form state
  let newFolderName = $state("");
  let newFolderColor = $state("#808080");
  let parentFolderId = $state<number | null>(null);

  // handle backdrop click
  function handleBackdropClick(event: MouseEvent) {
    // only close if the click is directly on the backdrop (self)
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  // load folders on mount
  onMount(async () => {
    await loadFolders();
    isLoading = false;
  });

  // load all folders
  async function loadFolders() {
    try {
      folders = await invoke("get_folders");
      console.log("Loaded folders:", folders);
    } catch (error) {
      console.error("Failed to load folders:", error);
      errorMessage = `Error loading folders: ${error}`;
    }
  }

  // create a new folder
  async function createFolder() {
    if (!newFolderName.trim()) {
      errorMessage = "Folder name cannot be empty";
      return;
    }

    try {
      errorMessage = "";
      const folderId = await invoke("create_folder", {
        name: newFolderName.trim(),
        parentId: parentFolderId,
        color: newFolderColor,
      });
      console.log("Created folder with ID:", folderId);

      // clear input fields
      newFolderName = "";
      parentFolderId = null;

      // refresh folders list
      await loadFolders();

      // notify parent component that folders have been updated
      if (onFoldersUpdated) {
        onFoldersUpdated();
      }

      // return to menu view after successful creation
      view = "menu";
    } catch (error) {
      console.error("Failed to create folder:", error);
      errorMessage = `Error creating folder: ${error}`;
    }
  }

  // delete a folder
  async function deleteFolder(folderId: number) {
    try {
      await invoke("delete_folder", { id: folderId });
      console.log("Deleted folder with ID:", folderId);

      // refresh folders list
      await loadFolders();

      // notify parent component
      if (onFoldersUpdated) {
        onFoldersUpdated();
      }
    } catch (error) {
      console.error("Failed to delete folder:", error);
      errorMessage = `Error deleting folder: ${error}`;
    }
  }

  // switch view
  function showView(newView: "menu" | "create" | "manage") {
    view = newView;
    errorMessage = "";
  }
</script>

<div class="modal-backdrop" onclick={handleBackdropClick} role="presentation">
  <div
    class="modal-container"
    role="dialog"
    aria-labelledby="folder-modal-title"
  >
    {#if isLoading}
      <div class="modal-content">
        <h2 id="folder-modal-title">Folder Management</h2>
        <div class="loading">Loading folders...</div>
      </div>
    {:else if view === "menu"}
      <div class="modal-content">
        <h2 id="folder-modal-title">Folder Management</h2>
        <div class="menu-options">
          <button class="menu-option" onclick={() => showView("create")}>
            <span class="icon">+</span>
            <span class="text">Create a new Folder</span>
          </button>
          <button class="menu-option" onclick={() => showView("manage")}>
            <span class="icon">📁</span>
            <span class="text">Manage Existing Folders</span>
          </button>
        </div>
        <div class="modal-actions">
          <button class="cancel-button" onclick={onClose}>Close</button>
        </div>
      </div>
    {:else if view === "create"}
      <div class="modal-content">
        <h2 id="folder-modal-title">Create a new Folder</h2>

        <div class="form-group">
          <label for="folder-name">Folder Name:</label>
          <input
            type="text"
            id="folder-name"
            bind:value={newFolderName}
            placeholder="Enter folder name"
          />
        </div>

        <div class="form-group">
          <label for="parent-folder">Parent Folder (optional):</label>
          <select id="parent-folder" bind:value={parentFolderId}>
            <option value={null}>None (Root folder)</option>
            {#each folders as folder}
              <option value={folder.id}>{folder.name}</option>
            {/each}
          </select>
        </div>

        <div class="form-group">
          <label for="folder-color">Folder Color (optional):</label>
          <div class="color-picker">
            <input type="color" id="folder-color" bind:value={newFolderColor} />
            <span class="color-value">{newFolderColor}</span>
          </div>
        </div>

        {#if errorMessage}
          <div class="error-message">{errorMessage}</div>
        {/if}

        <div class="modal-actions">
          <button class="back-button" onclick={() => showView("menu")}
            >Back</button
          >
          <button
            class="create-button"
            onclick={createFolder}
            disabled={!newFolderName.trim()}
          >
            Create Folder
          </button>
        </div>
      </div>
    {:else if view === "manage"}
      <div class="modal-content">
        <h2 id="folder-modal-title">Manage Existing Folders</h2>

        {#if folders.length === 0}
          <div class="no-folders">No folders available</div>
        {:else}
          <div class="folders-list">
            {#each folders as folder}
              <div
                class="folder-item"
                style={folder.color
                  ? `border-left: 4px solid ${folder.color}`
                  : ""}
              >
                <span class="folder-name">
                  {folder.parent_id ? "↳ " : ""}
                  {folder.name}
                </span>
                <button
                  class="delete-folder"
                  title="Delete Folder"
                  onclick={() => deleteFolder(folder.id)}
                >
                  ×
                </button>
              </div>
            {/each}
          </div>
        {/if}

        {#if errorMessage}
          <div class="error-message">{errorMessage}</div>
        {/if}

        <div class="modal-actions">
          <button class="back-button" onclick={() => showView("menu")}
            >Back</button
          >
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(3px);
  }

  .modal-container {
    width: 90%;
    max-width: 450px;
    background-color: rgba(255, 255, 255, 0.95);
    border-radius: 8px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
    animation: slideIn 0.2s ease-out;
    overflow: hidden;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .modal-content {
    padding: 24px;
  }

  h2 {
    margin: 0 0 20px 0;
    font-size: 20px;
    font-weight: 600;
    color: #333;
  }

  .loading {
    padding: 20px 0;
    text-align: center;
    color: rgba(128, 128, 128, 0.8);
  }

  .menu-options {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 24px;
  }

  .menu-option {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    background-color: rgba(128, 128, 128, 0.08);
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
    text-align: left;
    color: inherit;
  }

  .menu-option:hover {
    background-color: rgba(128, 128, 128, 0.15);
  }

  .menu-option .icon {
    font-size: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
  }

  .menu-option .text {
    font-size: 16px;
    font-weight: 500;
  }

  .form-group {
    margin-bottom: 16px;
  }

  label {
    display: block;
    margin-bottom: 8px;
    font-weight: 500;
    color: inherit;
  }

  input[type="text"],
  select {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid rgba(128, 128, 128, 0.3);
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.8);
    color: inherit;
    font-size: 14px;
  }

  .color-picker {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  input[type="color"] {
    width: 40px;
    height: 40px;
    padding: 0;
    border: 1px solid rgba(128, 128, 128, 0.2);
    border-radius: 4px;
    cursor: pointer;
  }

  .color-value {
    font-family: monospace;
    font-size: 14px;
    color: inherit;
  }

  .error-message {
    margin: 16px 0;
    padding: 10px;
    background-color: rgba(255, 0, 0, 0.1);
    border-radius: 4px;
    color: #d32f2f;
    font-size: 14px;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 20px;
  }

  .cancel-button,
  .back-button,
  .create-button {
    padding: 8px 16px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    border: none;
  }

  .cancel-button,
  .back-button {
    background-color: rgba(0, 0, 0, 0.05);
    color: inherit;
  }

  .cancel-button:hover,
  .back-button:hover {
    background-color: rgba(0, 0, 0, 0.1);
  }

  .create-button {
    background-color: rgba(100, 120, 255, 0.2);
    color: inherit;
  }

  .create-button:hover:not(:disabled) {
    background-color: rgba(100, 120, 255, 0.3);
  }

  .create-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .folders-list {
    max-height: 300px;
    overflow-y: auto;
    margin-bottom: 16px;
  }

  .folder-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    background: rgba(128, 128, 128, 0.05);
    margin-bottom: 8px;
    border-radius: 6px;
  }

  .folder-name {
    font-size: 14px;
  }

  .delete-folder {
    background: transparent;
    border: none;
    color: inherit;
    font-size: 18px;
    cursor: pointer;
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    opacity: 0.6;
    transition: all 0.2s;
  }

  .delete-folder:hover {
    background: rgba(255, 0, 0, 0.1);
    color: #ff3333;
    opacity: 1;
  }

  .no-folders {
    padding: 20px 0;
    text-align: center;
    color: rgba(128, 128, 128, 0.8);
    font-style: italic;
  }

  /* dark mode styles */
  :global(html.dark) .modal-container {
    background-color: rgba(40, 40, 40, 0.95);
  }

  :global(html.dark) h2 {
    color: #eee;
  }

  :global(html.dark) input[type="text"],
  :global(html.dark) select {
    background: rgba(60, 60, 60, 0.8);
    color: #eee;
    border-color: rgba(128, 128, 128, 0.5);
  }

  :global(html.dark) .error-message {
    background-color: rgba(255, 0, 0, 0.15);
    color: #ff6b6b;
  }
</style>
