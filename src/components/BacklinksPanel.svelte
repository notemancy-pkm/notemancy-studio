<!-- src/components/BacklinksPanel.svelte -->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { createEventDispatcher, onMount, onDestroy } from "svelte";
  import { fade, fly } from "svelte/transition";
  import Database from "@tauri-apps/plugin-sql";

  // Props
  const props = $props<{
    relativePath?: string;
  }>();

  // State
  let vaultPath = $state("");
  let isVisible = $state(false);
  let backlinks = $state<{ relative_path: string; title: string }[]>([]);
  let isLoading = $state(false);
  let error = $state("");
  let debugInfo = $state("");
  let lastFetchedPath = $state("");

  const dispatch = createEventDispatcher();

  // Toggle visibility with keyboard shortcut (Ctrl/Cmd + B)
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "b" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      toggleVisibility();
    }
  }

  function toggleVisibility() {
    isVisible = !isVisible;
    debugInfo = `Panel visibility toggled to: ${isVisible}`;

    if (
      isVisible &&
      props.relativePath &&
      lastFetchedPath !== props.relativePath
    ) {
      fetchBacklinks();
    }
  }

  async function fetchBacklinks() {
    // Prevent fetching the same path multiple times
    if (isLoading || lastFetchedPath === props.relativePath) {
      return;
    }

    try {
      isLoading = true;
      error = "";
      backlinks = [];
      debugInfo = `Fetching backlinks for: ${props.relativePath}`;

      // Get vault path from settings if not already loaded
      if (!vaultPath) {
        debugInfo += "\nLoading vault path from database...";
        const db = await Database.load("sqlite:settings.db");
        const result = await db.select(
          "SELECT value FROM settings WHERE key = $1",
          ["vault_path"],
        );

        if (result && result.length > 0) {
          vaultPath = result[0].value;
          debugInfo += `\nVault path loaded: ${vaultPath}`;
        } else {
          throw new Error("No vault path configured");
        }
      }

      // Call our Rust command to get backlinks
      if (props.relativePath) {
        debugInfo += `\nCalling Rust backend with relativePath=${props.relativePath}, vaultDirectory=${vaultPath}`;

        try {
          const result = await invoke("get_backlinks", {
            relativePath: props.relativePath,
            vaultDirectory: vaultPath,
          });

          backlinks = result as { relative_path: string; title: string }[];
          lastFetchedPath = props.relativePath; // Update the last fetched path
          debugInfo += `\nBacklinks returned: ${backlinks.length}`;
        } catch (invokeError) {
          debugInfo += `\nInvoke error: ${invokeError}`;
          throw invokeError;
        }
      }
    } catch (e) {
      console.error("Failed to load backlinks:", e);
      error = `Error loading backlinks: ${e}`;
      debugInfo += `\nError: ${e}`;
    } finally {
      isLoading = false;
    }
  }

  function navigateToNote(relativePath: string) {
    // Close panel before navigation to prevent loops
    isVisible = false;

    // Make sure the path is properly formatted with /note/ prefix
    const path = `/note/${encodeURIComponent(relativePath)}`;

    // Log the navigation for debugging
    console.log(`Navigating to: ${path}`);

    // Navigate to the note page
    goto(path);
  }
  // Only fetch when the relativePath changes and the panel is visible
  $effect(() => {
    const path = props.relativePath;
    if (isVisible && path && lastFetchedPath !== path) {
      fetchBacklinks();
    }
  });

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
    debugInfo = "Component mounted";
  });

  onDestroy(() => {
    window.removeEventListener("keydown", handleKeydown);
  });
</script>

<!-- The rest of the component remains the same -->

