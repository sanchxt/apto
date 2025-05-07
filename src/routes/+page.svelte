<script lang="ts">
  import { goto } from "$app/navigation";

  function navigateTo(path: string) {
    goto(path);
  }
</script>

<main class="home-container">
  <div class="menu-list">
    <button class="menu-item" on:click={() => navigateTo("/habits")}>
      <div class="menu-icon">
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M12 20V10"></path>
          <path d="M18 20V4"></path>
          <path d="M6 20v-4"></path>
        </svg>
      </div>
      <div class="menu-content">
        <h2>Habits</h2>
        <p>Track your habits and build routines</p>
      </div>
      <div class="menu-arrow">
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M9 18l6-6-6-6"></path>
        </svg>
      </div>
    </button>

    <button class="menu-item" on:click={() => navigateTo("/notes")}>
      <div class="menu-icon">
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"
          ></path>
          <polyline points="14 2 14 8 20 8"></polyline>
          <line x1="16" y1="13" x2="8" y2="13"></line>
          <line x1="16" y1="17" x2="8" y2="17"></line>
          <polyline points="10 9 9 9 8 9"></polyline>
        </svg>
      </div>
      <div class="menu-content">
        <h2>Notes</h2>
        <p>Capture and organize your thoughts</p>
      </div>
      <div class="menu-arrow">
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M9 18l6-6-6-6"></path>
        </svg>
      </div>
    </button>
  </div>
</main>

<style>
  :global(body) {
    font-family: "Fira Code", Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;
  }

  .home-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 1.5rem;
    max-width: 800px;
    margin: 0 auto;
  }

  .menu-list {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .menu-item {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 1.25rem 1.5rem;
    border-radius: 8px;
    background-color: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    transition: all 0.2s ease;
    cursor: pointer;
    font-family: inherit;
    text-align: left;
    color: inherit;
  }

  .menu-item:hover {
    background-color: rgba(255, 255, 255, 0.07);
    transform: translateY(-1px);
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.05);
  }

  .menu-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 40px;
    opacity: 0.8;
  }

  .menu-content {
    flex: 1;
    margin: 0 1.25rem;
  }

  .menu-content h2 {
    font-size: 1rem;
    font-weight: 500;
    margin: 0 0 0.25rem 0;
    letter-spacing: 0.01em;
  }

  .menu-content p {
    font-size: 0.85rem;
    margin: 0;
    opacity: 0.7;
    font-weight: 400;
  }

  .menu-arrow {
    opacity: 0.4;
    transition:
      opacity 0.2s ease,
      transform 0.2s ease;
  }

  .menu-item:hover .menu-arrow {
    opacity: 0.7;
    transform: translateX(2px);
  }

  /* light mode */
  :global(html.light) .menu-item {
    background-color: rgba(250, 250, 250, 0.7);
    border: 1px solid rgba(0, 0, 0, 0.03);
  }

  :global(html.light) .menu-item:hover {
    background-color: rgba(255, 255, 255, 0.95);
    border: 1px solid rgba(0, 0, 0, 0.06);
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.03);
  }

  /* dark mode */
  :global(html.dark) .menu-item {
    background-color: rgba(40, 40, 40, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.03);
  }

  :global(html.dark) .menu-item:hover {
    background-color: rgba(50, 50, 50, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.06);
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.2);
  }
</style>
