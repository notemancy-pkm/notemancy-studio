<script lang="ts">
  import { createEventDispatcher, onDestroy } from "svelte";
  import { fade } from "svelte/transition";

  // Types
  export interface CardData {
    id?: string;
    title: string;
    description: string;
  }

  // Props
  const props = $props<{
    show: boolean;
    card?: CardData;
    isEdit?: boolean;
  }>();

  // Event dispatcher
  const dispatch = createEventDispatcher<{
    save: CardData;
    close: void;
  }>();

  // State
  let card = $state<CardData>({
    id: undefined,
    title: "",
    description: "",
  });

  // Reset form when modal opens/closes or card changes
  $effect(() => {
    if (props.show && props.card) {
      card = { ...props.card };
    } else if (props.show) {
      card = { id: undefined, title: "", description: "" };
    }
  });

  // Handle escape key press to close modal
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && props.show) {
      dispatch("close");
    }
  }

  // Save the card
  function saveCard() {
    if (card.title.trim() === "") return;

    dispatch("save", {
      id: card.id || `card-${Date.now()}`,
      title: card.title,
      description: card.description,
    });
  }

  // Set up event listeners
  onDestroy(() => {
    window.removeEventListener("keydown", handleKeydown);
  });

  $effect(() => {
    if (props.show) {
      window.addEventListener("keydown", handleKeydown);
    } else {
      window.removeEventListener("keydown", handleKeydown);
    }
  });
</script>

{#if props.show}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4"
    transition:fade={{ duration: 200 }}
  >
    <!-- Backdrop -->
    <div
      class="absolute inset-0 bg-black/50"
      on:click={() => dispatch("close")}
    ></div>

    <!-- Modal -->
    <div
      class="relative w-full max-w-md bg-white shadow-lg p-6 z-10"
      transition:fade={{ duration: 150, delay: 50 }}
    >
      <h3 class="text-lg font-medium text-gray-900 mb-4">
        {props.isEdit ? "Edit Card" : "Add New Card"}
      </h3>

      <form on:submit|preventDefault={saveCard}>
        <div class="mb-4">
          <label
            for="card-title"
            class="block text-sm font-medium text-gray-700 mb-1"
          >
            Title
          </label>
          <input
            id="card-title"
            type="text"
            bind:value={card.title}
            placeholder="Enter card title"
            class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-400 transition-all"
            required
          />
        </div>

        <div class="mb-6">
          <label
            for="card-description"
            class="block text-sm font-medium text-gray-700 mb-1"
          >
            Description
          </label>
          <textarea
            id="card-description"
            bind:value={card.description}
            placeholder="Enter card description"
            rows="4"
            class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-400 transition-all"
          ></textarea>
        </div>

        <div class="flex justify-end gap-2">
          <button
            type="button"
            on:click={() => dispatch("close")}
            class="px-4 py-2 bg-gray-100 text-gray-700 rounded-md text-sm hover:bg-gray-200 transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            class="px-4 py-2 bg-blue-500 text-white rounded-md text-sm hover:bg-blue-600 transition-colors"
          >
            {props.isEdit ? "Save Changes" : "Add Card"}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
