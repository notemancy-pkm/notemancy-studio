<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
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

<main class="container mx-auto px-4 py-8">
  <!-- Main content with left sidebar for notes count -->
  <div class="flex items-center justify-center gap-6 max-w-6xl mx-auto">
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
          </div>
        {/if}
      </div>
    </div>
  </div>
</main>
