<script lang="ts">
  import HabitTracking from "$lib/components/habits/HabitTracking.svelte";
  import { goto } from "$app/navigation";

  function goHome() {
    goto("/");
  }
</script>

<main class="container">
  <div class="header">
    <button class="home-button" onclick={goHome}>
      <span class="home-icon">←</span>
    </button>
    <h1>Habits</h1>
  </div>

  <HabitTracking />
</main>

<style>
  :global(body) {
    font-family: "Fira Code", Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;
  }

  .container {
    margin: 0;
    padding: 10px 20px 20px 20px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    height: 100%;
    box-sizing: border-box;
    overflow-y: auto;
    min-height: 0;
  }

  .header {
    display: flex;
    width: 100%;
    align-items: center;
    justify-content: center;
    position: relative;
    margin-bottom: 2rem;
  }

  h1 {
    margin: 0;
  }

  .home-button {
    position: absolute;
    left: 0;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border-radius: 8px;
    border: none;
    background-color: rgba(128, 128, 128, 0.1);
    color: inherit;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .home-button:hover {
    background-color: rgba(128, 128, 128, 0.2);
  }

  .home-icon {
    font-size: 1.2rem;
  }
</style>
