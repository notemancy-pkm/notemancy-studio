<script lang="ts">
  import { onMount } from "svelte";
  import KanbanCardModal from "./KanbanCardModal.svelte";
  import type { CardData } from "./KanbanCardModal.svelte";

  // Define types for our Kanban data
  interface KanbanCard {
    id: string;
    title: string;
    description?: string;
  }

  interface KanbanColumn {
    id: string;
    title: string;
    cards: KanbanCard[];
  }

  // State
  let columns = $state<KanbanColumn[]>([
    { id: "col-1", title: "To Do", cards: [] },
    { id: "col-2", title: "In Progress", cards: [] },
    { id: "col-3", title: "Done", cards: [] },
  ]);

  let newColumnTitle = $state("");
  let isAddingColumn = $state(false);
  let editingColumnId = $state<string | null>(null);
  let editColumnTitle = $state("");

  // Card modal state
  let showCardModal = $state(false);
  let cardModalData = $state<CardData | undefined>(undefined);
  let isEditingCard = $state(false);
  let currentColumnId = $state("");
  let draggedCard = $state<{ card: KanbanCard; columnId: string } | null>(null);

  // Add a new column
  function addColumn() {
    if (newColumnTitle.trim() === "") return;

    const newColumn: KanbanColumn = {
      id: `col-${Date.now()}`,
      title: newColumnTitle,
      cards: [],
    };

    columns = [...columns, newColumn];
    newColumnTitle = "";
    isAddingColumn = false;
  }

  // Start editing a column title
  function startEditingColumn(column: KanbanColumn) {
    editingColumnId = column.id;
    editColumnTitle = column.title;
  }

  // Save column title edit
  function saveColumnEdit() {
    if (!editingColumnId) return;

    columns = columns.map((col) => {
      if (col.id === editingColumnId) {
        return { ...col, title: editColumnTitle };
      }
      return col;
    });

    editingColumnId = null;
  }

  // Cancel column title edit
  function cancelColumnEdit() {
    editingColumnId = null;
  }

  // Delete a column
  function deleteColumn(columnId: string) {
    if (confirm("Are you sure you want to delete this column?")) {
      columns = columns.filter((col) => col.id !== columnId);
    }
  }

  // Open the card modal to add a new card
  function openAddCardModal(columnId: string) {
    currentColumnId = columnId;
    cardModalData = undefined;
    isEditingCard = false;
    showCardModal = true;
  }

  // Open the card modal to edit a card
  function openEditCardModal(card: KanbanCard, columnId: string) {
    currentColumnId = columnId;
    cardModalData = {
      id: card.id,
      title: card.title,
      description: card.description || "",
    };
    isEditingCard = true;
    showCardModal = true;
  }

  // Save a new or edited card
  function saveCard(event: CustomEvent<CardData>) {
    const cardData = event.detail;

    if (isEditingCard) {
      // Update existing card
      columns = columns.map((col) => {
        if (col.id === currentColumnId) {
          return {
            ...col,
            cards: col.cards.map((card) =>
              card.id === cardData.id ? { ...cardData } : card,
            ),
          };
        }
        return col;
      });
    } else {
      // Add new card
      columns = columns.map((col) => {
        if (col.id === currentColumnId) {
          return {
            ...col,
            cards: [...col.cards, cardData],
          };
        }
        return col;
      });
    }

    showCardModal = false;
  }

  // Delete a card
  function deleteCard(cardId: string, columnId: string) {
    if (confirm("Are you sure you want to delete this card?")) {
      columns = columns.map((col) => {
        if (col.id === columnId) {
          return {
            ...col,
            cards: col.cards.filter((card) => card.id !== cardId),
          };
        }
        return col;
      });
    }
  }

  // Drag and drop handling
  function handleDragStart(card: KanbanCard, columnId: string) {
    draggedCard = { card, columnId };
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
  }

  function handleDrop(columnId: string) {
    if (!draggedCard) return;

    // Don't do anything if dropping in the same column
    if (draggedCard.columnId === columnId) return;

    // Remove card from source column
    columns = columns.map((col) => {
      if (col.id === draggedCard!.columnId) {
        return {
          ...col,
          cards: col.cards.filter((card) => card.id !== draggedCard!.card.id),
        };
      }
      return col;
    });

    // Add card to target column
    columns = columns.map((col) => {
      if (col.id === columnId) {
        return {
          ...col,
          cards: [...col.cards, draggedCard!.card],
        };
      }
      return col;
    });

    // Reset dragged card
    draggedCard = null;
  }
</script>

