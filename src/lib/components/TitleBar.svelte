<script lang="ts">
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { invoke } from "@tauri-apps/api/core";

  // get current window
  const appWindow = WebviewWindow.getCurrent();

  // track window maximized state
  let isMaximized = $state(false);

  // track acrylic & theme effect state
  let { useAcrylic = $bindable(), currentTheme = $bindable() } = $props();

  // load system theme initially
  let systemTheme = $state("light");

  // update maximized state on component mount and when state changes
  async function updateMaximizedState() {
    isMaximized = await appWindow.isMaximized();
  }

  // listener for window resize/state changes
  appWindow.onResized(updateMaximizedState);

  // initial state check
  updateMaximizedState();

  // window controls handlers
  async function minimizeWindow() {
    await appWindow.minimize();
  }

  async function maximizeWindow() {
    await appWindow.toggleMaximize();
    updateMaximizedState();
  }

  async function closeWindow() {
    await appWindow.close();
  }

  // toggle acrylic effect
  async function toggleAcrylic() {
    useAcrylic = !useAcrylic;
    try {
      await invoke("set_acrylic_effect", { enable: useAcrylic });
    } catch (error) {
      console.error("Failed to toggle acrylic:", error);
    }
  }

  // set theme
  async function setTheme(theme: string) {
    try {
      console.log("Setting theme to:", theme);
      currentTheme = theme;

      // update HTML root element class immediately for faster visual feedback
      if (theme === "light") {
        document.documentElement.classList.add("light");
        document.documentElement.classList.remove("dark");
      } else if (theme === "dark") {
        document.documentElement.classList.add("dark");
        document.documentElement.classList.remove("light");
      } else if (theme === "system") {
        // check OS preference
        const prefersDark = window.matchMedia(
          "(prefers-color-scheme: dark)"
        ).matches;
        if (prefersDark) {
          document.documentElement.classList.add("dark");
          document.documentElement.classList.remove("light");
        } else {
          document.documentElement.classList.add("light");
          document.documentElement.classList.remove("dark");
        }
      }

      // then invoke Tauri backend
      await invoke("set_theme", { theme });
    } catch (error) {
      console.error("Failed to set theme:", error);
    }
  }

  // cycle through themes (system -> light -> dark -> system)
  async function cycleTheme() {
    const nextTheme =
      currentTheme === "system"
        ? "light"
        : currentTheme === "light"
          ? "dark"
          : "system";

    console.log("Cycling theme from", currentTheme, "to", nextTheme);
    await setTheme(nextTheme);
  }

  // Initialize on mount
  import { onMount } from "svelte";

  onMount(async () => {
    try {
      // get initial system theme
      systemTheme = (await invoke("get_system_theme")) as string;
      console.log("Initial system theme:", systemTheme);

      // set initial theme to system
      await setTheme("system");
    } catch (error) {
      console.error("Initialization error:", error);
    }
  });
</script>

