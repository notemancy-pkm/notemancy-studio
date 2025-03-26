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
  let contentHeight = $state(0);
  let contentElement: Element | null = null;

  // Function to scroll to a heading when clicked
  function scrollToHeading(id: string, event: MouseEvent) {
    event.preventDefault();
    const element = document.getElementById(id);
    if (element) {
      element.scrollIntoView({ behavior: "smooth" });
      currentId = id;
    }
  }

  // Compute vertical line segments with gaps
  function computeSegments(sortedHeadings: Heading[]) {
    if (sortedHeadings.length === 0) return [];

    const newSegments: Segment[] = [];
    const gapPx = 9;
    const viewportHeight = window.innerHeight;
    const gapPercent = (gapPx / viewportHeight) * 100;
    const halfGap = gapPercent / 2;

    // Segment from top of container to just before the first dot
    const firstSegmentHeight = sortedHeadings[0].pos - halfGap;
    if (firstSegmentHeight > 0) {
      newSegments.push({ top: 0, height: firstSegmentHeight });
    }

    // Segments between successive dots
    for (let i = 0; i < sortedHeadings.length - 1; i++) {
      const start = sortedHeadings[i].pos + halfGap;
      const end = sortedHeadings[i + 1].pos - halfGap;
      if (end > start) {
        newSegments.push({ top: start, height: end - start });
      }
    }

    // Segment from the last dot to the bottom
    const lastSegmentTop =
      sortedHeadings[sortedHeadings.length - 1].pos + halfGap;
    if (lastSegmentTop < 100) {
      newSegments.push({ top: lastSegmentTop, height: 100 - lastSegmentTop });
    }

    return newSegments;
  }

  // Get active segments for the current heading
  function getActiveSegments(sortedHeadings: Heading[], activeId: string) {
    const activeSegments: Segment[] = [];
    if (!activeId || sortedHeadings.length === 0) return activeSegments;

    const activeIndex = sortedHeadings.findIndex((h) => h.id === activeId);
    if (activeIndex === -1) return activeSegments;

    const gapPx = 9;
    const viewportHeight = window.innerHeight;
    const gapPercent = (gapPx / viewportHeight) * 100;
    const halfGap = gapPercent / 2;

    // If this is the first heading, highlight from top to this heading
    if (activeIndex === 0) {
      activeSegments.push({
        top: 0,
        height: sortedHeadings[0].pos - halfGap,
      });
    } else {
      // Highlight segment from previous heading to this one
      const prevHeading = sortedHeadings[activeIndex - 1];
      activeSegments.push({
        top: prevHeading.pos + halfGap,
        height: sortedHeadings[activeIndex].pos - prevHeading.pos - gapPercent,
      });
    }

    // If this is the last heading, highlight from this heading to bottom
    if (activeIndex === sortedHeadings.length - 1) {
      activeSegments.push({
        top: sortedHeadings[activeIndex].pos + halfGap,
        height: 100 - (sortedHeadings[activeIndex].pos + halfGap),
      });
    } else {
      // Highlight segment from this heading to next one
      const nextHeading = sortedHeadings[activeIndex + 1];
      activeSegments.push({
        top: sortedHeadings[activeIndex].pos + halfGap,
        height: nextHeading.pos - sortedHeadings[activeIndex].pos - gapPercent,
      });
    }

    return activeSegments;
  }

  async function updateToC() {
    // Disconnect existing observer
    if (observer) {
      observer.disconnect();
    }

    // Wait for Markdown content to render
    await tick();

    contentElement = document.querySelector(contentSelector);
    if (!contentElement) {
      console.warn(`Content container '${contentSelector}' not found`);
      return;
    }

    contentHeight = contentElement.scrollHeight;

    const headingElements = contentElement.querySelectorAll(
      "h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]",
    );

    // If no headings with ids found, try to create ids for headings
    if (headingElements.length === 0) {
      const allHeadings = contentElement.querySelectorAll(
        "h1, h2, h3, h4, h5, h6",
      );
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
    const updatedHeadingElements = contentElement.querySelectorAll(
      "h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]",
    );

    // Create a new headings array
    const newHeadings: Heading[] = Array.from(updatedHeadingElements).map(
      (el: Element) => {
        const level = parseInt(el.tagName.substring(1));
        const text = el.textContent?.trim() || "";
        const rect = el.getBoundingClientRect();
        const top = rect.top + window.scrollY;
        const pos = (top / contentHeight) * 100;
        return { id: el.getAttribute("id") || "", text, level, pos };
      },
    );

    // Update headings state once
    headings = newHeadings;

    // Sort headings by position for segment calculation
    const sortedHeadings = [...newHeadings].sort((a, b) => a.pos - b.pos);

    // Calculate segments once
    segments = computeSegments(sortedHeadings);

    // Set up intersection observer for heading visibility tracking
    const observerOptions = {
      rootMargin: "0px 0px -50% 0px",
      threshold: 0.1,
    };

    observer = new IntersectionObserver((entries) => {
      // Find the topmost visible heading
      const visibleEntries = entries.filter((entry) => entry.isIntersecting);
      if (visibleEntries.length === 0) return;

      // Sort visible entries by their position in the viewport
      visibleEntries.sort((a, b) => {
        return a.boundingClientRect.top - b.boundingClientRect.top;
      });

      // Set current ID to the topmost visible heading
      const newCurrentId = (visibleEntries[0].target as HTMLElement).id;
      if (newCurrentId !== currentId) {
        currentId = newCurrentId;
      }
    }, observerOptions);

    // Observe all heading elements
    updatedHeadingElements.forEach((el) => observer!.observe(el));
  }

  // Handle window resize to recalculate positions
  function handleResize() {
    // Debounce the resize handler to avoid too many updates
    clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => {
      updateToC();
    }, 200);
  }

  let resizeTimer: ReturnType<typeof setTimeout>;

  onMount(() => {
    updateToC();
    window.addEventListener("resize", handleResize);
  });

  onDestroy(() => {
    if (observer) observer.disconnect();
    window.removeEventListener("resize", handleResize);
    clearTimeout(resizeTimer);
  });
</script>

<div class="relative h-[93svh] w-full toc-container">
  <!-- Gray background segments -->
  {#each segments as seg}
    <div
      class="absolute right-2 w-0.5 bg-gray-100"
      style="top: {seg.top}%; height: {seg.height}%"
    ></div>
  {/each}

  <!-- Active blue segments for current heading -->
  {#if currentId && headings.length > 0}
    {#each getActiveSegments( [...headings].sort((a, b) => a.pos - b.pos), currentId, ) as activeSeg}
      <div
        class="absolute right-2 w-px bg-blue-500"
        style="top: {activeSeg.top}%; height: {activeSeg.height}%"
      ></div>
    {/each}
  {/if}

  <!-- Render headings and their corresponding dots -->
  {#each headings as heading}
    <a
      href={"#" + heading.id}
      on:click={(e) => scrollToHeading(heading.id, e)}
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
