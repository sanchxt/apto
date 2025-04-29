<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  // props
  let { onClose, onTagsUpdated } = $props();

  // state
  let tags = $state<any[]>([]);
  let isLoading = $state(true);
  let errorMessage = $state("");
  let view = $state<"menu" | "create" | "manage">("menu");

  // tag creation form state
  let newTagName = $state("");
  let newTagColor = $state("#808080");

  // handle backdrop click
  function handleBackdropClick(event: MouseEvent) {
    // only close if the click is directly on the backdrop (self)
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  // load tags on mount
  onMount(async () => {
    await loadTags();
    isLoading = false;
  });

  // load all tags
  async function loadTags() {
    try {
      tags = await invoke("get_all_note_tags");
      console.log("Loaded tags:", tags);
    } catch (error) {
      console.error("Failed to load tags:", error);
      errorMessage = `Error loading tags: ${error}`;
    }
  }

  // create a new tag
  async function createTag() {
    if (!newTagName.trim()) {
      errorMessage = "Tag name cannot be empty";
      return;
    }

    try {
      errorMessage = "";
      const tagId = await invoke("create_note_tag", {
        name: newTagName.trim(),
        color: newTagColor,
      });
      console.log("Created tag with ID:", tagId);

      // clear input fields
      newTagName = "";

      // refresh tags list
      await loadTags();

      // notify parent component that tags have been updated
      if (onTagsUpdated) {
        onTagsUpdated();
      }

      // return to menu view after successful creation
      view = "menu";
    } catch (error) {
      console.error("Failed to create tag:", error);
      errorMessage = `Error creating tag: ${error}`;
    }
  }

  // delete a tag
  async function deleteTag(tagId: number) {
    try {
      await invoke("delete_note_tag", { id: tagId });
      console.log("Deleted tag with ID:", tagId);

      // refresh tags list
      await loadTags();

      // notify parent component
      if (onTagsUpdated) {
        onTagsUpdated();
      }
    } catch (error) {
      console.error("Failed to delete tag:", error);
      errorMessage = `Error deleting tag: ${error}`;
    }
  }

  // switch view
  function showView(newView: "menu" | "create" | "manage") {
    view = newView;
    errorMessage = ""; // Clear any previous error messages
  }
</script>

<div class="modal-backdrop" onclick={handleBackdropClick} role="presentation">
  <div class="modal-container" role="dialog" aria-labelledby="tag-modal-title">
    {#if isLoading}
      <div class="modal-content">
        <h2 id="tag-modal-title">Tag Management</h2>
        <div class="loading">Loading tags...</div>
      </div>
    {:else if view === "menu"}
      <div class="modal-content">
        <h2 id="tag-modal-title">Tag Management</h2>
        <div class="menu-options">
          <button class="menu-option" onclick={() => showView("create")}>
            <span class="icon">+</span>
            <span class="text">Create a new Tag</span>
          </button>
          <button class="menu-option" onclick={() => showView("manage")}>
            <span class="icon">🏷️</span>
            <span class="text">Manage Existing Tags</span>
          </button>
        </div>
        <div class="modal-actions">
          <button class="cancel-button" onclick={onClose}>Close</button>
        </div>
      </div>
    {:else if view === "create"}
      <div class="modal-content">
        <h2 id="tag-modal-title">Create a new Tag</h2>

        <div class="form-group">
          <label for="tag-name">Tag Name:</label>
          <input
            type="text"
            id="tag-name"
            bind:value={newTagName}
            placeholder="Enter tag name"
          />
        </div>

        <div class="form-group">
          <label for="tag-color">Tag Color:</label>
          <div class="color-picker">
            <input type="color" id="tag-color" bind:value={newTagColor} />
            <span class="color-value">{newTagColor}</span>
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
            onclick={createTag}
            disabled={!newTagName.trim()}
          >
            Create Tag
          </button>
        </div>
      </div>
    {:else if view === "manage"}
      <div class="modal-content">
        <h2 id="tag-modal-title">Manage Existing Tags</h2>

        {#if tags.length === 0}
          <div class="no-tags">No tags available</div>
        {:else}
          <div class="tags-list">
            {#each tags as tag}
              <div
                class="tag-item"
                style={tag.color ? `border-left: 4px solid ${tag.color}` : ""}
              >
                <span class="tag-name">{tag.name}</span>
                <button
                  class="delete-tag"
                  title="Delete Tag"
                  onclick={() => deleteTag(tag.id)}
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

  input[type="text"] {
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

  .tags-list {
    max-height: 300px;
    overflow-y: auto;
    margin-bottom: 16px;
  }

  .tag-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    background: rgba(128, 128, 128, 0.05);
    margin-bottom: 8px;
    border-radius: 6px;
  }

  .tag-name {
    font-size: 14px;
  }

  .delete-tag {
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

  .delete-tag:hover {
    background: rgba(255, 0, 0, 0.1);
    color: #ff3333;
    opacity: 1;
  }

  .no-tags {
    padding: 20px 0;
    text-align: center;
    color: rgba(128, 128, 128, 0.8);
    font-style: italic;
  }

  /* Dark mode styles */
  :global(html.dark) .modal-container {
    background-color: rgba(40, 40, 40, 0.95);
  }

  :global(html.dark) h2 {
    color: #eee;
  }

  :global(html.dark) input[type="text"] {
    background: rgba(60, 60, 60, 0.8);
    color: #eee;
    border-color: rgba(128, 128, 128, 0.5);
  }

  :global(html.dark) .error-message {
    background-color: rgba(255, 0, 0, 0.15);
    color: #ff6b6b;
  }
</style>
