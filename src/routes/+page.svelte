<script lang="ts">
  // You can add any page-specific logic here
</script>

<div class="flex h-[calc(100vh-60px)] flex-col items-center justify-center p-4">
  <div class="glass-card max-w-lg rounded-xl p-6 text-center">
    <h1 class="mb-4 text-2xl font-bold">Welcome to Apto</h1>
    <p class="mb-6">
      This is your new translucent application with a glass-like effect. You can
      see through the background while maintaining readability.
    </p>

    <div class="flex justify-center gap-4">
      <button class="glass-button rounded-lg px-4 py-2"> Get Started </button>
      <button class="glass-button-outline rounded-lg px-4 py-2">
        Learn More
      </button>
    </div>
  </div>
</div>

<style>
  /* Additional glass effect styles for cards and buttons */
  .glass-card {
    background: rgba(255, 255, 255, 0.35);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    box-shadow: 0 8px 32px rgba(31, 38, 135, 0.1);
  }

  .glass-button {
    background: rgba(255, 255, 255, 0.2);
    backdrop-filter: blur(5px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    transition: all 0.3s;
  }

  .glass-button:hover {
    background: rgba(255, 255, 255, 0.3);
    transform: translateY(-2px);
  }

  .glass-button-outline {
    background: transparent;
    backdrop-filter: blur(5px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    transition: all 0.3s;
  }

  .glass-button-outline:hover {
    border-color: rgba(255, 255, 255, 0.5);
    transform: translateY(-2px);
  }

  /* Dark mode support for components */
  :global(.dark) .glass-card {
    background: rgba(30, 30, 30, 0.4);
    border-color: rgba(255, 255, 255, 0.05);
  }

  :global(.dark) .glass-button {
    background: rgba(60, 60, 60, 0.3);
  }

  :global(.dark) .glass-button:hover {
    background: rgba(60, 60, 60, 0.5);
  }

  :global(.dark) .glass-button-outline {
    border-color: rgba(255, 255, 255, 0.1);
  }
</style>
