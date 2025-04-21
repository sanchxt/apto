<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  // form state
  let habitName = $state("");
  let habitDescription = $state("");
  let habitCategory = $state("");
  let habitTags = $state<string[]>([]);
  let habitFrequencyType = $state("daily");
  let habitFrequencyDays = $state<number[]>([]);
  let habitFrequencyInterval = $state(1);
  let habitFrequencyCustomPattern = $state("");
  let habitTargetValue = $state<number | null>(null);
  let habitTargetUnit = $state("");
  let habitColor = $state("#3b82f6"); // Default blue color
  let habitIcon = $state(""); // Could be an icon name or identifier
  let habitPriority = $state(2); // Default medium priority
  let habitStartDate = $state(new Date().toISOString().split("T")[0]); // Today's date in YYYY-MM-DD
  let habitEndDate = $state(""); // Optional end date
  let habitReminderTime = $state(""); // Format: HH:MM

  // state for existing habits
  let habits = $state<any[]>([]);

  // state for all available tags
  let allTags = $state<{ id: number; name: string; color: string }[]>([]);
  let newTagName = $state("");
  let newTagColor = $state("#3b82f6");

  // UI state
  let isLoading = $state(false);
  let statusMessage = $state("");
  let isSuccess = $state(false);
  let showAdvanced = $state(false);
  let showTagForm = $state(false);
  let activeTab = $state("create");

  // days of week for weekly frequency
  const daysOfWeek = [
    { id: 1, name: "Monday" },
    { id: 2, name: "Tuesday" },
    { id: 3, name: "Wednesday" },
    { id: 4, name: "Thursday" },
    { id: 5, name: "Friday" },
    { id: 6, name: "Saturday" },
    { id: 7, name: "Sunday" },
  ];

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

  // toggle tag selection
  function toggleTag(tagName: string) {
    if (habitTags.includes(tagName)) {
      habitTags = habitTags.filter((tag) => tag !== tagName);
    } else {
      habitTags = [...habitTags, tagName];
    }
  }

  // create a new tag
  async function createTag() {
    if (!newTagName.trim()) {
      statusMessage = "Tag name is required";
      isSuccess = false;
      return;
    }

    try {
      isLoading = true;

      await invoke("create_tag", {
        name: newTagName,
        color: newTagColor,
      });

      // reset form and reload tags
      newTagName = "";
      showTagForm = false;
      await loadTags();
    } catch (error) {
      statusMessage = `Error creating tag: ${error}`;
      isSuccess = false;
      console.error("Failed to create tag:", error);
    } finally {
      isLoading = false;
    }
  }

  // toggle days for weekly/monthly selection
  function toggleDay(day: number) {
    if (habitFrequencyDays.includes(day)) {
      habitFrequencyDays = habitFrequencyDays.filter((d) => d !== day);
    } else {
      habitFrequencyDays = [...habitFrequencyDays, day].sort((a, b) => a - b);
    }
  }

  // prepare frequency data based on selected type
  function getFrequencyData() {
    switch (habitFrequencyType) {
      case "daily":
        return { Daily: null };
      case "weekly":
        return {
          Weekly: {
            days:
              habitFrequencyDays.length > 0
                ? habitFrequencyDays
                : [1, 2, 3, 4, 5],
          },
        };
      case "monthly":
        return {
          Monthly: {
            days: habitFrequencyDays,
          },
        };
      case "interval":
        return {
          Interval: {
            days: habitFrequencyInterval || 1,
          },
        };
      case "custom":
        return {
          Custom: {
            pattern: habitFrequencyCustomPattern,
          },
        };
      default:
        return { Daily: null };
    }
  }

  // submit the form to create a new habit
  async function handleAddHabit(event: Event) {
    event.preventDefault();

    if (!habitName.trim()) {
      statusMessage = "Habit name is required";
      isSuccess = false;
      return;
    }

    if (habitFrequencyType === "weekly" && habitFrequencyDays.length === 0) {
      statusMessage = "Please select at least one day for weekly frequency";
      isSuccess = false;
      return;
    }

    if (habitFrequencyType === "monthly" && habitFrequencyDays.length === 0) {
      statusMessage = "Please select at least one day for monthly frequency";
      isSuccess = false;
      return;
    }

    try {
      isLoading = true;
      statusMessage = "Creating habit...";

      const frequencyData = getFrequencyData();

      // format the dates
      const formattedStartDate = habitStartDate;
      const formattedEndDate = habitEndDate || null;

      // prepare payload
      const payload = {
        name: habitName,
        description: habitDescription.trim() || null,
        category: habitCategory.trim() || null,
        tags: habitTags,
        frequency: frequencyData,
        targetValue: habitTargetValue,
        targetUnit: habitTargetUnit.trim() || null,
        color: habitColor || null,
        icon: habitIcon.trim() || null,
        isActive: true,
        priority: habitPriority,
        startDate: formattedStartDate,
        endDate: formattedEndDate,
        reminderTime: habitReminderTime || null,
      };

      console.log("Sending payload:", payload);

      // call rust command to add a habit
      const newHabitId = await invoke("add_habit", payload);

      statusMessage = `Successfully created habit '${habitName}'!`;
      isSuccess = true;

      // reload habits list
      await loadHabits();

      // reset the form after successful creation
      setTimeout(() => {
        resetForm();
      }, 1500);
    } catch (error) {
      statusMessage = `Error: ${error}`;
      isSuccess = false;
      console.error("Error adding habit:", error);
    } finally {
      isLoading = false;
    }
  }

  // reset form
  function resetForm() {
    habitName = "";
    habitDescription = "";
    habitCategory = "";
    habitTags = [];
    habitFrequencyType = "daily";
    habitFrequencyDays = [];
    habitFrequencyInterval = 1;
    habitFrequencyCustomPattern = "";
    habitTargetValue = null;
    habitTargetUnit = "";
    habitColor = "#3b82f6";
    habitIcon = "";
    habitPriority = 2;
    habitStartDate = new Date().toISOString().split("T")[0];
    habitEndDate = "";
    habitReminderTime = "";
    showAdvanced = false;
    statusMessage = "";
  }

  // toggle habit active state
  async function toggleHabitActive(id: number, currentState: boolean) {
    try {
      await invoke("toggle_habit_active", {
        id,
        isActive: !currentState,
      });

      // reload habits
      await loadHabits();
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

<div class="habits-container">
  <!-- Tabs -->
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

  <!-- Create Habit Tab -->
  {#if activeTab === "create"}
    <div class="habit-form glass-card">
      <h2>Create New Habit</h2>

      <form onsubmit={handleAddHabit}>
        <!-- Basic Information Section -->
        <div class="form-section">
          <h3>Basic Information</h3>

          <div class="form-group">
            <label for="habit-name">Name <span class="required">*</span></label>
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
            <label for="habit-description">Description</label>
            <textarea
              id="habit-description"
              placeholder="e.g., Drink 8 glasses of water daily"
              bind:value={habitDescription}
              disabled={isLoading}
            ></textarea>
          </div>

          <div class="form-row">
            <div class="form-group">
              <label for="habit-category">Category</label>
              <input
                id="habit-category"
                type="text"
                placeholder="e.g., Health"
                bind:value={habitCategory}
                disabled={isLoading}
              />
            </div>

            <div class="form-group">
              <label for="habit-color">Color</label>
              <div class="color-picker">
                <input
                  id="habit-color"
                  type="color"
                  bind:value={habitColor}
                  disabled={isLoading}
                />
                <span class="color-value">{habitColor}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Tags Section -->
        <div class="form-section">
          <div class="section-header">
            <h3>Tags</h3>
            <button
              type="button"
              class="small-button"
              onclick={() => (showTagForm = !showTagForm)}
              disabled={isLoading}
            >
              {showTagForm ? "Cancel" : "Add New Tag"}
            </button>
          </div>

          {#if showTagForm}
            <div class="tag-form">
              <div class="form-row">
                <div class="form-group">
                  <input
                    type="text"
                    placeholder="Tag name"
                    bind:value={newTagName}
                    disabled={isLoading}
                  />
                </div>
                <div class="form-group">
                  <div class="color-picker">
                    <input
                      type="color"
                      bind:value={newTagColor}
                      disabled={isLoading}
                    />
                  </div>
                </div>
                <button
                  type="button"
                  class="small-button"
                  onclick={createTag}
                  disabled={isLoading || !newTagName.trim()}
                >
                  Create
                </button>
              </div>
            </div>
          {/if}

          <div class="tags-container">
            {#if allTags.length > 0}
              {#each allTags as tag}
                <button
                  type="button"
                  class="tag-chip"
                  class:selected={habitTags.includes(tag.name)}
                  style="--tag-color: {tag.color || habitColor}"
                  onclick={() => toggleTag(tag.name)}
                  disabled={isLoading}
                >
                  {tag.name}
                </button>
              {/each}
            {:else}
              <div class="empty-state">
                No tags available. Create some tags to organize your habits.
              </div>
            {/if}
          </div>
        </div>

        <!-- Frequency Section -->
        <div class="form-section">
          <h3>Frequency <span class="required">*</span></h3>

          <div class="frequency-options">
            <div class="radio-group">
              <label>
                <input
                  type="radio"
                  bind:group={habitFrequencyType}
                  value="daily"
                  disabled={isLoading}
                />
                Daily
              </label>

              <label>
                <input
                  type="radio"
                  bind:group={habitFrequencyType}
                  value="weekly"
                  disabled={isLoading}
                />
                Weekly
              </label>

              <label>
                <input
                  type="radio"
                  bind:group={habitFrequencyType}
                  value="monthly"
                  disabled={isLoading}
                />
                Monthly
              </label>

              <label>
                <input
                  type="radio"
                  bind:group={habitFrequencyType}
                  value="interval"
                  disabled={isLoading}
                />
                Every X Days
              </label>

              <label>
                <input
                  type="radio"
                  bind:group={habitFrequencyType}
                  value="custom"
                  disabled={isLoading}
                />
                Custom
              </label>
            </div>

            {#if habitFrequencyType === "weekly"}
              <div class="days-selection">
                <p class="selection-label" id="weekly-days-label">
                  Select days:
                </p>
                <div
                  class="days-container"
                  aria-labelledby="weekly-days-label"
                  role="group"
                >
                  {#each daysOfWeek as day}
                    <button
                      type="button"
                      class="day-button"
                      class:selected={habitFrequencyDays.includes(day.id)}
                      onclick={() => toggleDay(day.id)}
                      disabled={isLoading}
                      aria-pressed={habitFrequencyDays.includes(day.id)}
                    >
                      {day.name.substring(0, 3)}
                    </button>
                  {/each}
                </div>
              </div>
            {:else if habitFrequencyType === "monthly"}
              <div class="days-selection">
                <p class="selection-label" id="monthly-days-label">
                  Select days of month:
                </p>
                <div
                  class="month-days-container"
                  aria-labelledby="monthly-days-label"
                  role="group"
                >
                  {#each Array(31) as _, i}
                    <button
                      type="button"
                      class="day-button day-of-month"
                      class:selected={habitFrequencyDays.includes(i + 1)}
                      onclick={() => toggleDay(i + 1)}
                      disabled={isLoading}
                      aria-pressed={habitFrequencyDays.includes(i + 1)}
                    >
                      {i + 1}
                    </button>
                  {/each}
                </div>
              </div>
            {:else if habitFrequencyType === "interval"}
              <div class="form-group">
                <label for="interval-days">Every</label>
                <div class="input-with-suffix">
                  <input
                    id="interval-days"
                    type="number"
                    min="1"
                    placeholder="1"
                    bind:value={habitFrequencyInterval}
                    disabled={isLoading}
                  />
                  <span class="suffix">days</span>
                </div>
              </div>
            {:else if habitFrequencyType === "custom"}
              <div class="form-group">
                <label for="custom-pattern">Custom Pattern</label>
                <input
                  id="custom-pattern"
                  type="text"
                  placeholder="Describe your pattern"
                  bind:value={habitFrequencyCustomPattern}
                  disabled={isLoading}
                />
                <small>Describe your custom frequency pattern</small>
              </div>
            {/if}
          </div>
        </div>

        <!-- Advanced Options Toggle -->
        <div class="form-section toggle-section">
          <button
            type="button"
            class="toggle-button"
            onclick={() => (showAdvanced = !showAdvanced)}
            disabled={isLoading}
          >
            {showAdvanced ? "Hide Advanced Options" : "Show Advanced Options"}
          </button>
        </div>

        {#if showAdvanced}
          <!-- Advanced Options Section -->
          <div class="form-section">
            <h3>Advanced Options</h3>

            <div class="form-row">
              <div class="form-group">
                <label for="habit-target-value">Target Value</label>
                <input
                  id="habit-target-value"
                  type="number"
                  placeholder="e.g., 8"
                  bind:value={habitTargetValue}
                  disabled={isLoading}
                  min="0"
                  step="any"
                />
              </div>

              <div class="form-group">
                <label for="habit-target-unit">Unit</label>
                <input
                  id="habit-target-unit"
                  type="text"
                  placeholder="e.g., glasses"
                  bind:value={habitTargetUnit}
                  disabled={isLoading}
                />
              </div>
            </div>

            <div class="form-row">
              <div class="form-group">
                <label for="habit-icon">Icon</label>
                <input
                  id="habit-icon"
                  type="text"
                  placeholder="e.g., water_drop"
                  bind:value={habitIcon}
                  disabled={isLoading}
                />
                <small>Enter an icon name (future feature)</small>
              </div>

              <div class="form-group">
                <label for="habit-priority">Priority</label>
                <select
                  id="habit-priority"
                  bind:value={habitPriority}
                  disabled={isLoading}
                >
                  <option value={1}>High</option>
                  <option value={2}>Medium</option>
                  <option value={3}>Low</option>
                </select>
              </div>
            </div>

            <div class="form-row">
              <div class="form-group">
                <label for="habit-start-date">Start Date</label>
                <input
                  id="habit-start-date"
                  type="date"
                  bind:value={habitStartDate}
                  disabled={isLoading}
                />
              </div>

              <div class="form-group">
                <label for="habit-end-date">End Date (Optional)</label>
                <input
                  id="habit-end-date"
                  type="date"
                  bind:value={habitEndDate}
                  disabled={isLoading}
                  min={habitStartDate}
                />
              </div>
            </div>

            <div class="form-group">
              <label for="habit-reminder">Reminder Time (Optional)</label>
              <input
                id="habit-reminder"
                type="time"
                bind:value={habitReminderTime}
                disabled={isLoading}
              />
            </div>
          </div>
        {/if}

        <!-- Submission Section -->
        <div class="form-section submission-section">
          {#if statusMessage}
            <div
              class="status-message"
              class:success={isSuccess}
              class:error={!isSuccess}
            >
              {statusMessage}
            </div>
          {/if}

          <div class="button-row">
            <button
              type="button"
              class="secondary-button"
              onclick={resetForm}
              disabled={isLoading}
            >
              Reset
            </button>
            <button type="submit" class="primary-button" disabled={isLoading}>
              {isLoading ? "Creating..." : "Create Habit"}
            </button>
          </div>
        </div>
      </form>
    </div>
  {:else}
    <!-- View Habits Tab -->
    <div class="habits-list glass-card">
      <h2>My Habits</h2>

      {#if habits.length === 0}
        <div class="empty-state large">
          You don't have any habits yet. Create your first habit to start
          tracking!
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

                    <span
                      class="badge status"
                      class:inactive={!habit.is_active}
                    >
                      {habit.is_active ? "Active" : "Inactive"}
                    </span>
                  </div>
                </div>

                <div class="toggle-switch">
                  <input
                    type="checkbox"
                    id={`toggle-${habit.id}`}
                    checked={habit.is_active}
                    onchange={() =>
                      toggleHabitActive(habit.id, habit.is_active)}
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
                  <span class="detail-value"
                    >{formatDate(habit.start_date)}</span
                  >
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
                      style="background-color: {allTags.find(
                        (t) => t.name === tag
                      )?.color || '#3b82f6'}"
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
  {/if}
</div>

<style>
  .habits-container {
    max-width: 900px;
    margin: 0 auto;
    padding: 20px 0;
  }

  .tabs {
    display: flex;
    justify-content: center;
    margin-bottom: 20px;
  }

  .tab-button {
    padding: 10px 20px;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    color: inherit;
    opacity: 0.7;
  }

  .tab-button.active {
    border-bottom: 2px solid #3b82f6;
    opacity: 1;
  }

  .glass-card {
    padding: 24px;
    margin-bottom: 24px;
  }

  h2 {
    margin-top: 0;
    margin-bottom: 24px;
    font-weight: 600;
    text-align: center;
  }

  h3 {
    margin-top: 0;
    margin-bottom: 16px;
    font-size: 1.1em;
    font-weight: 600;
  }

  .form-section {
    margin-bottom: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid rgba(128, 128, 128, 0.2);
  }

  .form-section:last-child,
  .toggle-section {
    border-bottom: none;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .form-group {
    margin-bottom: 16px;
    width: 100%;
  }

  .form-row {
    display: flex;
    gap: 16px;
    width: 100%;
  }

  @media (max-width: 600px) {
    .form-row {
      flex-direction: column;
      gap: 8px;
    }
  }

  label {
    display: block;
    margin-bottom: 8px;
    font-weight: 500;
    font-size: 0.9em;
  }

  input,
  textarea,
  select {
    width: 100%;
    padding: 10px 12px;
    border-radius: 8px;
    font-size: 0.95em;
    box-sizing: border-box;
  }

  .required {
    color: #e11d48;
    margin-left: 2px;
  }

  .color-picker {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .color-picker input[type="color"] {
    width: 36px;
    height: 36px;
    padding: 0;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .color-value {
    font-size: 0.8em;
    opacity: 0.7;
  }

  .tags-container {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 16px;
  }

  .tag-chip {
    padding: 6px 12px;
    border-radius: 16px;
    font-size: 0.85em;
    background-color: rgba(var(--tag-color), 0.1);
    border: 1px solid var(--tag-color);
    color: inherit;
    cursor: pointer;
    transition: all 0.2s;
  }

  .tag-chip.selected {
    background-color: var(--tag-color);
    color: white;
  }

  .tag-form {
    margin-bottom: 16px;
    padding: 12px;
    border-radius: 8px;
    background-color: rgba(128, 128, 128, 0.1);
  }

  .small-button {
    padding: 6px 12px;
    font-size: 0.85em;
    border-radius: 6px;
    background-color: transparent;
    border: 1px solid rgba(128, 128, 128, 0.3);
    cursor: pointer;
  }

  .frequency-options {
    margin-top: 8px;
  }

  .radio-group {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    margin-bottom: 16px;
  }

  .radio-group label {
    display: flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
    margin-bottom: 0;
  }

  .days-selection {
    margin-top: 16px;
  }

  .selection-label {
    margin-bottom: 8px;
    font-weight: 500;
    font-size: 0.9em;
  }

  .days-container {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 8px;
  }

  .month-days-container {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(40px, 1fr));
    gap: 8px;
    margin-top: 8px;
  }

  .day-button {
    padding: 8px;
    background-color: transparent;
    border: 1px solid rgba(128, 128, 128, 0.3);
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85em;
  }

  .day-button.selected {
    background-color: var(--primary-color, #3b82f6);
    color: white;
    border-color: var(--primary-color, #3b82f6);
  }

  .day-of-month {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .input-with-suffix {
    display: flex;
    align-items: center;
  }

  .input-with-suffix input {
    border-top-right-radius: 0;
    border-bottom-right-radius: 0;
    flex: 1;
  }

  .suffix {
    padding: 10px 12px;
    background-color: rgba(128, 128, 128, 0.1);
    border: 1px solid rgba(128, 128, 128, 0.3);
    border-left: none;
    border-top-right-radius: 8px;
    border-bottom-right-radius: 8px;
    font-size: 0.95em;
  }

  .toggle-button {
    width: 100%;
    padding: 10px;
    background-color: transparent;
    border: 1px dashed rgba(128, 128, 128, 0.3);
    border-radius: 8px;
    cursor: pointer;
    font-size: 0.9em;
    transition: all 0.2s;
  }

  .toggle-button:hover {
    background-color: rgba(128, 128, 128, 0.05);
  }

  .submission-section {
    margin-top: 24px;
  }

  .button-row {
    display: flex;
    justify-content: flex-end;
    gap: 16px;
    margin-top: 16px;
  }

  .primary-button,
  .secondary-button {
    padding: 10px 24px;
    border-radius: 8px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .primary-button {
    background-color: var(--primary-color, #3b82f6);
    color: white;
    border: none;
  }

  .primary-button:hover:not(:disabled) {
    background-color: var(--primary-color-dark, #2563eb);
  }

  .secondary-button {
    background-color: transparent;
    border: 1px solid rgba(128, 128, 128, 0.3);
  }

  .secondary-button:hover:not(:disabled) {
    background-color: rgba(128, 128, 128, 0.1);
  }

  .status-message {
    padding: 12px;
    border-radius: 8px;
    font-size: 0.9em;
    margin-bottom: 16px;
  }

  .status-message.success {
    background-color: rgba(16, 185, 129, 0.1);
    border: 1px solid rgb(16, 185, 129);
  }

  .status-message.error {
    background-color: rgba(239, 68, 68, 0.1);
    border: 1px solid rgb(239, 68, 68);
  }

  .empty-state {
    padding: 16px;
    text-align: center;
    color: rgba(128, 128, 128, 0.7);
    font-size: 0.9em;
    border: 1px dashed rgba(128, 128, 128, 0.3);
    border-radius: 8px;
    width: 100%;
  }

  .empty-state.large {
    padding: 32px;
    font-size: 1em;
  }

  /* Habit List/Card Styles */
  .habits-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 20px;
    margin-top: 20px;
  }

  .habit-card {
    border-radius: 10px;
    border-left: 4px solid #3b82f6;
    background-color: rgba(255, 255, 255, 0.05);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    transition:
      transform 0.2s,
      box-shadow 0.2s;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.05);
  }

  .habit-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .habit-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .habit-title {
    flex: 1;
  }

  .habit-title h3 {
    margin: 0 0 8px 0;
    font-size: 1.1em;
    line-height: 1.3;
  }

  .habit-badges {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .badge {
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 0.7em;
    font-weight: 500;
    text-transform: uppercase;
  }

  .badge.high {
    background-color: rgba(239, 68, 68, 0.15);
    color: rgb(239, 68, 68);
  }

  .badge.medium {
    background-color: rgba(245, 158, 11, 0.15);
    color: rgb(245, 158, 11);
  }

  .badge.low {
    background-color: rgba(16, 185, 129, 0.15);
    color: rgb(16, 185, 129);
  }

  .badge.status {
    background-color: rgba(59, 130, 246, 0.15);
    color: rgb(59, 130, 246);
  }

  .badge.status.inactive {
    background-color: rgba(107, 114, 128, 0.15);
    color: rgb(107, 114, 128);
  }

  .habit-description {
    margin: 0;
    font-size: 0.9em;
    opacity: 0.8;
    line-height: 1.5;
  }

  .habit-details {
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-size: 0.85em;
  }

  .detail-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .detail-label {
    font-weight: 500;
    opacity: 0.7;
  }

  .habit-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 4px;
  }

  .tag-pill {
    padding: 3px 8px;
    border-radius: 12px;
    font-size: 0.75em;
    color: white;
    white-space: nowrap;
  }

  .habit-actions {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }

  .action-button {
    flex: 1;
    padding: 8px;
    background-color: transparent;
    border: 1px solid rgba(128, 128, 128, 0.3);
    border-radius: 6px;
    font-size: 0.85em;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    transition: all 0.2s;
  }

  .action-button:hover {
    background-color: rgba(128, 128, 128, 0.1);
  }

  .action-button .icon {
    font-size: 1.1em;
  }

  /* Toggle Switch */
  .toggle-switch {
    position: relative;
    display: inline-block;
    width: 46px;
    height: 24px;
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
    background-color: rgba(128, 128, 128, 0.3);
    transition: 0.4s;
    border-radius: 34px;
  }

  .toggle-switch label:before {
    position: absolute;
    content: "";
    height: 18px;
    width: 18px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: 0.4s;
    border-radius: 50%;
  }

  .toggle-switch input:checked + label {
    background-color: #3b82f6;
  }

  .toggle-switch input:checked + label:before {
    transform: translateX(22px);
  }

  /* Theme-specific styles */
  :global(html.light) .tag-chip {
    border-color: var(--tag-color);
    background-color: rgba(255, 255, 255, 0.3);
    color: var(--tag-color);
  }

  :global(html.light) .tag-chip.selected {
    background-color: var(--tag-color);
    color: white;
  }

  :global(html.dark) .tag-chip {
    border-color: var(--tag-color);
    background-color: rgba(30, 30, 30, 0.5);
    color: var(--tag-color);
  }

  :global(html.dark) .tag-chip.selected {
    background-color: var(--tag-color);
    color: white;
  }

  /* Change button background when using theme */
  :global(html.light) .day-button.selected {
    background-color: #3b82f6;
    color: white;
    border-color: #3b82f6;
  }

  :global(html.dark) .day-button.selected {
    background-color: #3b82f6;
    color: white;
    border-color: #3b82f6;
  }

  :global(html.light) .primary-button {
    background-color: #3b82f6;
  }

  :global(html.light) .primary-button:hover:not(:disabled) {
    background-color: #2563eb;
  }

  :global(html.dark) .primary-button {
    background-color: #3b82f6;
  }

  :global(html.dark) .primary-button:hover:not(:disabled) {
    background-color: #2563eb;
  }

  /* Dark mode specific card styling */
  :global(html.dark) .habit-card {
    background-color: rgba(30, 30, 30, 0.3);
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
  }

  :global(html.dark) .habit-card:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  /* Media queries for responsive design */
  @media (max-width: 768px) {
    .habits-grid {
      grid-template-columns: 1fr;
    }

    .habit-card {
      max-width: 100%;
    }
  }
</style>
