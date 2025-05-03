<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  // props
  let { onTagsUpdated } = $props();

  // state
  let tags = $state<any[]>([]);
  let newTagName = $state("");
  let newTagColor = $state("#808080");
  let isLoading = $state(true);
  let errorMessage = $state("");

  // load tags on mount
  onMount(async () => {
    await loadTags();
    isLoading = false;
  });

  // load all tags from the backend
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
    } catch (error) {
      console.error("Failed to create tag:", error);
      errorMessage = `Error creating tag: ${error}`;
    }
  }
</script>

<div class="tag-management">
  <h3>Manage Tags</h3>

  {#if isLoading}
    <div class="loading">Loading tags...</div>
  {:else}
    <div class="tag-form">
      <div class="input-group">
        <input
          type="text"
          placeholder="New tag name"
          bind:value={newTagName}
          class="tag-name-input"
        />
        <input
          type="color"
          bind:value={newTagColor}
          class="tag-color-input"
          title="Choose tag color"
        />
        <button class="create-tag-btn" onclick={createTag}> Create Tag </button>
      </div>

      {#if errorMessage}
        <div class="error-message">{errorMessage}</div>
      {/if}
    </div>

    <div class="tags-list">
      <h4>Current Tags</h4>
      {#if tags.length === 0}
        <div class="no-tags">No tags available</div>
      {:else}
        <div class="tags-grid">
          {#each tags as tag}
            <div
              class="tag-item"
              style={tag.color ? `border-left: 4px solid ${tag.color}` : ""}
            >
              {tag.name}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .tag-management {
    padding: 16px;
    background: rgba(128, 128, 128, 0.05);
    border-radius: 8px;
    margin-bottom: 16px;
  }

  h3 {
    margin-top: 0;
    margin-bottom: 16px;
    font-size: 18px;
    font-weight: 600;
  }

  h4 {
    margin-top: 20px;
    margin-bottom: 12px;
    font-size: 16px;
    font-weight: 500;
  }

  .loading {
    padding: 8px;
    text-align: center;
    color: rgba(128, 128, 128, 0.8);
  }

  .tag-form {
    margin-bottom: 16px;
  }

  .input-group {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .tag-name-input {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid rgba(128, 128, 128, 0.2);
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.8);
    color: inherit;
  }

  .tag-color-input {
    width: 40px;
    height: 36px;
    padding: 0;
    border: 1px solid rgba(128, 128, 128, 0.2);
    border-radius: 4px;
    cursor: pointer;
  }

  .create-tag-btn {
    padding: 8px 16px;
    background: rgba(100, 120, 255, 0.2);
    border: none;
    border-radius: 4px;
    cursor: pointer;
    color: inherit;
    font-weight: 500;
    transition: background 0.2s;
  }

  .create-tag-btn:hover {
    background: rgba(100, 120, 255, 0.3);
  }

  .error-message {
    margin-top: 8px;
    color: #ff3333;
    font-size: 14px;
  }

  .tags-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 8px;
  }

  .tag-item {
    padding: 8px 12px;
    background: rgba(128, 128, 128, 0.1);
    border-radius: 4px;
    display: flex;
    align-items: center;
    font-size: 14px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .no-tags {
    padding: 8px;
    color: rgba(128, 128, 128, 0.8);
    font-style: italic;
  }

  /* dark mode */
  :global(html.dark) .tag-name-input {
    background: rgba(40, 40, 40, 0.8);
  }
</style>
