<script lang="ts">
  import { onMount, tick, onDestroy } from "svelte";

  // Props using Svelte 5 runes
  const props = $props<{
    contentSelector?: string;
  }>();

  // Set default values
  const contentSelector = props.contentSelector ?? ".prose";

  interface Heading {
    id: string;
    text: string;
    level: number;
    // pos is the vertical position (percentage) relative to the content container's height
    pos: number;
  }

  interface Segment {
    top: number;
    height: number;
  }

  let headings = $state<Heading[]>([]);
  let segments = $state<Segment[]>([]);
  let currentId = $state("");
  let observer: IntersectionObserver | null = null;
  // Create a sorted headings variable to avoid repeated sorting
  let sortedHeadings = $state<Heading[]>([]);

  // Compute vertical line segments with gaps
  function computeSegments() {
    segments = [];
    if (headings.length === 0) return;

    const gapPx = 9;
    const viewportHeight = window.innerHeight;
    const gapPercent = (gapPx / viewportHeight) * 100;
    const halfGap = gapPercent / 2;

    // Use the pre-sorted headings instead of sorting again
    const sorted = sortedHeadings;

    // Segment from top of container to just before the first dot
    const firstSegmentHeight = sorted[0].pos - halfGap;
    if (firstSegmentHeight > 0) {
      segments.push({ top: 0, height: firstSegmentHeight });
    }

    // Segments between successive dots
    for (let i = 0; i < sorted.length - 1; i++) {
      const start = sorted[i].pos + halfGap;
      const end = sorted[i + 1].pos - halfGap;
      if (end > start) {
        segments.push({ top: start, height: end - start });
      }
    }

    // Segment from the last dot to the bottom
    const lastSegmentTop = sorted[sorted.length - 1].pos + halfGap;
    if (lastSegmentTop < 100) {
      segments.push({ top: lastSegmentTop, height: 100 - lastSegmentTop });
    }
  }

  async function updateToC() {
    if (observer) {
      observer.disconnect();
    }

    // Wait for Markdown content to render
    await tick();

    const content = document.querySelector(contentSelector);
    if (!content) {
      console.warn(`Content container '${contentSelector}' not found`);
      return;
    }

    const contentHeight = content.scrollHeight;

    const headingElements = content.querySelectorAll(
      "h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]",
    );

    // If no headings with ids found, try to create ids for headings
    if (headingElements.length === 0) {
      const allHeadings = content.querySelectorAll("h1, h2, h3, h4, h5, h6");
      allHeadings.forEach((el, index) => {
        if (!el.id) {
          const text = el.textContent?.trim() || "";
          const slug = text
            .toLowerCase()
            .replace(/[^\w\s-]/g, "")
            .replace(/\s+/g, "-");
          el.id = `heading-${slug || index}`;
        }
      });
    }

    // Try again with the updated headings
    const updatedHeadingElements = content.querySelectorAll(
      "h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]",
    );

    // Create a new headings array
    const newHeadings = Array.from(updatedHeadingElements).map(
      (el: Element) => {
        const level = parseInt(el.tagName.substring(1));
        const text = el.textContent?.trim() || "";
        const rect = el.getBoundingClientRect();
        const top = rect.top + window.scrollY;
        const pos = (top / contentHeight) * 100;
        return { id: el.getAttribute("id") || "", text, level, pos };
      },
    );

    // Update headings and sortedHeadings in one go to avoid multiple updates
    headings = newHeadings;
    sortedHeadings = [...newHeadings].sort((a, b) => a.pos - b.pos);

    computeSegments();

    const visibleHeadings = new Map<string, IntersectionObserverEntry>();
    const observerOptions = { rootMargin: "0px 0px -80% 0px" };

    observer = new IntersectionObserver((entries) => {
      entries.forEach((entry) => {
        const id = (entry.target as HTMLElement).id;
        if (entry.isIntersecting) {
          visibleHeadings.set(id, entry);
        } else {
          visibleHeadings.delete(id);
        }
      });

      let minEntry: IntersectionObserverEntry | null = null;
      visibleHeadings.forEach((entry) => {
        if (
          !minEntry ||
          entry.boundingClientRect.top < minEntry.boundingClientRect.top
        ) {
          minEntry = entry;
        }
      });

      if (minEntry) {
        currentId = (minEntry.target as HTMLElement).id;
      }
    }, observerOptions);

    updatedHeadingElements.forEach((el) => observer!.observe(el));
  }

  onMount(() => {
    updateToC();

    // Add window resize event listener to recalculate positions
    window.addEventListener("resize", updateToC);
  });

  onDestroy(() => {
    if (observer) observer.disconnect();
    window.removeEventListener("resize", updateToC);
  });

  // Update sortedHeadings whenever headings or currentId changes
  // $effect(() => {
  //   if (headings.length > 0) {
  //     // Only update sortedHeadings if needed
  //     sortedHeadings = [...headings].sort((a, b) => a.pos - b.pos);
  //     computeSegments();
  //   }
  // });

  $effect(() => {
    currentId;
    computeSegments();
  });
