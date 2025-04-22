<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  // props
  const {
    allTags = [],
    onHabitCreated,
    onTagCreated,
  } = $props<{
    allTags: { id: number; name: string; color: string }[];
    onHabitCreated: () => Promise<void>;
    onTagCreated: () => Promise<void>;
  }>();

  // Form state
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
  let habitColor = $state("#3b82f6");
  let habitIcon = $state("");
  let habitPriority = $state(2);
  let habitStartDate = $state(new Date().toISOString().split("T")[0]);
  let habitEndDate = $state("");
  let habitReminderTime = $state("");

  // UI state
  let isLoading = $state(false);
  let statusMessage = $state("");
  let isSuccess = $state(false);
  let showAdvanced = $state(false);
  let showTagForm = $state(false);
  let newTagName = $state("");
  let newTagColor = $state("#3b82f6");

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

  // sample categories for organization
  const categories = [
    { id: "health", name: "Health", color: "#10b981" },
    { id: "fitness", name: "Fitness", color: "#f97316" },
    { id: "productivity", name: "Productivity", color: "#3b82f6" },
    { id: "learning", name: "Learning", color: "#8b5cf6" },
    { id: "mindfulness", name: "Mindfulness", color: "#ec4899" },
    { id: "finance", name: "Finance", color: "#14b8a6" },
    { id: "social", name: "Social", color: "#f43f5e" },
    { id: "other", name: "Other", color: "#6b7280" },
  ];

  // toggle tag selection
  function toggleTag(tagName: string) {
    if (habitTags.includes(tagName)) {
      habitTags = habitTags.filter((tag) => tag !== tagName);
    } else {
      habitTags = [...habitTags, tagName];
    }
  }

  // set category
  function selectCategory(categoryId: string) {
    const category = categories.find((c) => c.id === categoryId);
    if (category) {
      habitCategory = category.name;
      habitColor = category.color;
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
      await onTagCreated();
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

  // Select weekdays
  function selectWeekdays() {
    habitFrequencyDays = [1, 2, 3, 4, 5];
  }

  // Select weekends
  function selectWeekends() {
    habitFrequencyDays = [6, 7];
  }

  // Select all days
  function selectAllDays() {
    habitFrequencyDays = [1, 2, 3, 4, 5, 6, 7];
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
      await onHabitCreated();

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
</script>

<div class="habit-form">
  <form onsubmit={handleAddHabit}>
    <div class="form-layout">
      <!-- left Column -->
      <div class="form-column main-column">
        <!-- basic Information Section -->
        <div class="form-section">
          <input
            id="habit-name"
            type="text"
            placeholder="Name your habit"
            bind:value={habitName}
            required
            disabled={isLoading}
            class="habit-name-input"
          />

          <textarea
            placeholder="Description (optional)"
            bind:value={habitDescription}
            disabled={isLoading}
            class="habit-description-input"
          ></textarea>
        </div>

        <!-- frequency Selection -->
        <div class="form-section">
          <h3 class="section-title">Frequency</h3>

          <div class="radio-group frequency-options">
            <label class="radio-option">
              <input
                type="radio"
                bind:group={habitFrequencyType}
                value="daily"
                disabled={isLoading}
              />
              <span class="radio-label">Daily</span>
            </label>

            <label class="radio-option">
              <input
                type="radio"
                bind:group={habitFrequencyType}
                value="weekly"
                disabled={isLoading}
              />
              <span class="radio-label">Weekly</span>
            </label>

            <label class="radio-option">
              <input
                type="radio"
                bind:group={habitFrequencyType}
                value="monthly"
                disabled={isLoading}
              />
              <span class="radio-label">Monthly</span>
            </label>

            <label class="radio-option">
              <input
                type="radio"
                bind:group={habitFrequencyType}
                value="interval"
                disabled={isLoading}
              />
              <span class="radio-label">Every X Days</span>
            </label>
          </div>

          <!-- weekly frequency options -->
          {#if habitFrequencyType === "weekly"}
            <div class="frequency-details">
              <div class="frequency-quick-options">
                <button
                  type="button"
                  class="quick-select-button"
                  onclick={selectWeekdays}>Weekdays</button
                >
                <button
                  type="button"
                  class="quick-select-button"
                  onclick={selectWeekends}>Weekends</button
                >
                <button
                  type="button"
                  class="quick-select-button"
                  onclick={selectAllDays}>Every Day</button
                >
              </div>

              <div class="days-selection">
                {#each daysOfWeek as day}
                  <button
                    type="button"
                    class="day-button"
                    class:selected={habitFrequencyDays.includes(day.id)}
                    onclick={() => toggleDay(day.id)}
                    disabled={isLoading}
                  >
                    {day.name.substring(0, 3)}
                  </button>
                {/each}
              </div>
            </div>
          {/if}

          <!-- monthly frequency options -->
          {#if habitFrequencyType === "monthly"}
            <div class="frequency-details">
              <div class="month-days-selection">
                {#each Array(31) as _, i}
                  <button
                    type="button"
                    class="day-button month-day"
                    class:selected={habitFrequencyDays.includes(i + 1)}
                    onclick={() => toggleDay(i + 1)}
                    disabled={isLoading}
                  >
                    {i + 1}
                  </button>
                {/each}
              </div>
            </div>
          {/if}

          <!-- interval frequency options -->
          {#if habitFrequencyType === "interval"}
            <div class="frequency-details">
              <div class="interval-input-group">
                <span>Every</span>
                <input
                  type="number"
                  min="1"
                  bind:value={habitFrequencyInterval}
                  disabled={isLoading}
                />
                <span>days</span>
              </div>
            </div>
          {/if}

          <!-- custom frequency options -->
          {#if habitFrequencyType === "custom"}
            <div class="frequency-details">
              <textarea
                placeholder="Describe your custom frequency pattern"
                bind:value={habitFrequencyCustomPattern}
                disabled={isLoading}
                class="custom-pattern-input"
              ></textarea>
            </div>
          {/if}
        </div>

        <!-- tags section -->
        <div class="form-section">
          <div class="section-header">
            <h3 class="section-title">Tags</h3>
            <button
              type="button"
              class="add-button"
              onclick={() => (showTagForm = !showTagForm)}
              disabled={isLoading}
            >
              {showTagForm ? "Cancel" : "Add Tag"}
            </button>
          </div>

          {#if showTagForm}
            <div class="tag-form">
              <div class="tag-form-inputs">
                <input
                  type="text"
                  placeholder="Tag name"
                  bind:value={newTagName}
                  disabled={isLoading}
                  class="tag-name-input"
                />
                <div class="color-input-wrapper">
                  <input
                    type="color"
                    bind:value={newTagColor}
                    disabled={isLoading}
                    class="color-input"
                  />
                </div>
                <button
                  type="button"
                  class="create-tag-button"
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
                  class="tag"
                  class:selected={habitTags.includes(tag.name)}
                  style="--tag-color: {tag.color || habitColor}"
                  onclick={() => toggleTag(tag.name)}
                  disabled={isLoading}
                >
                  {tag.name}
                </button>
              {/each}
            {:else}
              <div class="empty-message">
                No tags available. Create some tags to organize your habits.
              </div>
            {/if}
          </div>
        </div>
      </div>

      <!-- right Column -->
      <div class="form-column side-column">
        <!-- category selection section -->
        <div class="form-section">
          <h3 class="section-title">Category</h3>

          <div class="categories-grid">
            {#each categories as category}
              <button
                type="button"
                class="category-button"
                class:selected={habitCategory === category.name}
                style="--category-color: {category.color}"
                onclick={() => selectCategory(category.id)}
                disabled={isLoading}
              >
                {category.name}
              </button>
            {/each}
          </div>
        </div>

        <!-- color selection -->
        <div class="form-section">
          <h3 class="section-title">Color</h3>

          <div class="color-selection">
            <input
              type="color"
              bind:value={habitColor}
              disabled={isLoading}
              class="color-picker"
              id="habit-color"
            />
            <label
              for="habit-color"
              class="color-preview"
              style="background-color: {habitColor}"
            ></label>
            <span class="color-hex">{habitColor}</span>
          </div>
        </div>

        <!-- advanced options section -->
        <div class="form-section">
          <button
            type="button"
            class="advanced-toggle"
            onclick={() => (showAdvanced = !showAdvanced)}
          >
            <span>{showAdvanced ? "Hide" : "Show"} Advanced Options</span>
            <span class="toggle-icon">{showAdvanced ? "−" : "+"}</span>
          </button>

          {#if showAdvanced}
            <div class="advanced-options">
              <!-- target value/unit -->
              <div class="option-group">
                <p class="option-label">Target</p>
                <div class="target-inputs">
                  <input
                    type="number"
                    placeholder="Value"
                    bind:value={habitTargetValue}
                    min="0"
                    step="any"
                    disabled={isLoading}
                    class="target-value"
                  />
                  <input
                    type="text"
                    placeholder="Unit (e.g. pages, minutes)"
                    bind:value={habitTargetUnit}
                    disabled={isLoading}
                    class="target-unit"
                  />
                </div>
              </div>

              <!-- priority -->
              <div class="option-group">
                <p class="option-label">Priority</p>
                <div class="priority-options">
                  <label class="priority-option">
                    <input
                      type="radio"
                      bind:group={habitPriority}
                      value={1}
                      disabled={isLoading}
                    />
                    <span class="priority-label high">High</span>
                  </label>
                  <label class="priority-option">
                    <input
                      type="radio"
                      bind:group={habitPriority}
                      value={2}
                      disabled={isLoading}
                    />
                    <span class="priority-label medium">Medium</span>
                  </label>
                  <label class="priority-option">
                    <input
                      type="radio"
                      bind:group={habitPriority}
                      value={3}
                      disabled={isLoading}
                    />
                    <span class="priority-label low">Low</span>
                  </label>
                </div>
              </div>

              <!-- dates -->
              <div class="option-group">
                <p class="option-label">Dates</p>
                <div class="dates-container">
                  <div class="date-field">
                    <label for="start-date">Start</label>
                    <input
                      id="start-date"
                      type="date"
                      bind:value={habitStartDate}
                      disabled={isLoading}
                      class="date-input"
                    />
                  </div>
                  <div class="date-field">
                    <label for="end-date">End (Optional)</label>
                    <input
                      id="end-date"
                      type="date"
                      bind:value={habitEndDate}
                      min={habitStartDate}
                      disabled={isLoading}
                      class="date-input"
                    />
                  </div>
                </div>
              </div>

              <!-- reminder -->
              <div class="option-group">
                <p class="option-label">Reminder Time</p>
                <input
                  type="time"
                  bind:value={habitReminderTime}
                  disabled={isLoading}
                  class="reminder-input"
                />
              </div>
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- submission -->
    <div class="form-footer">
      {#if statusMessage}
        <div
          class="status-message"
          class:success={isSuccess}
          class:error={!isSuccess}
        >
          {statusMessage}
        </div>
      {/if}

      <div class="action-buttons">
        <button
          type="button"
          class="cancel-button"
          onclick={resetForm}
          disabled={isLoading}
        >
          Cancel
        </button>
        <button
          type="submit"
          class="submit-button"
          disabled={isLoading || !habitName.trim()}
        >
          {isLoading ? "Creating..." : "Create Habit"}
        </button>
      </div>
    </div>
  </form>
</div>

<style>
  /* form layout */
  .form-layout {
    display: flex;
    gap: 24px;
    margin-bottom: 24px;
  }

  .form-column {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .main-column {
    flex: 3;
  }

  .side-column {
    flex: 2;
  }

  /* Form Sections */
  .form-section {
    margin-bottom: 0;
    background-color: rgba(255, 255, 255, 0.15);
    border-radius: 6px;
    padding: 16px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  }

  .section-title {
    font-size: 14px;
    font-weight: 500;
    margin: 0 0 12px 0;
    color: rgba(60, 60, 67, 0.9);
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .habit-name-input::placeholder,
  .habit-description-input::placeholder,
  .tag-name-input::placeholder,
  .target-value::placeholder,
  .target-unit::placeholder,
  .custom-pattern-input::placeholder {
    color: rgba(0, 0, 0, 0.5);
  }

  /* Name and Description Inputs */
  .habit-name-input {
    width: 100%;
    padding: 12px 0;
    font-size: 20px;
    font-weight: 500;
    border: none;
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
    background-color: transparent;
    margin-bottom: 12px;
    color: inherit;
  }

  .habit-name-input:focus {
    outline: none;
    border-color: rgba(0, 0, 0, 0.3);
  }

  .habit-description-input {
    width: 100%;
    padding: 12px 0;
    font-size: 14px;
    border: none;
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
    background-color: transparent;
    resize: vertical;
    min-height: 60px;
    color: inherit;
    font-family: inherit;
  }

  .habit-description-input:focus {
    outline: none;
    border-color: rgba(0, 0, 0, 0.3);
  }

  /* Frequency Options */
  .frequency-options {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    margin-bottom: 16px;
  }

  .radio-option {
    display: flex;
    align-items: center;
    cursor: pointer;
  }

  .radio-label {
    margin-left: 6px;
    font-size: 14px;
  }

  .frequency-details {
    margin-top: 12px;
    background-color: rgba(0, 0, 0, 0.02);
    border-radius: 4px;
    padding: 16px;
  }

  .frequency-quick-options {
    display: flex;
    gap: 8px;
    margin-bottom: 12px;
  }

  .quick-select-button {
    background-color: transparent;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    padding: 6px 12px;
    font-size: 13px;
    cursor: pointer;
    color: inherit;
  }

  .quick-select-button:hover {
    background-color: rgba(0, 0, 0, 0.03);
  }

  .days-selection {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .day-button {
    width: 38px;
    height: 38px;
    background-color: transparent;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
    color: inherit;
  }

  .day-button:hover {
    background-color: rgba(0, 0, 0, 0.03);
  }

  .day-button.selected {
    background-color: #3b82f6;
    color: white;
    border-color: #3b82f6;
  }

  .month-days-selection {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(36px, 1fr));
    gap: 8px;
  }

  .month-day {
    width: 36px;
    height: 36px;
    font-size: 12px;
  }

  .interval-input-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .interval-input-group input {
    width: 60px;
    padding: 8px;
    text-align: center;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    background-color: transparent;
    color: inherit;
  }

  .custom-pattern-input {
    width: 100%;
    padding: 8px;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    background-color: transparent;
    resize: vertical;
    min-height: 60px;
    font-family: inherit;
    font-size: 14px;
    color: inherit;
  }

  /* Tags Section */
  .add-button {
    background-color: transparent;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    padding: 6px 12px;
    font-size: 13px;
    cursor: pointer;
    color: inherit;
  }

  .add-button:hover {
    background-color: rgba(0, 0, 0, 0.03);
  }

  .tag-form {
    margin-bottom: 16px;
  }

  .tag-form-inputs {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .tag-name-input {
    flex: 1;
    padding: 8px;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    background-color: transparent;
    font-size: 14px;
    color: inherit;
  }

  .color-input-wrapper {
    position: relative;
    width: 30px;
    height: 30px;
    overflow: hidden;
    border-radius: 4px;
  }

  .color-input {
    position: absolute;
    top: -5px;
    left: -5px;
    width: 40px;
    height: 40px;
    border: none;
    cursor: pointer;
  }

  .create-tag-button {
    background-color: transparent;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    padding: 8px 12px;
    font-size: 13px;
    cursor: pointer;
    color: inherit;
  }

  .create-tag-button:hover:not(:disabled) {
    background-color: rgba(0, 0, 0, 0.03);
  }

  .create-tag-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tags-container {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .tag {
    padding: 6px 12px;
    background-color: transparent;
    border: 1px solid var(--tag-color, #3b82f6);
    border-radius: 4px;
    font-size: 13px;
    color: var(--tag-color, #3b82f6);
    cursor: pointer;
    transition: all 0.2s;
  }

  .tag:hover {
    background-color: rgba(0, 0, 0, 0.03);
  }

  .tag.selected {
    background-color: var(--tag-color, #3b82f6);
    color: white;
  }

  .empty-message {
    padding: 16px;
    color: rgba(0, 0, 0, 0.5);
    font-size: 14px;
    text-align: center;
    border: 1px dashed rgba(0, 0, 0, 0.1);
    border-radius: 4px;
  }

  /* Categories */
  .categories-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 8px;
  }

  .category-button {
    padding: 10px 8px;
    background-color: transparent;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    font-size: 13px;
    color: inherit;
    cursor: pointer;
    transition: all 0.2s;
  }

  .category-button:hover {
    background-color: rgba(var(--category-color, #3b82f6), 0.05);
    border-color: var(--category-color, #3b82f6);
  }

  .category-button.selected {
    background-color: var(--category-color, #3b82f6);
    color: white;
    border-color: var(--category-color, #3b82f6);
  }

  /* Color Selection */
  .color-selection {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .color-picker {
    width: 0;
    height: 0;
    padding: 0;
    visibility: hidden;
  }

  .color-preview {
    width: 36px;
    height: 36px;
    border-radius: 4px;
    border: 1px solid rgba(0, 0, 0, 0.1);
    cursor: pointer;
  }

  .color-hex {
    font-size: 14px;
    color: rgba(0, 0, 0, 0.6);
  }

  /* Advanced Options */
  .advanced-toggle {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px;
    background-color: transparent;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    font-size: 14px;
    cursor: pointer;
    color: inherit;
    margin-bottom: 0;
  }

  .advanced-toggle:hover {
    background-color: rgba(0, 0, 0, 0.02);
  }

  .toggle-icon {
    font-size: 16px;
  }

  .advanced-options {
    margin-top: 16px;
    padding: 16px;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    background-color: rgba(0, 0, 0, 0.02);
    overflow: hidden;
  }

  .option-group {
    margin-bottom: 16px;
  }

  .option-group:last-child {
    margin-bottom: 0;
  }

  .option-label {
    display: block;
    font-size: 14px;
    margin-bottom: 8px;
    color: rgba(60, 60, 67, 0.6);
  }

  .target-inputs {
    display: flex;
    gap: 8px;
  }

  .target-value {
    width: 80px;
    padding: 8px;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    background-color: transparent;
    color: inherit;
  }

  .target-unit {
    flex: 1;
    padding: 8px;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    background-color: transparent;
    color: inherit;
  }

  .priority-options {
    display: flex;
    gap: 8px;
  }

  .priority-option {
    position: relative;
  }

  .priority-option input {
    position: absolute;
    opacity: 0;
  }

  .priority-label {
    display: block;
    padding: 8px 16px;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .priority-label.high {
    background-color: rgba(239, 68, 68, 0.1);
    color: #ef4444;
  }

  .priority-label.medium {
    background-color: rgba(245, 158, 11, 0.1);
    color: #f59e0b;
  }

  .priority-label.low {
    background-color: rgba(16, 185, 129, 0.1);
    color: #10b981;
  }

  .priority-option input:checked + .priority-label.high {
    background-color: #ef4444;
    color: white;
  }

  .priority-option input:checked + .priority-label.medium {
    background-color: #f59e0b;
    color: white;
  }

  .priority-option input:checked + .priority-label.low {
    background-color: #10b981;
    color: white;
  }

  .dates-container {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
  }

  .date-field {
    flex: 1;
    min-width: 140px;
  }

  .date-field label {
    display: block;
    font-size: 12px;
    margin-bottom: 4px;
    color: rgba(60, 60, 67, 0.6);
  }

  .date-input,
  .reminder-input {
    width: 100%;
    padding: 8px;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    background-color: transparent;
    color: inherit;
  }

  /* Form Footer */
  .form-footer {
    margin-top: 24px;
    border-top: 1px solid rgba(0, 0, 0, 0.1);
    padding-top: 16px;
  }

  .status-message {
    padding: 12px;
    border-radius: 4px;
    margin-bottom: 16px;
    font-size: 14px;
  }

  .status-message.success {
    background-color: rgba(16, 185, 129, 0.1);
    border: 1px solid rgba(16, 185, 129, 0.2);
    color: #10b981;
  }

  .status-message.error {
    background-color: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .action-buttons {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
  }

  .cancel-button,
  .submit-button {
    padding: 10px 20px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .cancel-button {
    background-color: transparent;
    border: 1px solid rgba(0, 0, 0, 0.1);
    color: inherit;
  }

  .cancel-button:hover:not(:disabled) {
    background-color: rgba(0, 0, 0, 0.03);
  }

  .submit-button {
    background-color: #3b82f6;
    color: white;
    border: none;
  }

  .submit-button:hover:not(:disabled) {
    background-color: #2563eb;
  }

  .submit-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* Dark Mode Adjustments */
  :global(html.dark) .habit-name-input,
  :global(html.dark) .habit-description-input {
    border-color: rgba(255, 255, 255, 0.1);
  }

  :global(html.dark) .form-section {
    background-color: rgba(50, 50, 50, 0.5);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  }

  :global(html.dark) .habit-name-input:focus,
  :global(html.dark) .habit-description-input:focus {
    border-color: rgba(255, 255, 255, 0.3);
  }

  :global(html.dark) .quick-select-button,
  :global(html.dark) .day-button,
  :global(html.dark) .add-button,
  :global(html.dark) .tag-name-input,
  :global(html.dark) .create-tag-button,
  :global(html.dark) .category-button,
  :global(html.dark) .color-preview,
  :global(html.dark) .advanced-toggle,
  :global(html.dark) .target-value,
  :global(html.dark) .target-unit,
  :global(html.dark) .date-input,
  :global(html.dark) .reminder-input,
  :global(html.dark) .cancel-button,
  :global(html.dark) .frequency-details,
  :global(html.dark) .advanced-options {
    background-color: rgba(255, 255, 255, 0.03);
  }

  :global(html.dark) .empty-message {
    color: rgba(255, 255, 255, 0.5);
    border-color: rgba(255, 255, 255, 0.1);
  }

  :global(html.dark) .color-hex {
    color: rgba(255, 255, 255, 0.6);
  }

  :global(html.dark) .section-title,
  :global(html.dark) .option-label,
  :global(html.dark) .date-field label {
    color: rgba(200, 200, 200, 0.9);
  }

  :global(html.dark) .habit-name-input::placeholder,
  :global(html.dark) .habit-description-input::placeholder,
  :global(html.dark) .tag-name-input::placeholder,
  :global(html.dark) .target-value::placeholder,
  :global(html.dark) .target-unit::placeholder,
  :global(html.dark) .custom-pattern-input::placeholder {
    color: rgba(255, 255, 255, 0.6);
  }

  @media (max-width: 900px) {
    .form-layout {
      flex-direction: column;
    }

    .form-column {
      width: 100%;
    }

    .categories-grid {
      grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    }
  }

  @media (max-width: 600px) {
    .categories-grid {
      grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
    }

    .dates-container {
      flex-direction: column;
    }

    .action-buttons {
      flex-direction: column-reverse;
    }

    .cancel-button,
    .submit-button {
      width: 100%;
    }
  }
</style>
