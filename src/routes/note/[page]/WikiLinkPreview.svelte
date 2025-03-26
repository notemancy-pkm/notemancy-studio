<script lang="ts">
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import { createLinkPreview, melt } from "@melt-ui/svelte";
  import { Slot } from "@cartamd/plugin-component/svelte";
  import min_dark from "shiki/themes/min-dark.mjs";
  import min_light from "shiki/themes/min-light.mjs";
  import { Carta, Markdown, type Plugin } from "carta-md";
  import { getCartaInstance } from "./getCarta";
  import everforest_dark from "shiki/themes/everforest-dark.mjs";

  // These props are passed in from Carta’s component mapping.
  let { href, children } = $props();

  let currentTheme: "dark" | "light" = $state("light");
  const carta = getCartaInstance(currentTheme);

  // Remove a leading slash if present.
  const normalizedHref = href.startsWith("/") ? href.slice(1) : href;

  // A promise that resolves to the markdown content.
  let contentPromise: Promise<string>;

  // Updated API endpoint: Call the new /notes/content?relpath= endpoint.
  async function getContent(relpath: string): Promise<string> {
    try {
      // Call the Rocket API's note content endpoint.
      const response = await fetch(
        `http://127.0.0.1:8000/notes/content?relpath=${encodeURIComponent(relpath)}`,
      );
      if (!response.ok) {
        throw new Error("Failed to fetch note content");
      }
      const data = await response.json();
      if (!data || typeof data !== "object" || !("content" in data)) {
        throw new Error("Invalid response format");
      }
      return data.content || "";
    } catch (error) {
      console.error("Error fetching note content:", error);
      throw error;
    }
  }

  // On mount, begin fetching the preview content using the normalized href.
  onMount(() => {
    contentPromise = getContent(normalizedHref);
  });

  // Set up Melt‑UI's link preview.
  const {
    elements: { trigger, content },
    states: { open },
  } = createLinkPreview();
</script>

<!-- The trigger link -->
<a
  {href}
  rel="noopener noreferrer"
  use:melt={$trigger}
  class="rounded-sm underline-offset-4 hover:underline focus-visible:outline-2 focus-visible:outline-offset-8 focus-visible:outline-black"
>
  <Slot />
</a>

{#if $open}
  <!-- The preview container is fixed to 500x500 and scrollable -->
  <div
    use:melt={$content}
    transition:fly={{ duration: 150, y: -8 }}
    class="border-muted shadow-popover mt-2 h-[500px] w-[500px] overflow-auto rounded-xl border bg-white p-4 dark:border-gray-300 dark:bg-gray-700"
  >
    <div
      class="prose lg:prose-base dark:prose-invert mt-0 h-full w-full pt-0 font-[Noto_Sans] font-normal"
    >
      <div
        class="mt-0 mb-4 border-b border-gray-300 pt-0 pb-3.5 text-[1.5rem] font-normal"
      >
        <a
          class="mt-0 pt-0 text-gray-800 hover:text-blue-600 dark:text-gray-50 dark:hover:text-blue-500"
          href="/"
        >
          Gnosis
        </a>
      </div>
      <div id="mdcontent" class="h-full w-full">
        {#await contentPromise}
          <p>Loading preview...</p>
        {:then content}
          {#key content}
            <Markdown {carta} value={content} />
          {/key}
        {:catch error}
          <p>Error loading preview: {error.message}</p>
        {/await}
      </div>
    </div>
  </div>
{/if}
