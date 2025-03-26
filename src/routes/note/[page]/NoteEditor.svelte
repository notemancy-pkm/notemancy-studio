<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { Carta, MarkdownEditor, Markdown } from "carta-md";
  import ToC from "./ToC.svelte";
  import { addToast } from "$lib/Toaster.svelte";

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
  let saveSuccess = $state(false);
  let saveSuccessTimeout: ReturnType<typeof setTimeout>;

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

      // If we're exiting edit mode and there are unsaved changes, save first
      if (isEditMode && isDirty) {
        saveNote().then(() => {
          toggleEditMode();
        });
      } else {
        toggleEditMode();
      }
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

      // Show success message briefly
      saveSuccess = true;
      addToast({
        data: {
          title: "Saved",
          description: "",
          color: "green",
        },
      });

      if (props.onSave) props.onSave();
    } catch (e) {
      console.error("Failed to save note:", e);
      saveError = `Error saving note: ${e}`;
      addToast({
        data: {
          title: "Save Error",
          description: saveError,
          color: "red",
        },
      });
    } finally {
      saving = false;
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener("keydown", handleKeydown);
    if (saveSuccessTimeout) clearTimeout(saveSuccessTimeout);
  });
</script>

<div class="relative">
  {#if isDirty}
    <hr class="border-[#ff7eb6] mb-16" />
  {:else}
    <hr class="border-blue-400 mb-16" />
  {/if}

  <!-- Status Messages -->
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
  {:else if saveSuccess}
    <div
      class="absolute top-0 left-0 m-2 p-2 bg-green-100 text-green-800 rounded shadow-sm"
    >
      Saved successfully
    </div>
  {/if}

  {#if !isEditMode}
    <div
      class="fixed top-[5svh] right-0 flex w-[350px] justify-end p-2 text-xs"
    >
      <ToC contentSelector="#mdcontent" />
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
        id="mdcontent"
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
</div>
