<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  // Props
  export let habits: any[] = [];
  export let allTags: { id: number; name: string; color: string }[] = [];
  export let onHabitUpdated: () => Promise<void>;

  // Days of week for display
  const daysOfWeek = [
    { id: 1, name: "Monday" },
    { id: 2, name: "Tuesday" },
    { id: 3, name: "Wednesday" },
    { id: 4, name: "Thursday" },
    { id: 5, name: "Friday" },
    { id: 6, name: "Saturday" },
    { id: 7, name: "Sunday" },
  ];

  // toggle habit active state
  async function toggleHabitActive(id: number, currentState: boolean) {
    try {
      await invoke("toggle_habit_active", {
        id,
        isActive: !currentState,
      });

      // reload habits
      await onHabitUpdated();
    } catch (error) {
      console.error(`Failed to toggle habit ${id}:`, error);
    }
  }

  // formatted frequency text for display
  function getFrequencyDisplay(frequency: any): string {
    if (!frequency) return "Daily";

    if (frequency.Daily !== undefined) {
      return "Daily";
    } else if (frequency.Weekly) {
      const days = frequency.Weekly.days || [];
      if (days.length === 7) return "Every day";

      const dayNames = days
        .map((day: any) => {
          const dayObj = daysOfWeek.find((d) => d.id === day);
          return dayObj ? dayObj.name.substring(0, 3) : day;
        })
        .join(", ");

      return `Weekly: ${dayNames}`;
    } else if (frequency.Monthly) {
      const days = frequency.Monthly.days || [];
      return `Monthly on day${days.length > 1 ? "s" : ""}: ${days.join(", ")}`;
    } else if (frequency.Interval) {
      const interval = frequency.Interval.days || 1;
      return `Every ${interval} day${interval > 1 ? "s" : ""}`;
    } else if (frequency.Custom) {
      return `Custom: ${frequency.Custom.pattern}`;
    }

    return "Unknown";
  }

  // format date for display
  function formatDate(dateStr: string): string {
    if (!dateStr) return "";

    const date = new Date(dateStr);
    return date.toLocaleDateString();
  }
</script>

