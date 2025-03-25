<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { Carta, Markdown } from "carta-md";
  import "carta-md/default.css";
  import DOMPurify from "isomorphic-dompurify";
  import { page } from "$app/stores";
  import Database from "@tauri-apps/plugin-sql";

  // Create Carta instance with sanitizer
  const carta = new Carta({
    sanitizer: DOMPurify.sanitize,
  });

  let noteContent = $state("");
  let noteTitle = $state("");
  let loading = $state(true);
  let error = $state("");
  let vaultPath = $state("");

  // Get the path parameter from the URL
  const relativePath = $page.params.page;

  async function fetchNoteContent() {
    try {
      loading = true;
      error = "";

      // Get vault path from settings
      const db = await Database.load("sqlite:settings.db");
      const result = await db.select(
        "SELECT value FROM settings WHERE key = $1",
        ["vault_path"],
      );

      if (result && result.length > 0) {
        vaultPath = result[0].value;

        // Get the note content
        // In src/routes/note/[path]/+page.svelte
        const content = await invoke("get_note_content", {
          relativePath: relativePath, // Make sure this matches the Rust parameter name
          vaultDirectory: vaultPath, // Make sure this matches the Rust parameter name
        });

        noteTitle = await invoke("get_note_title", {
          relativePath: relativePath, // Make sure this matches the Rust parameter name
          vaultDirectory: vaultPath, // Make sure this matches the Rust parameter name
        });

        noteContent = content;
      } else {
        error = "No vault path configured";
      }
    } catch (e) {
      console.error("Failed to load note:", e);
      error = `Error loading note: ${e}`;
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    fetchNoteContent();
  });
</script>

<main class="container mx-auto p-6 max-w-4xl">
  <!-- <div class="mb-4">
    <a href="/" class="text-blue-600 hover:underline flex items-center">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="h-4 w-4 mr-1"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M10 19l-7-7m0 0l7-7m-7 7h18"
        />
      </svg>
      Back to notes
    </a>
  </div> -->

  {#if loading}
    <div class="flex justify-center items-center h-64">
      <p class="text-gray-500">Loading note...</p>
    </div>
  {:else if error}
    <div class="bg-red-50 text-red-700 p-4 rounded-lg">
      <p>{error}</p>
    </div>
  {:else}
    <div class="bg-white p-6">
      <div
        class="prose prose-base prose-headings:font-[Noto_Sans] mx-auto max-w-[784px] pb-16 font-[IBM_Plex_Serif] text-gray-700"
      >
        <div class="mb-16">
          <h1 class="mb-2 font-semibold">
            {noteTitle}
          </h1>
          <hr class="border-blue-400" />
        </div>
        {#key noteContent}
          <Markdown {carta} value={noteContent} />
        {/key}
      </div>
    </div>
  {/if}
</main>

<style>
  /* Set monospace font for code blocks */

  @import url("https://fonts.googleapis.com/css2?family=IBM+Plex+Serif:ital,wght@0,100;0,200;0,300;0,400;0,500;0,600;0,700;1,100;1,200;1,300;1,400;1,500;1,600;1,700&family=Noto+Sans+Mono:wght@100..900&family=Noto+Sans:ital,wght@0,100..900;1,100..900&family=Noto+Serif:ital,wght@0,100..900;1,100..900&family=Nunito:ital,wght@0,200..1000;1,200..1000&display=swap");

  @media print {
    .fixed {
      display: none !important;
    }
    body * {
      visibility: hidden;
    }
    #content,
    #content * {
      visibility: visible;
    }
    #content {
      position: absolute;
      left: 0;
      top: 0;
      width: 100%;
    }
  }
</style>
