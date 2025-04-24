<script lang="ts">
  // props
  let {
    isNew,
    note = null,
    folders = [],
    tags = [],
    onSave,
    onCancel,
  } = $props();

  // track current note ID to detect changes in the selected note
  let currentNoteId = $state(note?.id || null);

  // initialize note data based on whether it's new or existing
  let title = $state(isNew ? "" : note?.title || "");
  let content = $state(isNew ? "" : note?.content || "");
  let folderId = $state(isNew ? null : note?.folder_id);
  let isPinned = $state(isNew ? false : note?.is_pinned || false);
  let isArchived = $state(isNew ? false : note?.is_archived || false);
  let color = $state(isNew ? null : note?.color);
  let selectedTags = $state<string[]>(isNew ? [] : note?.tags || []);

  // reset form data when the note changes
  $effect(() => {
    if (note?.id !== currentNoteId) {
      currentNoteId = note?.id || null;
      title = note?.title || "";
      content = note?.content || "";
      folderId = note?.folder_id || null;
      isPinned = note?.is_pinned || false;
      isArchived = note?.is_archived || false;
      color = note?.color || null;
      selectedTags = note?.tags || [];
    }
  });

  // handle tag selection
  function toggleTag(tagName: string) {
    if (selectedTags.includes(tagName)) {
      selectedTags = selectedTags.filter((t) => t !== tagName);
    } else {
      selectedTags = [...selectedTags, tagName];
    }
  }

  // handle note saving
  function saveNote() {
    const noteData = {
      id: isNew ? undefined : note.id,
      title: title.trim() || "Untitled",
      content,
      folderId,
      tags: selectedTags,
      isPinned,
      isArchived,
      color,
    };

    onSave(noteData);
  }

  // auto-resize textarea
  function autoResizeTextarea(e: Event) {
    const textarea = e.target as HTMLTextAreaElement;
    textarea.style.height = "auto";
    textarea.style.height = `${textarea.scrollHeight}px`;
  }
</script>

<div class="note-editor">
  <div class="editor-header">
    <input
      type="text"
      class="title-input"
      placeholder="Untitled"
      bind:value={title}
    />

    <div class="note-controls">
      <div class="tags-dropdown">
        <button class="tag-button">Tags</button>
        <div class="tags-dropdown-content">
          {#if tags.length === 0}
            <div class="no-tags">No tags available</div>
          {:else}
            {#each tags as tag}
              <label class="tag-checkbox">
                <input
                  type="checkbox"
                  checked={selectedTags.includes(tag.name)}
                  onchange={() => toggleTag(tag.name)}
                />
                <span>{tag.name}</span>
              </label>
            {/each}
          {/if}
        </div>
      </div>

      <select bind:value={folderId} class="folder-select">
        <option value={null}>No folder</option>
        {#each folders as folder}
          <option value={folder.id}>{folder.name}</option>
        {/each}
      </select>

      <button
        class="pin-button"
        class:active={isPinned}
        title={isPinned ? "Unpin note" : "Pin note"}
        onclick={() => (isPinned = !isPinned)}
      >
        📌
      </button>
    </div>
  </div>

  <div class="editor-content">
    <textarea
      class="content-textarea"
      placeholder="Write your note here..."
      bind:value={content}
      oninput={autoResizeTextarea}
    ></textarea>
  </div>

  <div class="editor-footer">
    <div class="selected-tags">
      {#each selectedTags as tag}
        <span class="selected-tag">
          {tag}
          <button class="remove-tag" onclick={() => toggleTag(tag)}>×</button>
        </span>
      {/each}
    </div>

    <div class="actions">
      <button class="cancel-button" onclick={onCancel}>Cancel</button>
      <button class="save-button" onclick={saveNote}>Save</button>
    </div>
  </div>
</div>

<style>
  .note-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 16px;
  }

  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    flex-wrap: wrap;
    gap: 12px;
  }

  .title-input {
    font-size: 24px;
    font-weight: 600;
    border: none;
    background: transparent;
    outline: none;
    width: 100%;
    max-width: 500px;
    color: inherit;
  }

  .note-controls {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .tags-dropdown {
    position: relative;
    display: inline-block;
  }

  .tag-button {
    padding: 6px 12px;
    background: rgba(128, 128, 128, 0.1);
    border: none;
    border-radius: 4px;
    cursor: pointer;
    color: inherit;
  }

  .tags-dropdown-content {
    display: none;
    position: absolute;
    background: rgba(245, 245, 245, 0.95);
    min-width: 160px;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.1);
    z-index: 1;
    border-radius: 4px;
    overflow: hidden;
    right: 0;
    max-height: 300px;
    overflow-y: auto;
  }

  .tags-dropdown:hover .tags-dropdown-content {
    display: block;
  }

  .tag-checkbox {
    display: block;
    padding: 8px 12px;
    cursor: pointer;
    transition: background 0.2s;
    color: #333;
  }

  .tag-checkbox:hover {
    background: rgba(128, 128, 128, 0.1);
  }

  .no-tags {
    padding: 8px 12px;
    color: rgba(128, 128, 128, 0.8);
    font-style: italic;
  }

  .folder-select {
    padding: 6px 10px;
    background: rgba(128, 128, 128, 0.1);
    border: none;
    border-radius: 4px;
    color: inherit;
    cursor: pointer;
  }

  .pin-button {
    background: transparent;
    border: none;
    width: 32px;
    height: 32px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    opacity: 0.5;
    transition: all 0.2s;
    font-size: 16px;
  }

  .pin-button:hover {
    background: rgba(128, 128, 128, 0.1);
    opacity: 0.8;
  }

  .pin-button.active {
    opacity: 1;
  }

  .editor-content {
    flex: 1;
    overflow: auto;
  }

  .content-textarea {
    width: 100%;
    height: 100%;
    min-height: 300px;
    border: none;
    background: transparent;
    resize: none;
    padding: 8px 0;
    line-height: 1.6;
    font-size: 16px;
    outline: none;
    color: inherit;
  }

  .editor-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 16px;
    padding-top: 16px;
    border-top: 1px solid rgba(128, 128, 128, 0.2);
  }

  .selected-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .selected-tag {
    display: inline-flex;
    align-items: center;
    background: rgba(128, 128, 128, 0.1);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 13px;
  }

  .remove-tag {
    background: transparent;
    border: none;
    margin-left: 4px;
    cursor: pointer;
    font-size: 14px;
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    opacity: 0.7;
  }

  .remove-tag:hover {
    opacity: 1;
  }

  .actions {
    display: flex;
    gap: 8px;
  }

  .cancel-button,
  .save-button {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }

  .cancel-button {
    background: rgba(128, 128, 128, 0.1);
    color: inherit;
  }

  .save-button {
    background: rgba(100, 120, 255, 0.2);
    color: inherit;
    font-weight: 500;
  }

  .cancel-button:hover {
    background: rgba(128, 128, 128, 0.2);
  }

  .save-button:hover {
    background: rgba(100, 120, 255, 0.3);
  }

  /* dark mode */
  :global(html.dark) .tags-dropdown-content {
    background: rgba(40, 40, 40, 0.95);
  }

  :global(html.dark) .tag-checkbox {
    color: #eee;
  }
</style>
