<script lang="ts">
  import KanbanCardModal from "./KanbanCardModal.svelte";
  import type { CardData } from "./KanbanCardModal.svelte";
  import { draggable } from "@neodrag/svelte";

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
  let columns: KanbanColumn[] = [
    { id: "col-1", title: "To Do", cards: [] },
    { id: "col-2", title: "In Progress", cards: [] },
    { id: "col-3", title: "Done", cards: [] },
  ];

  let newColumnTitle = "";
  let isAddingColumn = false;
  let editingColumnId: string | null = null;
  let editColumnTitle = "";

  // Card modal state
  let showCardModal = false;
  let cardModalData: CardData | undefined = undefined;
  let isEditingCard = false;
  let currentColumnId = "";

  // Column operations
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

  function startEditingColumn(column: KanbanColumn) {
    editingColumnId = column.id;
    editColumnTitle = column.title;
  }

  function saveColumnEdit() {
    if (!editingColumnId) return;
    columns = columns.map((col) =>
      col.id === editingColumnId ? { ...col, title: editColumnTitle } : col,
    );
    editingColumnId = null;
  }

  function cancelColumnEdit() {
    editingColumnId = null;
  }

  function deleteColumn(columnId: string) {
    if (confirm("Are you sure you want to delete this column?")) {
      columns = columns.filter((col) => col.id !== columnId);
    }
  }

  // Card operations
  function openAddCardModal(columnId: string) {
    currentColumnId = columnId;
    cardModalData = undefined;
    isEditingCard = false;
    showCardModal = true;
  }

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

  function saveCard(event: CustomEvent<CardData>) {
    const cardData = event.detail;
    if (isEditingCard) {
      columns = columns.map((col) =>
        col.id === currentColumnId
          ? {
              ...col,
              cards: col.cards.map((card) =>
                card.id === cardData.id ? { ...cardData } : card,
              ),
            }
          : col,
      );
    } else {
      columns = columns.map((col) =>
        col.id === currentColumnId
          ? { ...col, cards: [...col.cards, cardData] }
          : col,
      );
    }
    showCardModal = false;
  }

  function deleteCard(cardId: string, columnId: string) {
    if (confirm("Are you sure you want to delete this card?")) {
      columns = columns.map((col) =>
        col.id === columnId
          ? { ...col, cards: col.cards.filter((card) => card.id !== cardId) }
          : col,
      );
    }
  }

  function handleDragStart(event: CustomEvent) {
    const draggedEl = event.detail.currentNode as HTMLElement;
  }

  // When a card drag ends, decide where to drop it based on pointer location.
  function handleDragEnd(event: CustomEvent) {
    const { offsetX, offsetY, currentNode, event: pointerEvent } = event.detail;
    const dropX = pointerEvent.clientX;
    const dropY = pointerEvent.clientY;
    const draggedEl = currentNode as HTMLElement;
    const cardId = draggedEl.dataset.cardId;
    const sourceColumnId = draggedEl.dataset.columnId;
    if (!cardId || !sourceColumnId) return;

    // Traverse from the drop point upward to find a column container.
    let targetColumnEl = document.elementFromPoint(
      dropX,
      dropY,
    ) as HTMLElement | null;
    while (targetColumnEl && !targetColumnEl.dataset.columnId) {
      targetColumnEl = targetColumnEl.parentElement;
    }
    if (!targetColumnEl) {
      // If not dropped over a valid column, reset transform and exit.
      draggedEl.style.transform = "";
      draggedEl.style.zIndex = "";
      return;
    }
    const targetColumnId = targetColumnEl.dataset.columnId;

    // Determine drop index within the target column.
    const cardEls = Array.from(
      targetColumnEl.querySelectorAll("[data-card-id]"),
    ) as HTMLElement[];
    const filtered = cardEls.filter((el) => el !== draggedEl);
    let targetIndex = filtered.length;
    for (let i = 0; i < filtered.length; i++) {
      const rect = filtered[i].getBoundingClientRect();
      if (dropY < rect.top + rect.height / 2) {
        targetIndex = i;
        break;
      }
    }

    // Update state: remove card from source and insert into target.
    let movedCard: KanbanCard | undefined;
    columns = columns.map((col) => {
      if (col.id === sourceColumnId) {
        movedCard = col.cards.find((c) => c.id === cardId);
        return { ...col, cards: col.cards.filter((c) => c.id !== cardId) };
      }
      return col;
    });
    if (movedCard) {
      columns = columns.map((col) => {
        if (col.id === targetColumnId) {
          const newCards = [...col.cards];
          newCards.splice(targetIndex, 0, movedCard);
          return { ...col, cards: newCards };
        }
        return col;
      });
    }

    // Reset any inline transforms.
    draggedEl.style.transform = "";
  }
