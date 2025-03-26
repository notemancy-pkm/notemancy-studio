<script lang="ts">
  import "../app.css";
  import FileDialog from "../components/FileDialog.svelte";
  import SettingsDialog from "../components/SettingsDialog.svelte";
</script>

<FileDialog />
<SettingsDialog />

<slot />

<div class="fixed bottom-4 right-4 flex flex-col gap-2 text-xs text-gray-500">
  <div class="bg-white/80 p-2 rounded shadow-sm">
    <span class="font-medium">Keyboard shortcuts:</span>
    <div class="mt-1">
      <div>
        <kbd class="px-1 py-0.5 bg-gray-100 border rounded">Ctrl + O</kbd> Open note
      </div>
      <div>
        <kbd class="px-1 py-0.5 bg-gray-100 border rounded">Ctrl + ,</kbd> Settings
      </div>
    </div>
  </div>
</div>
