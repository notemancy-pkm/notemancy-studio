<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import Fuse from "fuse.js";

  // Props using Svelte 5 runes syntax
  const props = $props<{
    placeholder?: string;
    data?: any[];
    keys?: string[];
    threshold?: number;
  }>();

  // Set default values for props
  const placeholder = props.placeholder ?? "Search notes...";
  const data = props.data ?? [];
  const keys = props.keys ?? ["title"];
  const threshold = props.threshold ?? 0.4;

  // State
  let query = $state("");
  let results = $state<any[]>([]);
  let fuse: Fuse<any>;

  // Event handling
  const dispatch = createEventDispatcher();

  // Initialize Fuse.js when data changes
  $effect(() => {
    if (data.length > 0) {
      const options = {
        includeScore: true,
        threshold,
        keys,
      };

      fuse = new Fuse(data, options);
      performSearch();
    }
  });

  // Perform search when query changes
  $effect(() => {
    performSearch();
  });

  function performSearch() {
    if (!fuse || query.trim() === "") {
      results = data;
    } else {
      results = fuse.search(query).map((result) => result.item);
    }
    dispatch("results", results);
  }

  function clearSearch() {
    query = "";
  }
</script>

<div class="relative mb-4">
  <div class="relative">
    <input
      type="text"
      class="w-full h-10 px-4 pl-10 text-sm bg-white border-none rounded-lg shadow-sm
      focus:outline-none focus:ring-2 focus:ring-blue-400 transition-all duration-200"
      bind:value={query}
      {placeholder}
    />

    <!-- Search icon -->
    <div
      class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none text-gray-400"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="w-5 h-5"
        viewBox="0 0 24 24"
      >
        <path
          fill="currentColor"
          d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"
        />
      </svg>
    </div>

    <!-- Clear button (only shows when there's text) -->
    {#if query}
      <button
        type="button"
        class="absolute inset-y-0 right-0 flex items-center pr-3 text-gray-400 hover:text-gray-600"
        on:click={clearSearch}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="w-5 h-5"
          viewBox="0 0 24 24"
        >
          <path
            fill="currentColor"
            d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"
          />
        </svg>
      </button>
    {/if}
  </div>
</div>