<div class="habits-list">
  <h2>My Habits</h2>

  {#if habits.length === 0}
    <div class="empty-message big">
      You don't have any habits yet. Create your first habit to start tracking!
    </div>
  {:else}
    <div class="habits-grid">
      {#each habits as habit}
        <div
          class="habit-card"
          style="border-left-color: {habit.color || '#3b82f6'}"
        >
          <div class="habit-header">
            <div class="habit-title">
              <h3>{habit.name}</h3>
              <div class="habit-badges">
                {#if habit.priority === 1}
                  <span class="badge high">High</span>
                {:else if habit.priority === 2}
                  <span class="badge medium">Medium</span>
                {:else}
                  <span class="badge low">Low</span>
                {/if}

                <span class="badge status" class:inactive={!habit.is_active}>
                  {habit.is_active ? "Active" : "Inactive"}
                </span>
              </div>
            </div>

            <div class="toggle-switch">
              <input
                type="checkbox"
                id={`toggle-${habit.id}`}
                checked={habit.is_active}
                onchange={() => toggleHabitActive(habit.id, habit.is_active)}
              />
              <label for={`toggle-${habit.id}`}></label>
            </div>
          </div>

          {#if habit.description}
            <p class="habit-description">{habit.description}</p>
          {/if}

          <div class="habit-details">
            <div class="detail-item">
              <span class="detail-label">Frequency:</span>
              <span class="detail-value"
                >{getFrequencyDisplay(habit.frequency)}</span
              >
            </div>

            {#if habit.category}
              <div class="detail-item">
                <span class="detail-label">Category:</span>
                <span class="detail-value">{habit.category}</span>
              </div>
            {/if}

            {#if habit.target_value !== null && habit.target_value !== undefined}
              <div class="detail-item">
                <span class="detail-label">Target:</span>
                <span class="detail-value"
                  >{habit.target_value} {habit.target_unit || ""}</span
                >
              </div>
            {/if}

            <div class="detail-item">
              <span class="detail-label">Started:</span>
              <span class="detail-value">{formatDate(habit.start_date)}</span>
            </div>

            {#if habit.current_streak > 0}
              <div class="detail-item">
                <span class="detail-label">Current Streak:</span>
                <span class="detail-value"
                  >{habit.current_streak} day{habit.current_streak !== 1
                    ? "s"
                    : ""}</span
                >
              </div>
            {/if}
          </div>

          {#if habit.tags && habit.tags.length > 0}
            <div class="habit-tags">
              {#each habit.tags as tag}
                <span
                  class="tag-pill"
                  style="background-color: {allTags.find((t) => t.name === tag)
                    ?.color || '#3b82f6'}"
                >
                  {tag}
                </span>
              {/each}
            </div>
          {/if}

          <div class="habit-actions">
            <button class="action-button">
              <span class="icon">✓</span>
              Complete
            </button>
            <button class="action-button">
              <span class="icon">✎</span>
              Edit
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .empty-message.big {
    padding: 32px;
    font-size: 16px;
  }

  /* Habits List */
  .habits-list {
    width: 100%;
  }

  .habits-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: 20px;
  }

  .habit-card {
    border-radius: 6px;
    border-left: 4px solid #3b82f6;
    background-color: rgba(255, 255, 255, 0.5);
    padding: 16px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
    transition: all 0.2s;
  }

  .habit-card:hover {
    box-shadow: 0 3px 8px rgba(0, 0, 0, 0.08);
    transform: translateY(-2px);
  }

  .habit-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 12px;
  }

  .habit-title {
    flex: 1;
  }

  .habit-title h3 {
    margin: 0 0 8px 0;
    font-size: 16px;
    font-weight: 600;
  }

  .habit-badges {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .badge {
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
  }

  .badge.high {
    background-color: rgba(239, 68, 68, 0.1);
    color: #ef4444;
  }

  .badge.medium {
    background-color: rgba(245, 158, 11, 0.1);
    color: #f59e0b;
  }

  .badge.low {
    background-color: rgba(16, 185, 129, 0.1);
    color: #10b981;
  }

  .badge.status {
    background-color: rgba(59, 130, 246, 0.1);
    color: #3b82f6;
  }

  .badge.status.inactive {
    background-color: rgba(107, 114, 128, 0.1);
    color: #6b7280;
  }

  .habit-description {
    margin: 0 0 12px 0;
    font-size: 14px;
    color: rgba(0, 0, 0, 0.6);
    line-height: 1.5;
  }

  .habit-details {
    margin-bottom: 12px;
    font-size: 13px;
  }

  .detail-item {
    display: flex;
    justify-content: space-between;
    margin-bottom: 4px;
  }

  .detail-label {
    color: rgba(0, 0, 0, 0.6);
  }

  .habit-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 12px;
  }

  .tag-pill {
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 12px;
    color: white;
  }

  .habit-actions {
    display: flex;
    gap: 8px;
  }

  .action-button {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 8px;
    background-color: transparent;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
    color: inherit;
  }

  .action-button:hover {
    background-color: rgba(0, 0, 0, 0.03);
  }

  /* Toggle Switch */
  .toggle-switch {
    position: relative;
    width: 36px;
    height: 20px;
  }

  .toggle-switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .toggle-switch label {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.2);
    transition: 0.4s;
    border-radius: 20px;
  }

  .toggle-switch label:before {
    position: absolute;
    content: "";
    height: 16px;
    width: 16px;
    left: 2px;
    bottom: 2px;
    background-color: white;
    transition: 0.4s;
    border-radius: 50%;
  }

  .toggle-switch input:checked + label {
    background-color: #3b82f6;
  }

  .toggle-switch input:checked + label:before {
    transform: translateX(16px);
  }

  :global(html.dark) .action-button {
    border-color: rgba(255, 255, 255, 0.1);
  }

  :global(html.dark) .habit-card {
    background-color: rgba(50, 50, 50, 0.3);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  }

  :global(html.dark) .habit-card:hover {
    box-shadow: 0 3px 8px rgba(0, 0, 0, 0.3);
  }

  :global(html.dark) .habit-description {
    color: rgba(255, 255, 255, 0.6);
  }

  :global(html.dark) .detail-label {
    color: rgba(255, 255, 255, 0.6);
  }

  :global(html.dark) .toggle-switch label {
    background-color: rgba(255, 255, 255, 0.2);
  }

  @media (max-width: 600px) {
    .habits-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