</script>

<div class="p-6">
  <!-- Board header -->
  <div class="mb-6 flex justify-between items-center">
    <h2 class="text-xl font-semibold text-gray-800">Kanban Board</h2>
    {#if !isAddingColumn}
      <button
        on:click={() => (isAddingColumn = true)}
        class="px-3 py-1.5 bg-blue-500 text-white rounded-md text-sm hover:bg-blue-600 transition-colors"
      >
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
          >Add</button
        >
        <button
          on:click={() => (isAddingColumn = false)}
          class="px-3 py-1.5 bg-gray-200 text-gray-700 rounded-md text-sm hover:bg-gray-300 transition-colors"
          >Cancel</button
        >
      </div>
    {/if}
  </div>

  <!-- Board columns -->
  <div class="flex gap-4 overflow-x-auto pb-4">
    {#each columns as column (column.id)}
      <div class="flex-shrink-0 w-sm bg-gray-50 shadow-sm rounded-sm">
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
                title="Save">Save</button
              >
              <button
                on:click={cancelColumnEdit}
                class="p-1 text-gray-500 hover:text-gray-700"
                title="Cancel">Cancel</button
              >
            </div>
          {:else}
            <div class="font-medium text-gray-700">{column.title}</div>
            <div class="flex items-center">
              <button
                on:click={() => startEditingColumn(column)}
                class="p-1 text-gray-400 hover:text-gray-700"
                title="Edit column title">Edit</button
              >
              <button
                on:click={() => deleteColumn(column.id)}
                class="p-1 text-gray-400 hover:text-red-500"
                title="Delete column">Delete</button
              >
            </div>
          {/if}
        </div>

        <!-- Cards container (mark with data attribute for drop detection) -->
        <div
          class="p-2 min-h-[80svh] overflow-y-auto"
          data-column-id={column.id}
        >
          {#each column.cards as card (card.id)}
            <div
              class="mb-2 p-3 bg-white rounded-md shadow-sm cursor-pointer hover:shadow-md transition-shadow group"
              data-card-id={card.id}
              data-column-id={column.id}
              use:draggable
              on:neodrag:start={handleDragStart}
              on:neodrag:end={handleDragEnd}
              on:click={() => openEditCardModal(card, column.id)}
            >
              <div class="flex justify-between">
                <div class="font-medium text-gray-800">{card.title}</div>
                <button
                  on:click|stopPropagation={() =>
                    deleteCard(card.id, column.id)}
                  class="p-1 text-gray-400 hover:text-red-500 rounded"
                  title="Delete card"
                >
                  Delete
                </button>
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
            Add a card
          </button>
        </div>
      </div>
    {/each}
  </div>

  <!-- Card Modal remains unchanged -->
  <KanbanCardModal
    show={showCardModal}
    card={cardModalData}
    isEdit={isEditingCard}
    on:save={saveCard}
    on:close={() => (showCardModal = false)}
  />
</div>

<style>
  .neodrag-dragging {
    z-index: 9999 !important;
    position: relative !important;
  }
</style>