</script>

<div class="relative h-[93svh] w-full toc-container">
  <!-- Render gray vertical line segments with gaps -->
  {#each segments as seg}
    <div
      class="absolute right-2 w-0.5 bg-gray-100"
      style="top: {seg.top}%; height: {seg.height}%"
    ></div>
  {/each}

  <!-- Render active blue segments adjacent to the current heading -->
  {#if headings.length > 0 && sortedHeadings.length > 0}
    {#if currentId}
      {#each headings as heading, i}
        {#if heading.id === currentId}
          <!-- Active segments -->
          {#if i === 0 || sortedHeadings[0].id === heading.id}
            <!-- For the first heading, highlight from the top to the dot -->
            <div
              class="absolute right-2 w-px bg-blue-500"
              style="top: 0%; height: {heading.pos -
                (9 / (2 * window.innerHeight)) * 100}%"
            ></div>
          {:else}
            <!-- For non-first headings, highlight the segment above the current dot -->
            {@const currentIndex = sortedHeadings.findIndex(
              (h) => h.id === heading.id,
            )}
            {@const prevHeading = sortedHeadings[currentIndex - 1]}
            <div
              class="absolute right-2 w-px bg-blue-500"
              style="top: {prevHeading.pos +
                (9 / (2 * window.innerHeight)) * 100}%; 
                    height: {heading.pos -
                (9 / (2 * window.innerHeight)) * 100 -
                (prevHeading.pos + (9 / (2 * window.innerHeight)) * 100)}%"
            ></div>
          {/if}

          {#if i === headings.length - 1 || sortedHeadings[sortedHeadings.length - 1].id === heading.id}
            <!-- For the last heading, highlight from the dot to the bottom -->
            <div
              class="absolute right-2 w-px bg-blue-500"
              style="top: {heading.pos +
                (9 / (2 * window.innerHeight)) * 100}%; 
                     height: {100 -
                (heading.pos + (9 / (2 * window.innerHeight)) * 100)}%"
            ></div>
          {:else}
            <!-- For non-last headings, highlight the segment below the current dot -->
            {@const currentIndex = sortedHeadings.findIndex(
              (h) => h.id === heading.id,
            )}
            {@const nextHeading = sortedHeadings[currentIndex + 1]}
            <div
              class="absolute right-2 w-px bg-blue-500"
              style="top: {heading.pos +
                (9 / (2 * window.innerHeight)) * 100}%; 
                     height: {nextHeading.pos -
                (9 / (2 * window.innerHeight)) * 100 -
                (heading.pos + (9 / (2 * window.innerHeight)) * 100)}%"
            ></div>
          {/if}
        {/if}
      {/each}
    {/if}
  {/if}

  <!-- Render headings and their corresponding dots -->
  {#each headings as heading}
    <a
      href={"#" + heading.id}
      class={`absolute no-underline ${heading.id === currentId ? "text-blue-500" : "text-gray-300"}`}
      style="top: {heading.pos}%; left: {(heading.level - 1) *
        20}px; transform: translateY(-50%);"
    >
      {heading.text}
    </a>
    <span
      class={`absolute rounded-full ${heading.id === currentId ? "bg-blue-500" : "bg-gray-300"}`}
      style="height: 0.2rem; width: 0.2rem; top: {heading.pos}%; right: 7.5px; transform: translateY(-50%);"
    ></span>
  {/each}
</div>

<style>
  .toc-container a {
    opacity: 0;
    transition: opacity 0.3s ease;
    font-size: 0.85rem;
  }

  .toc-container:hover a {
    opacity: 1;
  }
</style>
