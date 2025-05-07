<script lang="ts">
  import NotesManagement from "$lib/components/notes/NotesManagement.svelte";
  import { goto } from "$app/navigation";

  function goHome() {
    goto("/");
  }
</script>

<main class="notes-container">
  <div class="header">
    <button class="home-button" onclick={goHome}>
      <span class="home-icon">←</span>
    </button>
  </div>

  <NotesManagement />
</main>

<style>
  :global(body) {
    font-family: "Fira Code", Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;
  }
  .notes-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: 10px 20px;
    box-sizing: border-box;
    overflow: auto;
    min-height: 0;
  }
  .header {
    display: flex;
    width: 100%;
    align-items: center;
    justify-content: center;
    position: relative;
    margin-bottom: 1.5rem;
    height: 40px;
  }
  .home-button {
    position: absolute;
    left: 0;
    top: 0;
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
