<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { Carta, Markdown } from "carta-md";
  import "carta-md/default.css";
  import DOMPurify from "isomorphic-dompurify";
  import { page } from "$app/stores";
  import Database from "@tauri-apps/plugin-sql";
  import { getCartaInstance } from "./getCarta";
  import NoteEditor from "./NoteEditor.svelte";
  import ToC from "./ToC.svelte";
  import BacklinksPanel from "../../../components/BacklinksPanel.svelte";
  import { goto } from "$app/navigation";

  // Create Carta instance with sanitizer
  let carta = $state(getCartaInstance("light"));

  let noteContent = $state("");
  let noteTitle = $state("");
  let loading = $state(true);
  let error = $state("");
  let vaultPath = $state("");

  // Get the path parameter from the URL
  let relativePath = $state("");
  $effect(() => {
    relativePath = $page.params.page;
    fetchNoteContent();
  });

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
        const content = await invoke("get_note_content", {
          relativePath: relativePath,
          vaultDirectory: vaultPath,
        });

        noteTitle = await invoke("get_note_title", {
          relativePath: relativePath,
          vaultDirectory: vaultPath,
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

  // Refresh content after save
  async function handleNoteSaved() {
    await fetchNoteContent();
  }

  // Handle navigation from backlinks
  function handleNavigate(event) {
    const { relativePath } = event.detail;
    goto(`/note/${encodeURIComponent(relativePath)}`);
  }

  onMount(() => {
    fetchNoteContent();
  });
</script>

<main class="container mx-auto p-6 max-w-4xl">
  <!-- Add the Backlinks component -->
  <BacklinksPanel {relativePath} on:navigate={handleNavigate} />

  {#if loading}
    <div class="flex justify-center items-center h-64">
      <p class="text-gray-500">Loading note...</p>
    </div>
  {:else if error}
    <div class="bg-red-50 text-red-700 p-4 rounded-lg">
      <p>{error}</p>
    </div>
  {:else}
    <div class=" p-6">
      <div class="mx-auto max-w-[784px] pb-16" id="mdcontent">
        <div class="mb-16 prose prose-md mx-auto">
          <h1 class="mb-2 font-semibold font-[Noto_Sans] text-center">
            {noteTitle}
          </h1>
        </div>

        <!-- NoteEditor component -->
        <NoteEditor
          content={noteContent}
          {relativePath}
          {vaultPath}
          onSave={handleNoteSaved}
        />
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
