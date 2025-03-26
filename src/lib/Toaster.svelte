<script lang="ts" context="module">
  import { flip } from "svelte/animate";
  import { fly } from "svelte/transition";
  import Icon from "@iconify/svelte";

  export type ToastData = {
    title: string;
    description: string;
    color: string;
  };

  const {
    elements: { content, title, description, close },
    helpers,
    states: { toasts },
    actions: { portal },
  } = createToaster<ToastData>();

  export const addToast = helpers.addToast;
</script>

<script lang="ts">
  import { createToaster, melt } from "@melt-ui/svelte";
</script>

<div use:portal class="fixed bottom-4 right-4">
  {#each $toasts as { id, data } (id)}
    <div
      use:melt={$content(id)}
      animate:flip={{ duration: 500 }}
      in:fly={{ duration: 150, x: "100%" }}
      out:fly={{ duration: 150, x: "100%" }}
      class="bg-gray-50 p-4 rounded-md shadow-md"
    >
      <div class="relative w-[250px]">
        <div class="flex items-center w-full gap-4">
          {#if data.color === "green"}
            <Icon
              icon="charm:circle-tick"
              width="20"
              height="20"
              color={data.color}
            />
          {:else}
            <Icon
              icon="basil:cross-outline"
              width="20"
              height="20"
              color={data.color}
            />
          {/if}
          <div class="flex flex-col items-start">
            <h3 use:melt={$title(id)}>
              {data.title}
            </h3>
            {#if data.description !== ""}
              <div use:melt={$description(id)}>
                {data.description}
              </div>
            {/if}
          </div>
        </div>
        <button
          use:melt={$close(id)}
          aria-label="close notification"
          class="absolute top-0 right-0"
        >
          <Icon
            icon="basil:cross-outline"
            width="20"
            height="20"
            color={data.color}
          />
        </button>
      </div>
    </div>
  {/each}
</div>
