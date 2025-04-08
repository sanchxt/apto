<script lang="ts">
    import TitleBar from '../lib/components/TitleBar.svelte';

    // Enable window dragging on the title bar
    import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

    function handleTitleBarMouseDown(event: MouseEvent) {
      if (event.target instanceof HTMLElement) {
        // Only initiate dragging if clicking on the titlebar itself (not buttons)
        if (event.target.closest('.window-controls') === null) {
          WebviewWindow.getCurrent().startDragging();
        }
      }
    }
  </script>

  <div class="app">
    <!-- Title Bar with custom window controls -->
    <div class="titlebar-container" on:mousedown={handleTitleBarMouseDown} role="presentation">
      <TitleBar />
    </div>

    <!-- App Content -->
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
    }

    .titlebar-container {
      flex-shrink: 0;
    }

    .content {
      flex: 1;
      overflow: auto;
    }

    /* Reset any margin/padding for the whole app */
    :global(body) {
      margin: 0;
      padding: 0;
      overflow: hidden;
    }
  </style>
