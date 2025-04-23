<script lang="ts">
  import NotesManagement from "$lib/components/notes/NotesManagement.svelte";
</script>

<main class="notes-container">
  <NotesManagement />
</main>

<style>
  .notes-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
  }
</style>
