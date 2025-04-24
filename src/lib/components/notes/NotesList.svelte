<script lang="ts">
  // props
  let { notes, selectedNoteId, onSelectNote, onDeleteNote } = $props();

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
    const cleanContent = content
      .replace(/#{1,6} /g, "") // remove headers
      .replace(/[*_~`]/g, ""); // remove basic formatting

    return cleanContent.length > maxLength
      ? cleanContent.substring(0, maxLength) + "..."
      : cleanContent;
  }

  // handle delete button click
  function handleDeleteNote(event: Event, note: any) {
    event.stopPropagation();
    onDeleteNote(note);
  }
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
            {#if note.tags.length > 0}
              <div class="note-tags">
                {#each note.tags.slice(0, 2) as tag}
                  <span class="tag">{tag}</span>
                {/each}
                {#if note.tags.length > 2}
                  <span class="more-tags">+{note.tags.length - 2}</span>
                {/if}
              </div>
            {/if}
          </div>
        </div>
        <div
          class="note-actions"
          role="presentation"
          onclick={(e) => e.stopPropagation()}
          onkeydown={(e) => e.stopPropagation()}
        >
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
    padding-right: 35px;
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

  .note-tags {
    display: flex;
    gap: 4px;
    flex-wrap: nowrap;
    overflow: hidden;
    max-width: 70%;
  }

  .tag {
    padding: 2px 6px;
    background: rgba(128, 128, 128, 0.1);
    border-radius: 4px;
    white-space: nowrap;
  }

  .more-tags {
    font-size: 11px;
    opacity: 0.7;
  }

  .note-actions {
    position: absolute;
    right: 10px;
    top: 10px;
    opacity: 0;
    transition: opacity 0.2s;
    z-index: 2;
  }

  .note-item:hover .note-actions {
    opacity: 1;
  }

  .delete-btn {
    background: transparent;
    border: none;
    color: inherit;
    font-size: 18px;
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

  .delete-btn:hover,
  .delete-btn:focus {
    background: rgba(255, 0, 0, 0.1);
    color: #ff3333;
    opacity: 1;
    outline: none;
  }
</style>