{#if isVisible}
  <div
    class="fixed left-4 top-1/4 z-40 w-72 shadow-lg rounded-xl bg-white border border-gray-100 overflow-hidden"
    transition:fly={{
      x: -100,
      duration: 250,
      easing: (t) => 1 - Math.pow(1 - t, 3),
    }}
  >
    <!-- Panel header -->
    <div
      class="bg-gradient-to-r from-blue-50 to-indigo-50 px-4 py-3 border-b border-gray-100 flex justify-between items-center"
    >
      <h2 class="text-sm font-medium text-gray-700">
        Backlinks
        {#if backlinks.length > 0}
          <span
            class="ml-2 text-xs font-normal text-blue-500 bg-blue-50 px-2 py-0.5 rounded-full"
          >
            {backlinks.length}
          </span>
        {/if}
      </h2>
      <button
        class="text-gray-400 hover:text-gray-600 transition-colors"
        on:click={toggleVisibility}
        aria-label="Close panel"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-4 w-4"
          viewBox="0 0 20 20"
          fill="currentColor"
        >
          <path
            fill-rule="evenodd"
            d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
            clip-rule="evenodd"
          />
        </svg>
      </button>
    </div>

    <!-- Panel content -->
    <div class="max-h-96 overflow-y-auto bg-white p-1">
      {#if isLoading}
        <div class="flex justify-center items-center p-6 opacity-75">
          <div
            class="w-5 h-5 border-2 border-blue-400 border-t-transparent rounded-full animate-spin"
          ></div>
        </div>
      {:else if error}
        <div class="p-4 text-xs text-red-500">
          {error}

          {#if debugInfo}
            <div
              class="mt-2 p-2 text-left bg-gray-50 rounded text-[10px] whitespace-pre-wrap text-gray-700"
            >
              {debugInfo}
            </div>
          {/if}
        </div>
      {:else if backlinks.length === 0}
        <div class="p-4 text-center text-xs text-gray-400">
          No
          <div class="p-4 text-center text-xs text-gray-400">
            No backlinks found for this note

            {#if debugInfo}
              <div
                class="mt-2 p-2 text-left bg-gray-50 rounded text-[10px] whitespace-pre-wrap text-gray-700"
              >
                {debugInfo}
              </div>
            {/if}
          </div>
        </div>
      {:else}
        <ul class="divide-y divide-gray-50">
          {#each backlinks as link}
            <li>
              <button
                on:click={() => navigateToNote(link.relative_path)}
                class="w-full text-left p-3 hover:bg-blue-50 transition-colors rounded-md flex items-start gap-2"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  class="h-4 w-4 mt-0.5 text-blue-400 flex-shrink-0"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fill-rule="evenodd"
                    d="M12.586 4.586a2 2 0 112.828 2.828l-3 3a2 2 0 01-2.828 0 1 1 0 00-1.414 1.414 4 4 0 005.656 0l3-3a4 4 0 00-5.656-5.656l-1.5 1.5a1 1 0 101.414 1.414l1.5-1.5zm-5 5a2 2 0 012.828 0 1 1 0 101.414-1.414 4 4 0 00-5.656 0l-3 3a4 4 0 105.656 5.656l1.5-1.5a1 1 0 10-1.414-1.414l-1.5 1.5a2 2 0 11-2.828-2.828l3-3z"
                    clip-rule="evenodd"
                  />
                </svg>
                <div>
                  <div class="text-sm font-medium text-gray-700">
                    {link.title || "Untitled"}
                  </div>
                  <div
                    class="text-xs text-gray-400 truncate mt-0.5 max-w-[15rem]"
                  >
                    {link.relative_path}
                  </div>
                </div>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    <!-- Keyboard shortcut hint -->
    <div
      class="bg-gray-50 px-3 py-2 text-xs text-center text-gray-400 border-t border-gray-100"
    >
      <kbd
        class="px-1.5 py-0.5 bg-white border border-gray-200 rounded text-[10px] mx-1"
        >Ctrl</kbd
      >
      <span class="mx-0.5">+</span>
      <kbd
        class="px-1.5 py-0.5 bg-white border border-gray-200 rounded text-[10px] mx-1"
        >B</kbd
      >
      <span class="mx-1">to toggle</span>
    </div>
  </div>
{/if}
