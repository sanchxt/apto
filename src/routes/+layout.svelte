<script lang="ts">
  import TitleBar from "../lib/components/TitleBar.svelte";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  // states
  let useAcrylic = $state(true);
  let currentTheme = $state("system");

  // get current window
  const appWindow = WebviewWindow.getCurrent();

  // handle titlebar mouse down for dragging
  function handleTitleBarMouseDown(event: MouseEvent) {
    if (event.target instanceof HTMLElement) {
      if (
        event.target.closest(".window-controls") === null &&
        event.target.closest(".right-controls") === null
      ) {
        appWindow.startDragging();
      }
    }
  }

  // update theme and acrylic
  async function updateAppearance() {
    if (useAcrylic) {
      await invoke("set_acrylic_effect", { enable: true });
    } else {
      await invoke("set_acrylic_effect", { enable: false });
    }
  }

  // listen for theme changes and apply them
  onMount(async () => {
    await updateAppearance();

    // force theme application on initial load
    const initialSystemTheme = (await invoke("get_system_theme")) as string;
    console.log("Initial system theme:", initialSystemTheme);

    // set explicit theme class on HTML element
    if (initialSystemTheme === "dark") {
      document.documentElement.classList.add("dark");
      document.documentElement.classList.remove("light");
    } else {
      document.documentElement.classList.add("light");
      document.documentElement.classList.remove("dark");
    }

    appWindow.listen("tauri://theme-changed", (event) => {
      const theme =
        typeof event.payload === "string"
          ? event.payload
          : (event.payload as { theme?: string })?.theme || "system";

      currentTheme = theme;
      console.log("Theme changed to:", theme);

      // force refresh the UI when theme changes
      if (!useAcrylic) {
        // small timeout to ensure DOM updates
        setTimeout(() => {
          const staticBg = document.querySelector(".static-bg");
          if (staticBg) {
            staticBg.classList.remove("static-bg");
            void (staticBg as HTMLElement).offsetWidth; // force reflow
            staticBg.classList.add("static-bg");
          }
        }, 50);
      }
    });
  });

  let { children } = $props();
</script>

<div
  class="app"
  class:glass-container={useAcrylic}
  class:static-bg={!useAcrylic}
>
  <div
    class="titlebar-container"
    onmousedown={handleTitleBarMouseDown}
    role="presentation"
  >
    <TitleBar bind:useAcrylic bind:currentTheme />
  </div>
  <div class="content">
    {@render children()}
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
    position: relative;
  }

  .glass-container {
    background: transparent;
  }

  .static-bg {
    background: rgba(245, 245, 245, 1);
    color: #333333;
  }

  .titlebar-container {
    flex-shrink: 0;
    z-index: 1;
  }

  .content {
    flex: 1;
    overflow: auto;
    padding: 10px;
    z-index: 1;
    color: inherit;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: transparent;
  }

  :global(html) {
    color-scheme: light dark;
  }

  /* Custom scrollbar styles */
  :global(*::-webkit-scrollbar) {
    width: 8px;
    height: 8px;
  }

  :global(*::-webkit-scrollbar-track) {
    background: transparent;
  }

  :global(*::-webkit-scrollbar-thumb) {
    border-radius: 4px;
    border: 2px solid transparent;
    background-clip: content-box;
    background-color: rgba(128, 128, 128, 0.4);
  }

  :global(*::-webkit-scrollbar-thumb:hover) {
    background-color: rgba(128, 128, 128, 0.5);
  }

  /* Firefox scrollbar styles */
  :global(*) {
    scrollbar-width: thin;
    scrollbar-color: rgba(128, 128, 128, 0.4) transparent;
  }

  /* Theme-specific scrollbar colors */
  :global(html.light *::-webkit-scrollbar-thumb) {
    background-color: rgba(0, 0, 0, 0.2);
  }

  :global(html.light *::-webkit-scrollbar-thumb:hover) {
    background-color: rgba(0, 0, 0, 0.3);
  }

  :global(html.light *) {
    scrollbar-color: rgba(0, 0, 0, 0.2) transparent;
  }

  :global(html.dark *::-webkit-scrollbar-thumb) {
    background-color: rgba(255, 255, 255, 0.2);
  }

  :global(html.dark *::-webkit-scrollbar-thumb:hover) {
    background-color: rgba(255, 255, 255, 0.3);
  }

  :global(html.dark *) {
    scrollbar-color: rgba(255, 255, 255, 0.2) transparent;
  }

  /* light mode */
  :global(html.light) .static-bg {
    background: rgba(245, 245, 245, 1) !important;
    color: #333333 !important;
  }

  :global(html.light) .app {
    color: #333333 !important;
  }

  :global(html.light) .glass-card {
    background: rgba(255, 255, 255, 0.15) !important;
    border: 1px solid rgba(255, 255, 255, 0.12) !important;
  }

  /* dark mode */
  :global(html.dark) .static-bg {
    background: rgba(30, 30, 30, 1) !important;
    color: #f6f6f6 !important;
  }

  :global(html.dark) .app {
    color: #f6f6f6 !important;
  }

  :global(html.dark) .glass-card {
    background: rgba(50, 50, 50, 0.25) !important;
    border: 1px solid rgba(100, 100, 100, 0.15) !important;
  }

  /* fallback to OS preference if no class is present */
  @media (prefers-color-scheme: dark) {
    .static-bg {
      background: rgba(30, 30, 30, 1);
      color: #f6f6f6;
    }
  }
</style>
