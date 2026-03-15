<script lang="ts">
  import { page } from '$app/state';
  import MindMapCanvas from '$lib/components/MindMapCanvas.svelte';
</script>

<MindMapCanvas mindMapId={page.params.id} />
