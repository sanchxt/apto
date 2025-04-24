<script lang="ts">
  import CreateHabit from "./CreateHabit.svelte";
  import ViewHabits from "./ViewHabits.svelte";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let activeTab = $state("create");

  // shared data between components
  let habits = $state<any[]>([]);
  let allTags = $state<{ id: number; name: string; color: string }[]>([]);

  // load data on component mount
  onMount(async () => {
    await Promise.all([loadTags(), loadHabits()]);
  });

  // load habits from backend
  async function loadHabits() {
    try {
      habits = await invoke("get_habits");
      console.log("Loaded habits:", habits);
    } catch (error) {
      console.error("Failed to load habits:", error);
    }
  }

  // load tags
  async function loadTags() {
    try {
      allTags = await invoke("get_all_tags");
      console.log("Loaded tags:", allTags);
    } catch (error) {
      console.error("Failed to load tags:", error);
    }
  }
</script>

<div class="habits-container">
  <!-- tabs -->
  <div class="tabs">
    <button
      class="tab-button"
      class:active={activeTab === "create"}
      onclick={() => (activeTab = "create")}
    >
      Create Habit
    </button>
    <button
      class="tab-button"
      class:active={activeTab === "view"}
      onclick={() => (activeTab = "view")}
    >
      My Habits
    </button>
  </div>

  <!-- render component based on active tab -->
  {#if activeTab === "create"}
    <CreateHabit
      {allTags}
      onHabitCreated={loadHabits}
      onTagCreated={loadTags}
    />
  {:else}
    <ViewHabits {habits} {allTags} onHabitUpdated={loadHabits} />
  {/if}
</div>

<style>
  /* main container */
  .habits-container {
    width: 100%;
    max-width: 1200px;
    margin: 0 auto;
    padding: 0;
  }

  /* tabs */
  .tabs {
    display: flex;
    margin-bottom: 24px;
    border-bottom: 1px solid rgba(128, 128, 128, 0.15);
  }

  .tab-button {
    padding: 12px 24px;
    background: transparent;
    border: none;
    font-size: 15px;
    font-weight: 500;
    cursor: pointer;
    color: inherit;
    opacity: 0.6;
    transition: opacity 0.2s;
  }

  .tab-button:hover {
    opacity: 0.9;
  }

  .tab-button.active {
    opacity: 1;
    box-shadow: inset 0 -2px 0 currentColor;
  }
</style>
