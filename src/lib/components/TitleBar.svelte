<script lang="ts">
    import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

    // Get the current window
    const appWindow = WebviewWindow.getCurrent();

    // Track window maximized state
    let isMaximized = $state(false);

    // Update maximized state on component mount and when state changes
    async function updateMaximizedState() {
      isMaximized = await appWindow.isMaximized();
    }

    // Set up listener for window resize/state changes
    appWindow.onResized(updateMaximizedState);

    // Initial state check
    updateMaximizedState();

    // Functions to handle window controls
    async function minimizeWindow() {
      await appWindow.minimize();
    }

    async function maximizeWindow() {
      if (isMaximized) {
        await appWindow.unmaximize();
      } else {
        await appWindow.maximize();
      }
      // Update state after action
      updateMaximizedState();
    }

    async function closeWindow() {
      await appWindow.close();
    }
  </script>

  <div class="titlebar">
    <div class="window-controls">
      <button class="close" onclick={closeWindow} aria-label="Close Window">
        <svg width="8" height="8" viewBox="0 0 8 8">
          <path d="M1.5,1.5 L6.5,6.5 M1.5,6.5 L6.5,1.5" stroke="currentColor" stroke-width="1.25" />
        </svg>
      </button>
      <button class="minimize" onclick={minimizeWindow} aria-label="Minimize Window">
        <svg width="8" height="8" viewBox="0 0 8 8">
          <rect width="6" height="1" x="1" y="3.5" fill="currentColor" />
        </svg>
      </button>
      <button class="maximize" onclick={maximizeWindow} aria-label="Maximize Window">
        {#if isMaximized}
          <svg width="8" height="8" viewBox="0 0 8 8">
            <path d="M2.5,2.5 v3 h3 v-3 h-3 M1.5,1.5 v5 h5 v-5 h-5" fill="none" stroke="currentColor" stroke-width="1" />
          </svg>
        {:else}
          <svg width="8" height="8" viewBox="0 0 8 8">
            <path d="M1.5,1.5 v5 h5 v-5 h-5" fill="none" stroke="currentColor" stroke-width="1" />
          </svg>
        {/if}
      </button>
    </div>
    <div class="title">apto</div>
  </div>

  <style>
    .titlebar {
      height: 32px;
      background: #f6f6f6;
      display: flex;
      justify-content: center;
      align-items: center;
      padding: 0 12px;
      -webkit-user-select: none;
      user-select: none;
      cursor: default;
      position: relative;
    }

    .title {
      color: #333333;
      font-size: 13px;
      font-weight: 500;
    }

    .window-controls {
      display: flex;
      gap: 8px;
      position: absolute;
      left: 12px;
    }

    button {
      display: flex;
      justify-content: center;
      align-items: center;
      width: 12px;
      height: 12px;
      padding: 0;
      background: #dddddd;
      border: none;
      outline: none;
      box-shadow: none;
      border-radius: 50%;
      color: transparent;
      cursor: pointer;
      transition: color 0.1s;
    }

    button svg {
      width: 8px;
      height: 8px;
      opacity: 0;
      transition: opacity 0.1s;
    }

    .close {
      background: #ff5f57;
    }

    .minimize {
      background: #ffbd2e;
    }

    .maximize {
      background: #28c940;
    }

    /* Show icons on hover */
    button:hover svg {
      opacity: 1;
    }

    .close:hover {
      color: #5d0000;
    }

    .minimize:hover {
      color: #975500;
    }

    .maximize:hover {
      color: #006500;
    }

    /* Dark mode support */
    @media (prefers-color-scheme: dark) {
      .titlebar {
        background: #2f2f2f;
      }

      .title {
        color: #f6f6f6;
      }

      button svg {
        opacity: 0;
      }

      button:hover svg {
        opacity: 1;
      }
    }
  </style>
