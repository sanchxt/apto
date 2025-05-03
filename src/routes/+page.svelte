<script lang="ts">
  import { goto } from "$app/navigation";

  function navigateTo(path: string) {
    goto(path);
  }
</script>

<main class="home-container">
  <h1 class="app-title">Apto</h1>

  <div class="cards-container">
    <button class="card habits-card" on:click={() => navigateTo("/habits")}>
      <div class="card-icon">📊</div>
      <h2>Habits</h2>
      <p>Track your habits and build routines</p>
    </button>

    <button class="card notes-card" on:click={() => navigateTo("/notes")}>
      <div class="card-icon">📝</div>
      <h2>Notes</h2>
      <p>Capture and organize your thoughts</p>
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
    padding: 2rem;
  }

  .app-title {
    font-size: 2.5rem;
    margin-bottom: 2rem;
    font-weight: 600;
  }

  .cards-container {
    display: flex;
    flex-wrap: wrap;
    gap: 2rem;
    justify-content: center;
    max-width: 1000px;
  }

  .card {
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 16px;
    padding: 2rem;
    width: 240px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
    transition: all 0.3s ease;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: inherit;
    font-family: inherit;
  }

  .card:hover {
    transform: translateY(-5px);
    box-shadow: 0 10px 20px rgba(0, 0, 0, 0.1);
    background-color: rgba(255, 255, 255, 0.15);
  }

  .card-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
  }

  .card h2 {
    font-size: 1.5rem;
    margin-bottom: 0.5rem;
  }

  .card p {
    opacity: 0.8;
    font-size: 0.9rem;
    line-height: 1.4;
  }

  /* light mode */
  :global(html.light) .card {
    background-color: rgba(255, 255, 255, 0.7);
    border: 1px solid rgba(0, 0, 0, 0.05);
  }

  :global(html.light) .card:hover {
    background-color: rgba(255, 255, 255, 0.9);
  }

  /* dark mode */
  :global(html.dark) .card {
    background-color: rgba(40, 40, 40, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  :global(html.dark) .card:hover {
    background-color: rgba(60, 60, 60, 0.7);
  }
</style>
