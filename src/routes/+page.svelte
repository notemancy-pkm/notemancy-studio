<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import SettingsDialog from "../components/SettingsDialog.svelte";
  import Database from "@tauri-apps/plugin-sql";
  import FuzzySearchInput from "../components/FuzzySearchInput.svelte";
  import { onMount } from "svelte";
  import "../app.css";

  let name = $state("");
  let greetMsg = $state("");
  let notes = $state([]);
  let filteredNotes = $state([]);
  let vaultPath = $state("");
  let loading = $state(true);
  let error = $state("");

  async function greet(event: Event) {
    event.preventDefault();
    greetMsg = await invoke("greet", { name });
  }

  async function loadNotes() {
    try {
      loading = true;
      error = "";

      // Load the vault path from settings
      const db = await Database.load("sqlite:settings.db");
      const result = await db.select(
        "SELECT value FROM settings WHERE key = $1",
        ["vault_path"],
      );

      if (result && result.length > 0) {
        vaultPath = result[0].value;

        // If we have a vault path, load the notes
        if (vaultPath) {
          notes = await invoke("get_notes", { vaultDirectory: vaultPath });
        } else {
          error = "No vault path configured. Please set one in settings.";
        }
      } else {
        error = "No vault path configured. Please set one in settings.";
      }
    } catch (e) {
      console.error("Failed to load notes:", e);
      error = `Error loading notes: ${e}`;
    } finally {
      loading = false;
    }
  }

  // Function to encode path for URL
  function encodePathForUrl(path) {
    return encodeURIComponent(path);
  }

  function handleSearchResults(event) {
    filteredNotes = event.detail;
  }

  onMount(() => {
    loadNotes();
  });
</script>

<SettingsDialog />

<main class="container mx-auto px-4 py-8">
  <!-- Main content with left sidebar for notes count -->
  <div class="flex flex-col md:flex-row gap-6 max-w-6xl mx-auto">
    <!-- Left sidebar with note count -->
    <div class="w-full md:w-64 flex-shrink-0">
      <div class="rounded-xl shadow-lg p-6 text-gray-800">
        <h3 class="text-lg font-medium opacity-90 mb-1">Notes Count</h3>

        {#if loading}
          <div class="animate-pulse h-12 bg-white/20 rounded mt-2"></div>
        {:else if error}
          <div class="flex items-center mt-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-6 w-6 mr-2"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
              />
            </svg>
            <span>Error</span>
          </div>
        {:else}
          <div class="flex flex-col">
            <span class="text-4xl font-bold">{notes.length}</span>
            <span class="text-sm opacity-75 mt-1">
              {notes.length === 1 ? "note" : "notes"} in vault
            </span>
            {#if notes.length !== filteredNotes.length}
              <span class="text-xs text-blue-600 mt-2">
                Filtered: {filteredNotes.length} notes
              </span>
            {/if}
          </div>
        {/if}
      </div>
    </div>

    <!-- Notes table on the right -->
    <div class="flex-grow">
      <h2 class="text-2xl font-semibold mb-4">Your Notes</h2>
      {#if notes.length > 0 && !loading && !error}
        <FuzzySearchInput
          data={notes}
          keys={["title", "relative_path"]}
          on:results={handleSearchResults}
          placeholder="Search notes by title or path..."
        />
      {/if}

      {#if loading}
        <div class="text-center py-10">
          <p class="text-gray-500">Loading notes...</p>
        </div>
      {:else if error}
        <div class="bg-red-50 text-red-700 p-4 rounded-lg text-center">
          <p>{error}</p>
        </div>
      {:else if notes.length === 0}
        <div class="bg-blue-50 text-blue-700 p-6 rounded-lg text-center">
          <p>
            No notes found in your vault. Add some markdown files to get
            started!
          </p>
        </div>
      {:else}
        <div class="bg-white rounded-lg shadow overflow-hidden">
          <table class="min-w-full divide-y divide-none">
            <thead class="bg-gray-50">
              <tr>
                <th
                  class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  Note Title
                </th>
              </tr>
            </thead>
            <tbody class="bg-white divide-y divide-none">
              {#if filteredNotes.length === 0}
                <tr>
                  <td
                    class="px-6 py-4 text-center text-sm text-gray-500"
                    colspan="1"
                  >
                    No matching notes found
                  </td>
                </tr>
              {:else}
                {#each filteredNotes as note}
                  <tr class="hover:bg-gray-50 transition-colors duration-150">
                    <td class="px-6 py-4 whitespace-nowrap">
                      <a
                        href={`/note/${encodePathForUrl(note.relative_path)}`}
                        class="block"
                      >
                        <div
                          class="text-sm font-medium text-blue-600 hover:text-blue-800 hover:underline"
                        >
                          {note.title}
                        </div>
                        <div class="text-xs text-gray-500 mt-0.5">
                          {note.relative_path}
                        </div>
                      </a>
                    </td>
                  </tr>
                {/each}
              {/if}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  </div>
</main>
