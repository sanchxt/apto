<script lang="ts">
  // props
  let { noteTitle = "this note", onConfirm, onCancel } = $props();

  // handle transition
  let modalElement: HTMLElement;

  // focus confirm button on mount
  let confirmButton: HTMLButtonElement;

  $effect(() => {
    // focus the cancel button when modal opens
    if (confirmButton) {
      setTimeout(() => confirmButton.focus(), 50);
    }
  });

  // handle keyboard events
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onCancel();
    } else if (
      event.key === "Enter" &&
      document.activeElement === confirmButton
    ) {
      onConfirm();
    }
  }
</script>

<div class="modal-backdrop" onkeydown={handleKeydown} role="presentation">
  <div
    class="modal-container"
    bind:this={modalElement}
    role="dialog"
    aria-labelledby="confirm-delete-title"
    aria-describedby="confirm-delete-description"
  >
    <div class="modal-content">
      <h3 id="confirm-delete-title">Delete Note</h3>
      <p id="confirm-delete-description">
        Are you sure you want to delete "{noteTitle}"? This action cannot be
        undone.
      </p>

      <div class="modal-actions">
        <button class="cancel-button" onclick={onCancel} type="button">
          Cancel
        </button>
        <button
          class="delete-button"
          onclick={onConfirm}
          type="button"
          bind:this={confirmButton}
        >
          Delete
        </button>
      </div>
    </div>
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
    max-width: 400px;
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
    padding: 20px;
  }

  h3 {
    margin: 0 0 12px 0;
    font-size: 18px;
    font-weight: 600;
    color: #333;
  }

  p {
    margin: 0 0 20px 0;
    font-size: 14px;
    line-height: 1.5;
    color: #555;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 16px;
  }

  .cancel-button,
  .delete-button {
    padding: 8px 16px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    border: none;
  }

  .cancel-button {
    background-color: rgba(0, 0, 0, 0.05);
    color: #555;
  }

  .cancel-button:hover {
    background-color: rgba(0, 0, 0, 0.1);
  }

  .delete-button {
    background-color: rgba(220, 53, 69, 0.8);
    color: white;
  }

  .delete-button:hover {
    background-color: rgba(220, 53, 69, 1);
  }

  .delete-button:focus,
  .cancel-button:focus {
    outline: 2px solid rgba(100, 120, 255, 0.5);
  }

  /* Dark mode styles */
  :global(html.dark) .modal-container {
    background-color: rgba(40, 40, 40, 0.95);
  }

  :global(html.dark) h3 {
    color: #eee;
  }

  :global(html.dark) p {
    color: #ccc;
  }

  :global(html.dark) .cancel-button {
    background-color: rgba(255, 255, 255, 0.1);
    color: #ddd;
  }

  :global(html.dark) .cancel-button:hover {
    background-color: rgba(255, 255, 255, 0.15);
  }

  :global(html.dark) .delete-button {
    background-color: rgba(220, 53, 69, 0.7);
  }

  :global(html.dark) .delete-button:hover {
    background-color: rgba(220, 53, 69, 0.9);
  }
</style>
