<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { Carta, MarkdownEditor, Markdown } from "carta-md";
  import "carta-md/default.css";
  import DOMPurify from "isomorphic-dompurify";
  import { getCartaInstance } from "./getCarta";
  import "./tw.css";

  // Props
  const props = $props<{
    content: string;
    relativePath: string;
    vaultPath: string;
    onSave?: () => void;
  }>();

  // State
  let isEditMode = $state(false);
  let editorContent = $state(props.content || "");
  let isDirty = $state(false);
  let saving = $state(false);
  let saveError = $state("");

  // Get carta instance
  const carta = getCartaInstance("light");

  // Track if content has changed
  $effect(() => {
    isDirty = editorContent !== props.content;
  });

  // Handle keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    // Ctrl+L to toggle edit mode
    if (event.key === "l" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      toggleEditMode();
    }

    // Ctrl+S to save
    if (event.key === "s" && (event.ctrlKey || event.metaKey) && isEditMode) {
      event.preventDefault();
      saveNote();
    }
  }

  // Toggle between edit and preview mode
  function toggleEditMode() {
    isEditMode = !isEditMode;
  }

  // Save the note content
  async function saveNote() {
    if (!isDirty) return;

    try {
      saving = true;
      saveError = "";

      await invoke("update_note_content", {
        relativePath: props.relativePath,
        absolutePath: null,
        vaultDirectory: props.vaultPath,
        newContent: editorContent,
      });

      isDirty = false;
      if (props.onSave) props.onSave();
    } catch (e) {
      console.error("Failed to save note:", e);
      saveError = `Error saving note: ${e}`;
    } finally {
      saving = false;
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener("keydown", handleKeydown);
  });
</script>

<div class="relative">
  <!-- 
  <button
    class="absolute top-0 right-0 z-10 p-2 m-2 bg-white/80 hover:bg-white text-gray-600 rounded shadow-sm"
    on:click={toggleEditMode}
  >
    {#if isEditMode}
      <span class="flex items-center">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-5 w-5 mr-1"
          viewBox="0 0 20 20"
          fill="currentColor"
        >
          <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
          <path
            fill-rule="evenodd"
            d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z"
            clip-rule="evenodd"
          />
        </svg>
        View
      </span>
    {:else}
      <span class="flex items-center">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-5 w-5 mr-1"
          viewBox="0 0 20 20"
          fill="currentColor"
        >
          <path
            d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z"
          />
        </svg>
        Edit
      </span>
    {/if}
  </button>-->

  <!-- Save Status Messages -->
  {#if saveError}
    <div
      class="absolute top-0 left-0 m-2 p-2 bg-red-100 text-red-800 rounded shadow-sm"
    >
      {saveError}
    </div>
  {:else if saving}
    <div
      class="absolute top-0 left-0 m-2 p-2 bg-blue-100 text-blue-800 rounded shadow-sm"
    >
      Saving...
    </div>
  {:else if isDirty && isEditMode}
    <div
      class="absolute top-0 left-0 m-2 p-2 bg-yellow-100 text-yellow-800 rounded shadow-sm"
    >
      Unsaved changes
    </div>
  {/if}

  <!-- Editor / Preview Container -->
  <div class="min-h-[400px]">
    {#if isEditMode}
      <div class="rounded-md">
        <MarkdownEditor
          {carta}
          bind:value={editorContent}
          disableToolbar={true}
          scroll={"sync"}
          theme="tw"
          mode={"tabs"}
        />
      </div>
    {:else}
      <div
        class="prose prose-base prose-headings:font-[Noto_Sans] mx-auto max-w-[784px] pb-16 font-[IBM_Plex_Serif] text-gray-700"
      >
        <Markdown
          {carta}
          value={editorContent !== props.content
            ? editorContent
            : props.content}
        />
      </div>
    {/if}
  </div>

  <!-- Save Button (only in edit mode) -->
  {#if isEditMode}
    <div class="mt-4 flex justify-end">
      <button
        on:click={saveNote}
        disabled={!isDirty || saving}
        class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {saving ? "Saving..." : "Save"}
      </button>
    </div>
  {/if}
</div>
