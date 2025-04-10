<script lang="ts">
  import TitleBar from "../lib/components/TitleBar.svelte";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let useAcrylic = $state(true);

  function handleTitleBarMouseDown(event: MouseEvent) {
    if (event.target instanceof HTMLElement) {
      if (event.target.closest(".window-controls") === null) {
        WebviewWindow.getCurrent().startDragging();
      }
    }
  }

  async function toggleAcrylic() {
    try {
      await invoke("set_acrylic_effect", { enable: useAcrylic });
    } catch (error) {
      console.error("Failed to toggle acrylic:", error);
    }
  }

  onMount(() => {
    toggleAcrylic();
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
    <TitleBar />
  </div>
  <div class="content">
    <label>
      <input
        type="checkbox"
        bind:checked={useAcrylic}
        onchange={toggleAcrylic}
      />
      Acrylic Effect
    </label>
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
    background: rgba(50, 50, 50, 1);
  }

  .titlebar-container {
    flex-shrink: 0;
    z-index: 1; /* ensure titlebar stays above background */
  }

  .content {
    flex: 1;
    overflow: auto;
    padding: 10px;
    z-index: 1; /* ensure content stays above background */
  }

  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: transparent;
  }

  @media (prefers-color-scheme: dark) {
    .static-bg {
      background: rgba(30, 30, 30, 1);
    }
  }
</style>
