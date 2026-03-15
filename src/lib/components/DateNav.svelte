<script lang="ts">
  import { goto } from '$app/navigation';

  let { date }: { date: string } = $props();

  function offsetDate(days: number): string {
    const d = new Date(date + 'T00:00:00');
    d.setDate(d.getDate() + days);
    return d.toISOString().split('T')[0];
  }

  function formatDate(dateStr: string): string {
    const d = new Date(dateStr + 'T00:00:00');
    return d.toLocaleDateString('en-US', { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' });
  }

  function prev() { void goto(`/journal/${offsetDate(-1)}`); }
  function next() { void goto(`/journal/${offsetDate(1)}`); }
  function goToday() {
    const d = new Date();
    const today = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
    void goto(`/journal/${today}`);
  }
</script>

<div class="date-nav">
  <button onclick={prev} class="nav-btn" aria-label="Previous day">←</button>
  <h1 class="date-header">{formatDate(date)}</h1>
  <button onclick={next} class="nav-btn" aria-label="Next day">→</button>
  <button onclick={goToday} class="today-btn">Today</button>
</div>

<style>
  .date-nav {
    display: flex;
    align-items: baseline;
    gap: var(--space-4);
    margin-bottom: var(--space-8);
  }
  .date-header {
    font-size: var(--text-2xl);
    font-weight: 600;
    line-height: var(--leading-tight);
  }
  .nav-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: var(--text-lg);
    color: var(--text-muted);
    padding: var(--space-1);
  }
  .nav-btn:hover { color: var(--text-primary); }
  .nav-btn:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
    border-radius: 2px;
  }
  .today-btn {
    background: none;
    border: 1px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
    font-size: var(--text-sm);
    color: var(--text-secondary);
    padding: var(--space-1) var(--space-3);
  }
  .today-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .today-btn:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
