<script>
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  onMount(() => {
    const today = new Date().toISOString().split('T')[0];
    goto(`/journal/${today}`);
  });
</script>
