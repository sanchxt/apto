<script lang="ts">
  import { onMount } from "svelte";

  // props
  let {
    notes,
    folders,
    selectedNoteId,
    onSelectNote,
    onDeleteNote,
    onTogglePin,
    onMoveNote,
  } = $props();

  // format date for display
  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const yesterday = new Date(now);
    yesterday.setDate(yesterday.getDate() - 1);

    // if today, show time only
    if (date.toDateString() === now.toDateString()) {
      return date.toLocaleTimeString([], {
        hour: "2-digit",
        minute: "2-digit",
      });
    }
    // if yesterday, show "Yesterday"
    else if (date.toDateString() === yesterday.toDateString()) {
      return "Yesterday";
    }
    // otherwise show date
    else {
      return date.toLocaleDateString([], { month: "short", day: "numeric" });
    }
  }

  // preview of the note content
  function getContentPreview(content: string, maxLength: number = 100): string {
    // remove markdown symbols for better preview
    const cleanContent = content.replace(/#{1,6} /g, "").replace(/[*_~`]/g, "");

    return cleanContent.length > maxLength
      ? cleanContent.substring(0, maxLength) + "..."
      : cleanContent;
  }

  // handle delete button click
  function handleDeleteNote(event: Event, note: any) {
    event.stopPropagation();
    onDeleteNote(note);
  }

  // handle pin/unpin button click
  function handleTogglePin(event: Event, note: any) {
    event.stopPropagation();
    onTogglePin(note);
  }

  // handle folder change
  function handleMoveNote(event: Event, note: any, folderId: number | null) {
    event.stopPropagation();
    onMoveNote(note, folderId);
  }

  // get tag color from tag name (if tag has associated color object)
  function getTagStyle(tagName: string, note: any): string {
    if (!note.tag_colors || !note.tag_colors[tagName]) {
      return "";
    }
    return `background-color: ${note.tag_colors[tagName]}; color: ${getContrastTextColor(note.tag_colors[tagName])}`;
  }

  // get folder name from folder id
  function getFolderName(folderId: number | null): string {
    if (folderId === null) return "";

    const folder = folders.find((f: any) => f.id === folderId);
    return folder ? folder.name : "";
  }

  // get folder color
  function getFolderColor(folderId: number | null): string | null {
    if (folderId === null) return null;

    const folder = folders.find((f: any) => f.id === folderId);
    return folder ? folder.color : null;
  }

  // generate style for folder indicator
  function getFolderStyle(folderId: number | null): string {
    const folderColor = getFolderColor(folderId);
    if (!folderColor) return "";

    return `background-color: ${folderColor}; color: ${getContrastTextColor(folderColor)};`;
  }

  // determine text color (black or white) based on background color
  function getContrastTextColor(hexColor: string): string {
    if (!hexColor || !hexColor.startsWith("#")) {
      return "inherit";
    }

    // convert hex to RGB
    let r = 0,
      g = 0,
      b = 0;

    // 3 digits
    if (hexColor.length === 4) {
      r = parseInt(hexColor[1] + hexColor[1], 16);
      g = parseInt(hexColor[2] + hexColor[2], 16);
      b = parseInt(hexColor[3] + hexColor[3], 16);
    }
    // 6 digits
    else if (hexColor.length === 7) {
      r = parseInt(hexColor.substring(1, 3), 16);
      g = parseInt(hexColor.substring(3, 5), 16);
      b = parseInt(hexColor.substring(5, 7), 16);
    } else {
      return "inherit";
    }

    // calculate relative luminance using YIQ formula
    // YIQ = (Y * 299 + I * 587 + Q * 114) / 1000
    const yiq = (r * 299 + g * 587 + b * 114) / 1000;

    // return black or white depending on luminance
    return yiq >= 128 ? "#000000" : "#ffffff";
  }

  // show folder dropdown
  function showFolderDropdown(event: Event, noteItem: HTMLElement | null) {
    event.stopPropagation();
    if (!noteItem) return;

    const dropdown = noteItem.querySelector(".folder-dropdown") as HTMLElement;
    if (dropdown) {
      // close all other open dropdowns
      const allDropdowns = document.querySelectorAll(".folder-dropdown.show");
      allDropdowns.forEach((dd) => {
        if (dd !== dropdown) dd.classList.remove("show");
      });

      dropdown.classList.toggle("show");
    }
  }

  // close folder dropdown when clicking outside
  function setupDropdownCloseListener() {
    function closeDropdowns(e: MouseEvent) {
      if (!(e.target as HTMLElement).matches(".folder-btn, .folder-btn *")) {
        const dropdowns = document.querySelectorAll(".folder-dropdown.show");
        dropdowns.forEach((dd) => dd.classList.remove("show"));
      }
    }

    document.addEventListener("click", closeDropdowns);
    return () => document.removeEventListener("click", closeDropdowns);
  }

  // set up event listener on mount and clean up on unmount
  onMount(setupDropdownCloseListener);
</script>

<div class="notes-list" role="listbox" aria-label="Notes list">
  {#if notes.length === 0}
    <div class="empty-list">
      <p>No notes yet</p>
    </div>
  {:else}
    {#each notes as note}
      <div
        class="note-item"
        class:selected={note.id === selectedNoteId}
        class:pinned={note.is_pinned}
        onclick={() => onSelectNote(note)}
        onkeydown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            onSelectNote(note);
            e.preventDefault();
          }
        }}
        aria-selected={note.id === selectedNoteId}
        role="option"
        tabindex="0"
      >
        <div class="note-content">
          <div class="note-title">{note.title || "Untitled"}</div>
          <div class="note-preview">{getContentPreview(note.content)}</div>
          <div class="note-meta">
            <span class="note-date">{formatDate(note.updated_at)}</span>
            <div class="note-details">
              {#if note.folder_id}
                <span
                  class="folder-indicator"
                  style={getFolderStyle(note.folder_id)}
                >
                  📁 {getFolderName(note.folder_id)}
                </span>
              {/if}
              {#if note.tags.length > 0}
                <div class="note-tags">
                  {#if note.tags.length > 2}
                    <span class="more-tags">+{note.tags.length - 2}</span>
                  {/if}
                  {#each note.tags.slice(0, 2) as tag}
                    <span class="tag" style={getTagStyle(tag, note)}>{tag}</span
                    >
                  {/each}
                </div>
              {/if}
            </div>
          </div>
        </div>
        <div
          class="note-actions"
          role="presentation"
          onclick={(e) => e.stopPropagation()}
          onkeydown={(e) => e.stopPropagation()}
        >
          <div class="folder-dropdown-container">
            <span
              class="folder-btn"
              title="Move to folder"
              onclick={(e) =>
                showFolderDropdown(e, e.currentTarget.closest(".note-item")!)}
              onkeydown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  showFolderDropdown(e, e.currentTarget.closest(".note-item")!);
                  e.preventDefault();
                }
              }}
              aria-label="Move to folder"
              role="button"
              tabindex="0"
            >
              📁
            </span>
            <div class="folder-dropdown">
              <button
                class="folder-dropdown-item"
                onclick={(e) => handleMoveNote(e, note, null)}
              >
                No folder
              </button>
              {#each folders as folder}
                <button
                  class="folder-dropdown-item"
                  onclick={(e) => handleMoveNote(e, note, folder.id)}
                  style={folder.color
                    ? `border-left: 3px solid ${folder.color}`
                    : ""}
                >
                  {folder.name}
                </button>
              {/each}
            </div>
          </div>
          <span
            class="pin-btn"
            class:active={note.is_pinned}
            title={note.is_pinned ? "Unpin note" : "Pin note"}
            onclick={(e) => handleTogglePin(e, note)}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                handleTogglePin(e, note);
                e.preventDefault();
              }
            }}
            aria-label={note.is_pinned ? "Unpin note" : "Pin note"}
            role="button"
            tabindex="0"
          >
            📌
          </span>
          <span
            class="delete-btn"
            title="Delete note"
            onclick={(e) => handleDeleteNote(e, note)}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                handleDeleteNote(e, note);
                e.preventDefault();
              }
            }}
            aria-label="Delete note"
            role="button"
            tabindex="0"
          >
            ×
          </span>
        </div>
      </div>
    {/each}
  {/if}
