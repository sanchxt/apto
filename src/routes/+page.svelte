<script lang="ts">
  import HabitTracking from "../lib/components/habits/HabitTracking.svelte";
</script>

<main class="container">
  <!-- <h1>Apto</h1> -->

  <HabitTracking />
</main>

<style>
  :root {
    font-family: "Fira Code", Inter, Avenir, Helvetica, Arial, sans-serif;
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
    padding: 10px 20px 20px 20px;
    padding-top: 5vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    height: 100%;
    box-sizing: border-box;
    overflow-y: auto;
  }
</style>
