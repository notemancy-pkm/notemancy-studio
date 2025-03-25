<script lang="ts">
  import { createDialog, melt } from "@melt-ui/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Database from "@tauri-apps/plugin-sql";
  import { onMount, onDestroy } from "svelte";
  import { fade } from "svelte/transition";

  let db: any;
  let vaultPath = $state("");
  let isLoading = $state(false);
  let error = $state("");
  let success = $state("");

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

  // Handle keyboard shortcut (Ctrl + ,)
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "," && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      open.set(true);
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

  async function saveSettings() {
    try {
      isLoading = true;
      error = "";
      success = "";

      // Check and create directory if needed
      const dirResult = await invoke("check_and_create_directory", {
        path: vaultPath,
      });

      // Save to database directly using the SQL plugin
      await db.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ($1, $2)",
        ["vault_path", vaultPath],
      );

      success = "Settings saved successfully!";
      setTimeout(() => {
        open.set(false);
        success = "";
      }, 1500);
    } catch (e) {
      error = `Error: ${e}`;
    } finally {
      isLoading = false;
    }
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
<button use:melt={$trigger} class="hidden"> Open Settings </button>

{#if $open}
  <div use:melt={$portalled}>
    <div
      use:melt={$overlay}
      class="fixed inset-0 z-50 bg-black/50"
      transition:fade={{ duration: 150 }}
    />
    <div
      class="fixed left-1/2 top-1/2 z-50 max-h-[85vh] w-[90vw]
            max-w-[450px] -translate-x-1/2 -translate-y-1/2 rounded-xl bg-white
            p-6 shadow-lg"
      transition:flyAndScale={{
        duration: 150,
        y: 8,
        start: 0.96,
      }}
      use:melt={$content}
    >
      <h2 use:melt={$title} class="m-0 text-lg font-medium text-black">
        Settings
      </h2>
      <p use:melt={$description} class="mb-5 mt-2 leading-normal text-zinc-600">
        Configure your Notemancy Studio settings here.
      </p>

      {#if error}
        <div class="mb-4 p-2 bg-red-100 text-red-800 rounded">
          {error}
        </div>
      {/if}

      {#if success}
        <div class="mb-4 p-2 bg-green-100 text-green-800 rounded">
          {success}
        </div>
      {/if}

      <fieldset class="mb-4 flex items-center gap-5">
        <label class="w-[90px] text-right text-black" for="vault-path">
          Vault Path
        </label>
        <input
          class="inline-flex h-8 w-full flex-1 items-center justify-center
                  rounded-sm border border-solid px-3 leading-none text-black"
          id="vault-path"
          placeholder="/path/to/your/vault"
          bind:value={vaultPath}
        />
      </fieldset>

      <div class="mt-6 flex justify-end gap-4">
        <button
          use:melt={$close}
          class="inline-flex h-8 items-center justify-center rounded-sm
                  bg-zinc-100 px-4 font-medium leading-none text-zinc-600"
        >
          Cancel
        </button>
        <button
          on:click={saveSettings}
          disabled={isLoading}
          class="inline-flex h-8 items-center justify-center rounded-sm
                  bg-blue-100 px-4 font-medium leading-none text-blue-900
                  disabled:opacity-50"
        >
          {isLoading ? "Saving..." : "Save changes"}
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
