<script lang="ts">
  import TitleBar from "../lib/components/TitleBar.svelte";

  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

  function handleTitleBarMouseDown(event: MouseEvent) {
    if (event.target instanceof HTMLElement) {
      // initiate dragging if clicking on the titlebar only
      if (event.target.closest(".window-controls") === null) {
        WebviewWindow.getCurrent().startDragging();
      }
    }
  }
</script>

<div class="app glass-container">
  <!-- window toolbar -->
  <div
    class="titlebar-container"
    on:mousedown={handleTitleBarMouseDown}
    role="presentation"
  >
    <TitleBar />
  </div>

  <!-- app content -->
  <div class="content">
    <slot />
  </div>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    margin: 0;
    padding: 0;
    border-radius: 12px;
    overflow: hidden;
  }

  .glass-container {
    background: rgba(255, 255, 255, 0.25);
    backdrop-filter: blur(80px);
    -webkit-backdrop-filter: blur(80px);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.12);
  }

  .titlebar-container {
    flex-shrink: 0;
  }

  .content {
    flex: 1;
    overflow: auto;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: transparent;
  }

  /* dark mode */
  @media (prefers-color-scheme: dark) {
    .glass-container {
      background: rgba(30, 30, 30, 0.4);
      border: 1px solid rgba(100, 100, 100, 0.12);
    }
  }
</style>