<div class="titlebar">
  <div class="window-controls">
    <button class="close" onclick={closeWindow} aria-label="Close Window">
      <svg width="8" height="8" viewBox="0 0 8 8">
        <path
          d="M1.5,1.5 L6.5,6.5 M1.5,6.5 L6.5,1.5"
          stroke="currentColor"
          stroke-width="1.25"
        />
      </svg>
    </button>
    <button
      class="minimize"
      onclick={minimizeWindow}
      aria-label="Minimize Window"
    >
      <svg width="8" height="8" viewBox="0 0 8 8">
        <rect width="6" height="1" x="1" y="3.5" fill="currentColor" />
      </svg>
    </button>
    <button
      class="maximize"
      onclick={maximizeWindow}
      aria-label="Maximize Window"
    >
      {#if isMaximized}
        <svg width="8" height="8" viewBox="0 0 8 8">
          <path
            d="M2.5,2.5 v3 h3 v-3 h-3 M1.5,1.5 v5 h5 v-5 h-5"
            fill="none"
            stroke="currentColor"
            stroke-width="1"
          />
        </svg>
      {:else}
        <svg width="8" height="8" viewBox="0 0 8 8">
          <path
            d="M1.5,1.5 v5 h5 v-5 h-5"
            fill="none"
            stroke="currentColor"
            stroke-width="1"
          />
        </svg>
      {/if}
    </button>
  </div>
  <div class="title">apto</div>
  <div class="right-controls">
    <button
      class="theme-toggle"
      onclick={cycleTheme}
      aria-label="Toggle Theme"
      title="Toggle Theme ({currentTheme === 'system'
        ? 'System'
        : currentTheme === 'light'
          ? 'Light'
          : 'Dark'})"
    >
      {#if currentTheme === "system"}
        <!-- System Theme Icon -->
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path
            d="M12,2C6.48,2,2,6.48,2,12s4.48,10,10,10s10-4.48,10-10S17.52,2,12,2z M12,20c-4.41,0-8-3.59-8-8s3.59-8,8-8s8,3.59,8,8 S16.41,20,12,20z M12,6c-3.31,0-6,2.69-6,6s2.69,6,6,6s6-2.69,6-6S15.31,6,12,6z M12,16c-2.21,0-4-1.79-4-4s1.79-4,4-4s4,1.79,4,4 S14.21,16,12,16z"
          />
        </svg>
      {:else if currentTheme === "light"}
        <!-- Light Theme Icon -->
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path
            d="M12,7c-2.76,0-5,2.24-5,5s2.24,5,5,5s5-2.24,5-5S14.76,7,12,7L12,7z M2,13h2c0.55,0,1-0.45,1-1s-0.45-1-1-1H2 c-0.55,0-1,0.45-1,1S1.45,13,2,13z M20,13h2c0.55,0,1-0.45,1-1s-0.45-1-1-1h-2c-0.55,0-1,0.45-1,1S19.45,13,20,13z M11,2v2 c0,0.55,0.45,1,1,1s1-0.45,1-1V2c0-0.55-0.45-1-1-1S11,1.45,11,2z M11,20v2c0,0.55,0.45,1,1,1s1-0.45,1-1v-2c0-0.55-0.45-1-1-1 S11,19.45,11,20z M5.99,4.58c-0.39-0.39-1.03-0.39-1.41,0c-0.39,0.39-0.39,1.03,0,1.41l1.06,1.06c0.39,0.39,1.03,0.39,1.41,0 s0.39-1.03,0-1.41L5.99,4.58z M18.36,16.95c-0.39-0.39-1.03-0.39-1.41,0c-0.39,0.39-0.39,1.03,0,1.41l1.06,1.06 c0.39,0.39,1.03,0.39,1.41,0c0.39-0.39,0.39-1.03,0-1.41L18.36,16.95z M19.42,5.99c0.39-0.39,0.39-1.03,0-1.41 c-0.39-0.39-1.03-0.39-1.41,0l-1.06,1.06c-0.39,0.39-0.39,1.03,0,1.41s1.03,0.39,1.41,0L19.42,5.99z M7.05,18.36 c0.39-0.39,0.39-1.03,0-1.41c-0.39-0.39-1.03-0.39-1.41,0l-1.06,1.06c-0.39,0.39-0.39,1.03,0,1.41s1.03,0.39,1.41,0L7.05,18.36z"
          />
        </svg>
      {:else}
        <!-- Dark Theme Icon -->
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path
            d="M12,3c-4.97,0-9,4.03-9,9s4.03,9,9,9s9-4.03,9-9c0-0.46-0.04-0.92-0.1-1.36c-0.98,1.37-2.58,2.26-4.4,2.26 c-2.98,0-5.4-2.42-5.4-5.4c0-1.81,0.89-3.42,2.26-4.4C12.92,3.04,12.46,3,12,3L12,3z"
          />
        </svg>
      {/if}
    </button>

    <button
      class="acrylic-toggle"
      onclick={toggleAcrylic}
      aria-label="Toggle Acrylic Effect"
      title="Toggle Acrylic Effect"
    >
      {#if useAcrylic}
        <!-- Glass Icon (Filled) -->
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path
            d="M9.5,4h5l0.5,9.5l-0.5,0.5h-5l-0.5-0.5L9.5,4z M11,5v8h2V5H11z M5,19v-2h14v2H5z"
          />
        </svg>
      {:else}
        <!-- Glass Icon (Outline) -->
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          stroke="currentColor"
          fill="none"
          stroke-width="1.5"
        >
          <path
            d="M9.5,4h5l0.5,9.5l-0.5,0.5h-5l-0.5-0.5L9.5,4z M11,5v8h2V5H11z M5,19v-2h14v2H5z"
          />
        </svg>
      {/if}
    </button>
  </div>
</div>

<style>
  .titlebar {
    height: 32px;
    background: transparent;
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 0 12px;
    -webkit-user-select: none;
    user-select: none;
    cursor: default;
    position: relative;
    margin-top: 4px;
    font-weight: 900;
    letter-spacing: 1px;
  }

  .title {
    font-size: 13px;
    font-weight: 500;
  }

  .window-controls {
    display: flex;
    gap: 8px;
    position: absolute;
    left: 12px;
  }

  .right-controls {
    display: flex;
    gap: 8px;
    position: absolute;
    right: 12px;
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

  .acrylic-toggle,
  .theme-toggle {
    width: 20px;
    height: 20px;
    background: transparent;
    border-radius: 4px;
    margin-left: 8px;
  }

  .acrylic-toggle svg,
  .theme-toggle svg {
    width: 16px;
    height: 16px;
    opacity: 0.7;
  }

  .acrylic-toggle:hover svg,
  .theme-toggle:hover svg {
    opacity: 1;
  }

  /* icons on hover */
  .window-controls button:hover svg {
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

  .title,
  .acrylic-toggle,
  .theme-toggle {
    color: inherit;
  }

  /* light theme */
  :global(html.light) .title,
  :global(html.light) .acrylic-toggle,
  :global(html.light) .theme-toggle {
    color: #333333;
  }

  /* dark theme */
  :global(html.dark) .title,
  :global(html.dark) .acrylic-toggle,
  :global(html.dark) .theme-toggle {
    color: #f6f6f6;
  }

  /* OS preference if no class */
  @media (prefers-color-scheme: dark) {
    .title,
    .acrylic-toggle,
    .theme-toggle {
      color: #f6f6f6;
    }
  }

  @media (prefers-color-scheme: light) {
    .title,
    .acrylic-toggle,
    .theme-toggle {
      color: #333333;
    }
  }
</style>
