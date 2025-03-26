<script lang="ts">
  import { createDialog, melt } from "@melt-ui/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Database from "@tauri-apps/plugin-sql";
  import { onMount, onDestroy } from "svelte";
  import { fade } from "svelte/transition";
  import { goto } from "$app/navigation";
  import Fuse from "fuse.js";

  let db: any;
  let notes = $state([]);
  let filteredNotes = $state([]);
  let searchQuery = $state("");
  let vaultPath = $state("");
  let isLoading = $state(false);
  let error = $state("");
  let fuse: Fuse<any>;

  // Dialog setup
  const {
    elements: {
      trigger,
      overlay,
      content,
      title,
      description,
      close,
      portalled,
    },
    states: { open },
  } = createDialog({
    defaultOpen: false,
  });

  // Handle keyboard shortcut (Ctrl + O)
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "o" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      open.set(true);

      // Load notes when dialog opens
      if (notes.length === 0) {
        loadNotes();
      }
    }
  }

  async function initDatabase() {
    try {
      db = await Database.load("sqlite:settings.db");
      loadVaultPath();
    } catch (e) {
      console.error("Database initialization error:", e);
      error = `Failed to initialize database: ${e}`;
    }
  }

  async function loadVaultPath() {
    try {
      // Query the database directly using the SQL plugin
      const result = await db.select(
        "SELECT value FROM settings WHERE key = $1",
        ["vault_path"],
      );
      if (result && result.length > 0) {
        vaultPath = result[0].value;
      }
    } catch (e) {
      console.error("Failed to load vault path:", e);
    }
  }

  async function loadNotes() {
    try {
      isLoading = true;
      error = "";

      // If vault path is not loaded yet, load it
      if (!vaultPath) {
        await loadVaultPath();
      }

      // If we have a vault path, load the notes
      if (vaultPath) {
        notes = await invoke("get_notes", { vaultDirectory: vaultPath });

        // Initialize fuse for searching
        const options = {
          includeScore: true,
          threshold: 0.4,
          keys: ["title", "relative_path"],
        };

        fuse = new Fuse(notes, options);
        updateFilteredNotes();
      } else {
        error = "No vault path configured. Please set one in settings.";
      }
    } catch (e) {
      console.error("Failed to load notes:", e);
      error = `Error loading notes: ${e}`;
    } finally {
      isLoading = false;
    }
  }

  function updateFilteredNotes() {
    if (!fuse || searchQuery.trim() === "") {
      filteredNotes = notes;
    } else {
      filteredNotes = fuse.search(searchQuery).map((result) => result.item);
    }
  }

  // Update filtered notes when search query changes
  $effect(() => {
    updateFilteredNotes();
  });

  function openNote(relativePath: string) {
    console.log("REL: ", relativePath);
    // Navigate to the note page
    goto(`/note/${encodeURIComponent(relativePath)}`);
    open.set(false); // Close the dialog
    searchQuery = ""; // Reset search
  }

  function clearSearch() {
    searchQuery = "";
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
    initDatabase();
  });

  onDestroy(() => {
    window.removeEventListener("keydown", handleKeydown);
  });

  function flyAndScale(node, { duration = 150, y = 8, start = 0.96 }) {
    return {
      duration,
      css: (t) => `
        transform: translate3d(0, ${(1 - t) * y}px, 0) scale(${start + (1 - start) * t});
        opacity: ${t}
      `,
    };
  }
</script>

<!-- Button can be hidden since we're using keyboard shortcut -->
<button use:melt={$trigger} class="hidden"> Open File Dialog </button>

{#if $open}
  <div use:melt={$portalled}>
    <div
      use:melt={$overlay}
      class="fixed inset-0 z-50 bg-black/50"
      transition:fade={{ duration: 150 }}
    />
    <div
      class="fixed left-1/2 top-1/2 z-50 max-h-[85vh] w-[90vw]
            max-w-[600px] -translate-x-1/2 -translate-y-1/2 rounded-xl bg-white
            p-6 shadow-lg overflow-hidden flex flex-col"
      transition:flyAndScale={{
        duration: 150,
        y: 8,
        start: 0.96,
      }}
      use:melt={$content}
    >
      <h2 use:melt={$title} class="m-0 text-lg font-medium text-black">
        Open Note
      </h2>
      <p use:melt={$description} class="mb-5 mt-2 leading-normal text-zinc-600">
        Press Ctrl+O to search and open a note from your vault.
      </p>

      {#if error}
        <div class="mb-4 p-2 bg-red-100 text-red-800 rounded">
          {error}
        </div>
      {/if}

      <!-- Search input -->
      <div class="relative mb-4">
        <div class="relative">
          <input
            type="text"
            class="w-full h-10 px-4 pl-10 text-sm bg-white border border-gray-200 rounded-lg
                  focus:outline-none focus:ring-2 focus:ring-blue-400 transition-all duration-200"
            bind:value={searchQuery}
            placeholder="Search notes by title or path..."
            autofocus
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
          {#if searchQuery}
            <button
              type="button"
              class="absolute inset-y-0 right-0 flex items-center pr-3 text-gray-400 hover:text-gray-600"
              onclick={clearSearch}
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

      <!-- Results list -->
      <div class="flex-grow overflow-y-auto max-h-[50vh]">
        {#if isLoading}
          <div class="text-center py-4">
            <p class="text-gray-500">Loading notes...</p>
          </div>
        {:else if notes.length === 0}
          <div class="text-center py-4">
            <p class="text-gray-500">No notes found in your vault</p>
          </div>
        {:else if filteredNotes.length === 0}
          <div class="text-center py-4">
            <p class="text-gray-500">No matching notes found</p>
          </div>
        {:else}
          <ul class="divide-y divide-gray-100">
            {#each filteredNotes as note}
              <li>
                <button
                  class="w-full text-left px-4 py-3 hover:bg-gray-50 transition-colors duration-150 rounded"
                  onclick={() => openNote(note.relative_path)}
                >
                  <div class="font-medium text-blue-600">
                    {note.title}
                  </div>
                  <div class="text-xs text-gray-500 mt-0.5">
                    {note.relative_path}
                  </div>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>

      <div class="mt-6 flex justify-end gap-4">
        <button
          use:melt={$close}
          class="inline-flex h-8 items-center justify-center rounded-sm
                  bg-zinc-100 px-4 font-medium leading-none text-zinc-600"
        >
          Cancel
        </button>
      </div>

      <button
        use:melt={$close}
        aria-label="close"
        class="absolute right-4 top-4 inline-flex h-6 w-6 appearance-none
                items-center justify-center rounded-full p-1 text-zinc-800
                hover:bg-zinc-100 focus:shadow-blue-400"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="24"
          height="24"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="size-4"
        >
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>
  </div>
{/if}
