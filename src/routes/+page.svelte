<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  // --- Existing greet state ---
  let name = $state("");
  let greetMsg = $state("");

  // --- New Habit Form State ---
  let habitName = $state("");
  let habitDescription = $state("");
  let habitFrequency = $state("daily"); // Default frequency, maybe change later
  let statusMessage = $state(""); // For success/error feedback
  let isLoading = $state(false); // To disable button during submission

  // --- Existing greet function ---
  async function greet(event: Event) {
    event.preventDefault();
    greetMsg = await invoke("greet", { name });
  }

  // --- Function to handle adding a new habit ---
  async function handleAddHabit(event: Event) {
    event.preventDefault(); // Prevent default form submission
    if (!habitName.trim() || !habitFrequency.trim()) {
      statusMessage = "Habit name and frequency are required.";
      return;
    }

    isLoading = true;
    statusMessage = "Adding habit...";

    try {
      // Prepare payload - keys must match Rust function arguments
      const payload = {
        name: habitName,
        description: habitDescription.trim() || null, // Send null if empty
        frequency: habitFrequency,
      };

      // Invoke the Rust command 'add_habit'
      const newHabitId: number = await invoke("add_habit", payload);

      statusMessage = `Successfully added habit '${habitName}' (ID: ${newHabitId})!`;

      // Clear the form on success
      habitName = "";
      habitDescription = "";
      habitFrequency = "daily"; // Reset to default

      // Optional: Clear status message after a delay
      setTimeout(() => {
        statusMessage = "";
      }, 5000);
    } catch (error) {
      console.error("Error adding habit:", error);
      // Display the error message received from Rust
      statusMessage = `Error: ${error}`;
    } finally {
      isLoading = false; // Re-enable button
    }
  }
</script>

<main class="container">
  <h1>Welcome to Tauri + Svelte</h1>

  <div class="row">
    <a href="https://vitejs.dev" target="_blank" rel="noreferrer">
      <img src="/vite.svg" class="logo vite" alt="Vite Logo" />
    </a>
    <a href="https://tauri.app" target="_blank" rel="noreferrer">
      <img src="/tauri.svg" class="logo tauri" alt="Tauri Logo" />
    </a>
    <a href="https://kit.svelte.dev" target="_blank" rel="noreferrer">
      <img src="/svelte.svg" class="logo svelte-kit" alt="SvelteKit Logo" />
    </a>
  </div>
  <p>Click on the Tauri, Vite, and SvelteKit logos to learn more.</p>

  <div class="glass-card">
    <form class="row" onsubmit={greet}>
      <input id="greet-input" placeholder="Enter a name..." bind:value={name} />
      <button type="submit">Greet</button>
    </form>
    {#if greetMsg}
      <p class="result">{greetMsg}</p>
    {/if}
  </div>

  <div class="glass-card habit-form">
    <h2>Create New Habit</h2>
    <form onsubmit={handleAddHabit}>
      <div class="form-group">
        <label for="habit-name">Name:</label>
        <input
          id="habit-name"
          type="text"
          placeholder="e.g., Drink Water"
          bind:value={habitName}
          required
          disabled={isLoading}
        />
      </div>
      <div class="form-group">
        <label for="habit-description">Description (Optional):</label>
        <textarea
          id="habit-description"
          placeholder="e.g., Drink 8 glasses daily"
          bind:value={habitDescription}
          disabled={isLoading}
        ></textarea>
      </div>
      <div class="form-group">
        <label for="habit-frequency">Frequency:</label>
        <input
          id="habit-frequency"
          type="text"
          placeholder="e.g., daily, weekly:Mon,Fri"
          bind:value={habitFrequency}
          required
          disabled={isLoading}
        />
      </div>
      <button type="submit" disabled={isLoading}>
        {isLoading ? "Adding..." : "Add Habit"}
      </button>
    </form>
    {#if statusMessage}
      <p class="result" class:error={statusMessage.startsWith("Error:")}>
        {statusMessage}
      </p>
    {/if}
  </div>
</main>

<style>
  /* --- Keep all your existing styles --- */

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
    padding: 10px 20px 20px 20px; /* Adjusted padding */
    padding-top: 5vh; /* Keep original top padding */
    display: flex;
    flex-direction: column;
    /* Removed fixed justify-content */
    align-items: center; /* Center items horizontally */
    text-align: center;
    height: 100%;
    box-sizing: border-box; /* Include padding in height calculation */
    overflow-y: auto; /* Allow scrolling if content exceeds height */
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
    width: 90%; /* Ensure cards don't stretch too wide */
    box-sizing: border-box;
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
    word-wrap: break-word; /* Prevent long messages overflowing */
  }
  .result.error {
    color: #ff6b6b; /* Simple error color */
  }
  :global(html.dark) .result.error {
    color: #ff8787; /* Lighter error color for dark mode */
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
    align-items: center; /* Vertically align items in the row */
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
  h2 {
    margin-top: 0;
    margin-bottom: 1em;
    font-weight: 600;
  }

  input,
    textarea, /* Added textarea */
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
    box-sizing: border-box; /* Include padding/border in element width */
  }
  input,
  textarea {
    width: 100%; /* Make inputs take full width */
  }
  textarea {
    min-height: 60px; /* Give textarea some default height */
    resize: vertical; /* Allow vertical resize */
  }

  /* input/button styles */
  :global(html.light) input,
  :global(html.light) textarea,
  :global(html.light) button {
    color: #0f0f0f !important;
    background-color: rgba(255, 255, 255, 0.7) !important;
  }

  :global(html.light) button:hover:not(:disabled) {
    /* Don't change style if disabled */
    background-color: rgba(255, 255, 255, 0.9) !important;
    border-color: #396cd8 !important;
  }

  :global(html.light) button:active:not(:disabled) {
    background-color: rgba(232, 232, 232, 0.8) !important;
  }

  :global(html.dark) input,
  :global(html.dark) textarea,
  :global(html.dark) button {
    color: #ffffff !important;
    background-color: rgba(30, 30, 30, 0.7) !important;
  }

  :global(html.dark) button:hover:not(:disabled) {
    background-color: rgba(30, 30, 30, 0.9) !important;
    border-color: #396cd8 !important;
  }

  :global(html.dark) button:active:not(:disabled) {
    background-color: rgba(20, 20, 20, 0.8) !important;
  }

  button {
    cursor: pointer;
  }
  button:disabled {
    cursor: not-allowed;
    opacity: 0.6;
  }

  input,
  textarea,
  button {
    outline: none;
  }

  #greet-input {
    margin-right: 5px;
    width: auto; /* Override width for this specific input */
  }

  /* Styles for the new habit form */
  .habit-form form {
    display: flex;
    flex-direction: column;
    gap: 15px; /* Spacing between form groups */
    text-align: left; /* Align labels to the left */
  }
  .habit-form .form-group {
    display: flex;
    flex-direction: column;
    gap: 5px; /* Space between label and input */
  }
  .habit-form label {
    font-weight: 500;
    font-size: 0.9em;
  }
  .habit-form button[type="submit"] {
    margin-top: 10px; /* Add space above submit button */
    align-self: center; /* Center button */
    width: auto; /* Allow button to size based on content */
    min-width: 120px;
  }
</style>