</div>

<style>
  .notes-list {
    overflow-y: auto;
    flex: 1;
  }

  .empty-list {
    padding: 20px;
    text-align: center;
    color: rgba(128, 128, 128, 0.8);
  }

  .note-item {
    width: 100%;
    box-sizing: border-box;
    padding: 12px 16px;
    cursor: pointer;
    transition: background 0.2s;
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    text-align: left;
    background: transparent;
    border: none;
    border-bottom: 1px solid rgba(128, 128, 128, 0.1);
    color: inherit;
    font-family: inherit;
    font-size: inherit;
    outline: none;
    position: relative;
    overflow: visible;
  }

  .note-item:focus-visible {
    box-shadow: inset 0 0 0 2px rgba(100, 120, 255, 0.5);
  }

  .note-item:hover {
    background: rgba(128, 128, 128, 0.05);
  }

  .note-item.selected {
    background: rgba(128, 128, 128, 0.1);
  }

  .note-item.pinned::before {
    content: "📌";
    position: absolute;
    left: 4px;
    font-size: 10px;
    opacity: 0.7;
  }

  .note-content {
    flex: 1;
    min-width: 0;
    padding-right: 85px;
    max-width: calc(100% - 10px);
  }

  .note-title {
    font-weight: 500;
    margin-bottom: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .note-preview {
    font-size: 13px;
    color: rgba(128, 128, 128, 0.9);
    margin-bottom: 6px;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    line-height: 1.3;
  }

  .note-meta {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
    color: rgba(128, 128, 128, 0.7);
  }

  .note-date {
    flex-shrink: 0;
  }

  .note-details {
    display: flex;
    align-items: center;
    gap: 6px;
    max-width: 70%;
    overflow: hidden;
  }

  .folder-indicator {
    padding: 2px 6px;
    background: rgba(128, 128, 128, 0.1);
    border-radius: 4px;
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100px;
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .folder-dropdown {
    display: none;
    position: absolute;
    top: 28px;
    right: 0;
    background-color: rgba(255, 255, 255, 0.95);
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    min-width: 150px;
    z-index: 10;
    max-height: 300px;
    overflow-y: auto;
  }

  .folder-dropdown-item {
    padding: 8px 12px;
    cursor: pointer;
    font-size: 13px;
    color: inherit;
    transition: background 0.2s;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    border-left: 3px solid transparent;
  }

  .folder-dropdown-item:hover {
    background-color: rgba(0, 0, 0, 0.05);
  }

  .note-tags {
    display: flex;
    gap: 4px;
    flex-wrap: nowrap;
    overflow: hidden;
    flex-direction: row-reverse;
  }

  .tag {
    padding: 2px 6px;
    background: rgba(128, 128, 128, 0.1);
    border-radius: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100px;
  }

  .more-tags {
    font-size: 11px;
    opacity: 0.7;
    flex-shrink: 0;
    margin-right: 2px;
  }

  .note-actions {
    position: absolute;
    right: 10px;
    top: 10px;
    opacity: 0;
    transition: opacity 0.2s;
    z-index: 2;
    display: flex;
    gap: 4px;
  }

  .note-item:hover .note-actions {
    opacity: 1;
  }

  .folder-dropdown-container {
    position: relative;
  }

  .folder-btn,
  .pin-btn,
  .delete-btn {
    background: transparent;
    border: none;
    color: inherit;
    font-size: 16px;
    cursor: pointer;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    opacity: 0.5;
    transition: all 0.2s;
  }

  .folder-btn:hover,
  .folder-btn:focus {
    background: rgba(0, 128, 255, 0.1);
    opacity: 1;
    outline: none;
  }

  .pin-btn {
    font-size: 14px;
  }

  .pin-btn.active {
    opacity: 1;
    color: #ffa500;
  }

  .pin-btn:hover,
  .pin-btn:focus {
    background: rgba(255, 165, 0, 0.1);
    opacity: 1;
    outline: none;
  }

  .delete-btn:hover,
  .delete-btn:focus {
    background: rgba(255, 0, 0, 0.1);
    color: #ff3333;
    opacity: 1;
    outline: none;
  }

  .note-item.pinned .pin-btn {
    opacity: 1;
    color: #ffa500;
  }

  /* Folder dropdown styles */
  .folder-dropdown {
    position: absolute;
    top: 100%;
    right: 0;
    background-color: rgba(255, 255, 255, 0.95);
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    min-width: 150px;
    max-height: 0;
    overflow: hidden;
    opacity: 0;
    transition: all 0.2s;
    z-index: 10;
    transform-origin: top right;
    transform: scale(0.95);
    pointer-events: none;
  }

  .folder-dropdown-item {
    padding: 8px 12px;
    cursor: pointer;
    font-size: 13px;
    color: inherit;
    transition: background 0.2s;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .folder-dropdown-item:hover {
    background-color: rgba(0, 0, 0, 0.05);
  }

  /* Dark mode styles */
  :global(html.dark) .folder-dropdown {
    background-color: rgba(40, 40, 40, 0.95);
  }

  :global(html.dark) .folder-dropdown-item:hover {
    background-color: rgba(255, 255, 255, 0.1);
  }
</style>