<div class="p-6">
  <div class="mb-6 flex justify-between items-center">
    <h2 class="text-xl font-semibold text-gray-800">Kanban Board</h2>

    {#if !isAddingColumn}
      <button
        on:click={() => (isAddingColumn = true)}
        class="px-3 py-1.5 bg-blue-500 text-white rounded-md text-sm hover:bg-blue-600 transition-colors flex items-center gap-1"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-4 w-4"
          viewBox="0 0 20 20"
          fill="currentColor"
        >
          <path
            fill-rule="evenodd"
            d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z"
            clip-rule="evenodd"
          />
        </svg>
        Add Column
      </button>
    {:else}
      <div class="flex items-center gap-2">
        <input
          type="text"
          bind:value={newColumnTitle}
          placeholder="Column title"
          class="px-3 py-1.5 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-400 transition-all"
        />
        <button
          on:click={addColumn}
          class="px-3 py-1.5 bg-blue-500 text-white rounded-md text-sm hover:bg-blue-600 transition-colors"
        >
          Add
        </button>
        <button
          on:click={() => (isAddingColumn = false)}
          class="px-3 py-1.5 bg-gray-200 text-gray-700 rounded-md text-sm hover:bg-gray-300 transition-colors"
        >
          Cancel
        </button>
      </div>
    {/if}
  </div>

  <div class="flex gap-4 overflow-x-auto pb-4">
    {#each columns as column}
      <div class="flex-shrink-0 w-sm bg-gray-50 shadow-sm">
        <!-- Column header -->
        <div
          class="p-3 border-b border-blue-400 flex items-center justify-between"
        >
          {#if editingColumnId === column.id}
            <div class="flex items-center gap-1 w-full">
              <input
                type="text"
                bind:value={editColumnTitle}
                class="px-2 py-1 border border-gray-300 rounded-md text-sm w-full focus:outline-none focus:ring-2 focus:ring-blue-400 transition-all"
              />
              <button
                on:click={saveColumnEdit}
                class="p-1 text-blue-500 hover:text-blue-700"
                title="Save"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  class="h-4 w-4"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fill-rule="evenodd"
                    d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                    clip-rule="evenodd"
                  />
                </svg>
              </button>
              <button
                on:click={cancelColumnEdit}
                class="p-1 text-gray-500 hover:text-gray-700"
                title="Cancel"
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
          {:else}
            <div class="font-medium text-gray-700">{column.title}</div>
            <div class="flex items-center">
              <button
                on:click={() => startEditingColumn(column)}
                class="p-1 text-gray-400 hover:text-gray-700"
                title="Edit column title"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  class="h-4 w-4"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z"
                  />
                </svg>
              </button>
              <button
                on:click={() => deleteColumn(column.id)}
                class="p-1 text-gray-400 hover:text-red-500"
                title="Delete column"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  class="h-4 w-4"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fill-rule="evenodd"
                    d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z"
                    clip-rule="evenodd"
                  />
                </svg>
              </button>
            </div>
          {/if}
        </div>

        <!-- Cards container -->
        <div
          class="p-2 min-h-[80svh] overflow-y-auto"
          on:dragover={handleDragOver}
          on:drop={() => handleDrop(column.id)}
        >
          {#each column.cards as card}
            <div
              class="mb-2 p-3 bg-white rounded-md shadow-sm cursor-pointer hover:shadow-md transition-shadow group"
              draggable="true"
              on:dragstart={() => handleDragStart(card, column.id)}
              on:click={() => openEditCardModal(card, column.id)}
            >
              <div class="flex justify-between">
                <div class="font-medium text-gray-800">{card.title}</div>
                <div
                  class="opacity-0 group-hover:opacity-100 transition-opacity"
                >
                  <button
                    on:click|stopPropagation={() =>
                      deleteCard(card.id, column.id)}
                    class="p-1 text-gray-400 hover:text-red-500 rounded"
                    title="Delete card"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      class="h-3.5 w-3.5"
                      viewBox="0 0 20 20"
                      fill="currentColor"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z"
                        clip-rule="evenodd"
                      />
                    </svg>
                  </button>
                </div>
              </div>
              {#if card.description}
                <div
                  class="mt-1 text-xs text-gray-500 line-clamp-2 overflow-hidden"
                >
                  {card.description}
                </div>
              {/if}
            </div>
          {/each}

          <!-- Add card button -->
          <button
            on:click={() => openAddCardModal(column.id)}
            class="w-full p-2 text-sm text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-md flex items-center justify-center gap-1 transition-colors"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-4 w-4"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fill-rule="evenodd"
                d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z"
                clip-rule="evenodd"
              />
            </svg>
            Add a card
          </button>
        </div>
      </div>
    {/each}
  </div>

  <!-- Card Modal -->
  <KanbanCardModal
    show={showCardModal}
    card={cardModalData}
    isEdit={isEditingCard}
    on:save={saveCard}
    on:close={() => (showCardModal = false)}
  />
</div>
