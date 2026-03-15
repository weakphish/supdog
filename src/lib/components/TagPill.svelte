<script lang="ts">
  let { name }: { name: string } = $props();
</script>

<a href="/tag/{name}" class="tag-pill">#{name}</a>

<style>
  .tag-pill {
    display: inline-block;
    font-size: var(--text-xs);
    padding: 1px 8px;
    background: var(--tag-bg);
    color: var(--tag-text);
    border-radius: 3px;
    text-decoration: none;
    margin-left: var(--space-2);
  }
  .tag-pill:hover {
    background: var(--border-strong);
  }
  .tag-pill:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
