<script lang="ts">
  import "../app.css";
  import FileDialog from "../components/FileDialog.svelte";
  import SettingsDialog from "../components/SettingsDialog.svelte";
  import Toaster from "$lib/Toaster.svelte";
</script>

<FileDialog />
<SettingsDialog />
<Toaster />

<slot />

<div class="fixed bottom-4 left-4 flex flex-col gap-2 text-xs text-gray-500">
  <div class="bg-white/80 p-2 rounded shadow-sm">
    <span class="font-medium">Keyboard shortcuts:</span>
    <div class="mt-1 flex flex-col gap-2 items-start">
      <div>
        <kbd class="px-1 py-0.5 bg-gray-100 border rounded">Ctrl + O</kbd> Open note
      </div>
      <div>
        <kbd class="px-1 py-0.5 bg-gray-100 border rounded">Ctrl + ,</kbd> Settings
      </div>
      <div>
        <kbd class="px-1 py-0.5 bg-gray-100 border rounded">Ctrl + L</kbd>
        Toggle Edit/View
      </div>
      <div>
        <kbd class="px-1 py-0.5 bg-gray-100 border rounded">Ctrl + S</kbd>
        Save
      </div>
    </div>
  </div>
</div>
