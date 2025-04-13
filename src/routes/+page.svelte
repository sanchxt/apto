<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let name = $state("");
  let greetMsg = $state("");

  async function greet(event: Event) {
    event.preventDefault();
    greetMsg = await invoke("greet", { name });
  }
</script>

<main class="container">
  <h1>Welcome to Tauri + Svelte</h1>

  <div class="row">
    <a href="https://vitejs.dev" target="_blank">
      <img src="/vite.svg" class="logo vite" alt="Vite Logo" />
    </a>
    <a href="https://tauri.app" target="_blank">
      <img src="/tauri.svg" class="logo tauri" alt="Tauri Logo" />
    </a>
    <a href="https://kit.svelte.dev" target="_blank">
      <img src="/svelte.svg" class="logo svelte-kit" alt="SvelteKit Logo" />
    </a>
  </div>
  <p>Click on the Tauri, Vite, and SvelteKit logos to learn more.</p>

  <div class="glass-card">
    <form class="row" onsubmit={greet}>
      <input id="greet-input" placeholder="Enter a name..." bind:value={name} />
      <button type="submit">Greet</button>
    </form>
    <p class="result">{greetMsg}</p>
  </div>
</main>

<style>
  .logo.vite:hover {
    filter: drop-shadow(0 0 2em #747bff);
  }

  .logo.svelte-kit:hover {
    filter: drop-shadow(0 0 2em #ff3e00);
  }

  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;
    color: inherit;
    background-color: transparent;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
  }

  .container {
    margin: 0;
    padding-top: 5vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    text-align: center;
    height: 100%;
    overflow-y: auto;
  }

  .glass-card {
    background: rgba(255, 255, 255, 0.15);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    padding: 20px;
    margin: 20px auto;
    max-width: 500px;
  }

  /* Theme-specific glass card styles */
  :global(html.light) .glass-card {
    background: rgba(255, 255, 255, 0.15) !important;
    border: 1px solid rgba(255, 255, 255, 0.12) !important;
  }

  :global(html.dark) .glass-card {
    background: rgba(50, 50, 50, 0.25) !important;
    border: 1px solid rgba(100, 100, 100, 0.15) !important;
  }

  .result {
    margin-top: 12px;
    font-weight: 500;
  }

  .logo {
    height: 6em;
    padding: 1.5em;
    will-change: filter;
    transition: 0.75s;
  }

  .logo.tauri:hover {
    filter: drop-shadow(0 0 2em #24c8db);
  }

  .row {
    display: flex;
    justify-content: center;
  }

  a {
    font-weight: 500;
    color: #646cff;
    text-decoration: inherit;
  }

  a:hover {
    color: #535bf2;
  }

  h1 {
    text-align: center;
  }

  input,
  button {
    border-radius: 8px;
    border: 1px solid transparent;
    padding: 0.6em 1.2em;
    font-size: 1em;
    font-weight: 500;
    font-family: inherit;
    transition:
      border-color 0.25s,
      background-color 0.25s;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.1);
  }

  /* input/button styles */
  :global(html.light) input,
  :global(html.light) button {
    color: #0f0f0f !important;
    background-color: rgba(255, 255, 255, 0.7) !important;
  }

  :global(html.light) button:hover {
    background-color: rgba(255, 255, 255, 0.9) !important;
    border-color: #396cd8 !important;
  }

  :global(html.light) button:active {
    background-color: rgba(232, 232, 232, 0.8) !important;
  }

  :global(html.dark) input,
  :global(html.dark) button {
    color: #ffffff !important;
    background-color: rgba(30, 30, 30, 0.7) !important;
  }

  :global(html.dark) button:hover {
    background-color: rgba(30, 30, 30, 0.9) !important;
    border-color: #396cd8 !important;
  }

  :global(html.dark) button:active {
    background-color: rgba(20, 20, 20, 0.8) !important;
  }

  button {
    cursor: pointer;
  }

  input,
  button {
    outline: none;
  }

  #greet-input {
    margin-right: 5px;
  }
</style>
