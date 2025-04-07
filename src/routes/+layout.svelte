<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";

  // Props for rendering children
  let { children } = $props();

  // Optionally add dark mode toggle functionality
  let darkMode = false;

  // Add a function to safely call Tauri window controls
  function useTauriWindow(callback: (tauriWindow: any) => Promise<void>) {
    return async () => {
      // Make sure we're in a browser environment and Tauri is available
      if (
        typeof window !== "undefined" &&
        window.__TAURI__ &&
        window.__TAURI__.window
      ) {
        try {
          await callback(window.__TAURI__);
        } catch (e) {
          console.error("Tauri window operation failed:", e);
        }
      } else {
        console.warn("Tauri API not available");
      }
    };
  }

  // Create window control functions with safety checks
  const closeWindow = useTauriWindow(async (tauri) => {
    await tauri.window.appWindow.close();
  });

  const minimizeWindow = useTauriWindow(async (tauri) => {
    await tauri.window.appWindow.minimize();
  });

  const maximizeWindow = useTauriWindow(async (tauri) => {
    const isMaximized = await tauri.window.appWindow.isMaximized();
    if (isMaximized) {
      await tauri.window.appWindow.unmaximize();
    } else {
      await tauri.window.appWindow.maximize();
    }
  });

  function toggleDarkMode() {
    darkMode = !darkMode;
    if (darkMode) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }

  // Suppress SSR warnings by setting up Tauri only after component is mounted
  onMount(() => {
    // Log Tauri availability for debugging
    if (typeof window !== "undefined") {
      console.log("Tauri available:", !!window.__TAURI__);
      if (window.__TAURI__) {
        console.log("Window API available:", !!window.__TAURI__.window);
      }
    }
  });
</script>

<svelte:head>
  <!-- Add type definition for the Tauri global object -->
  <script>
    // Prevent errors during SSR by creating a placeholder for Tauri
    if (typeof window !== "undefined" && !window.__TAURI__) {
      console.warn(
        "Tauri API not found. This might be expected during development."
      );
    }
  </script>
</svelte:head>

<!-- Apply the glass container to the entire app -->
<div class="glass-container">
  <!-- Add a custom macOS-style window titlebar since decorations are disabled -->
  <div class="titlebar drag-region flex items-center p-2">
    <!-- macOS window controls (left side) -->
    <div class="macos-window-controls ml-2 flex items-center gap-1.5">
      <!-- Close button -->
      <button
        class="window-control close-button flex h-3 w-3 items-center justify-center rounded-full bg-red-500 hover:bg-red-600"
        on:click={closeWindow}
      >
        <svg
          class="h-2 w-2 opacity-0 hover:opacity-100"
          viewBox="0 0 10 10"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <path d="M2 2L8 8M8 2L2 8" />
        </svg>
      </button>

      <!-- Minimize button -->
      <button
        class="window-control minimize-button flex h-3 w-3 items-center justify-center rounded-full bg-yellow-500 hover:bg-yellow-600"
        on:click={minimizeWindow}
      >
        <svg
          class="h-2 w-2 opacity-0 hover:opacity-100"
          viewBox="0 0 10 10"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <path d="M2 5H8" />
        </svg>
      </button>

      <!-- Maximize/fullscreen button -->
      <button
        class="window-control maximize-button flex h-3 w-3 items-center justify-center rounded-full bg-green-500 hover:bg-green-600"
        on:click={maximizeWindow}
      >
        <svg
          class="h-2 w-2 opacity-0 hover:opacity-100"
          viewBox="0 0 10 10"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
        >
          <rect x="2" y="2" width="6" height="6" />
        </svg>
      </button>
    </div>

    <!-- App title (center) -->
    <div class="app-title flex-1 text-center text-sm">apto</div>

    <!-- Right side controls -->
    <div class="window-right-controls mr-2 flex gap-2">
      <button
        on:click={toggleDarkMode}
        class="rounded-full p-1 hover:bg-gray-200 dark:hover:bg-gray-700"
      >
        {#if darkMode}
          <!-- Sun icon for light mode -->
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-4 w-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <circle cx="12" cy="12" r="5"></circle>
            <path
              d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"
            ></path>
          </svg>
        {:else}
          <!-- Moon icon for dark mode -->
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-4 w-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
          </svg>
        {/if}
      </button>
    </div>
  </div>

  <!-- App content -->
  <div class="app-content p-4">
    {@render children()}
  </div>
</div>

<style>
  /* Make the titlebar draggable */
  .drag-region {
    -webkit-app-region: drag;
    app-region: drag;
  }

  /* Make buttons not draggable */
  .window-control,
  .window-right-controls button {
    -webkit-app-region: no-drag;
    app-region: no-drag;
  }

  /* macOS window control styles */
  .macos-window-controls {
    display: flex;
    align-items: center;
  }

  .macos-window-controls button {
    position: relative;
    transition: all 0.2s;
  }

  /* Show icons on hover */
  .macos-window-controls button svg {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    transition: opacity 0.2s;
  }

  /* Dark mode adjustments for window controls */
  :global(.dark) .close-button {
    background-color: rgba(255, 59, 48, 0.8);
  }

  :global(.dark) .minimize-button {
    background-color: rgba(255, 204, 0, 0.8);
  }

  :global(.dark) .maximize-button {
    background-color: rgba(40, 205, 65, 0.8);
  }

  :global(.dark) .window-control svg {
    stroke: rgba(0, 0, 0, 0.5);
  }
</style>
