<script lang="ts">
  import type { TaskStatus } from '$lib/types';

  let { status, onchange }: { status: TaskStatus; onchange: (newStatus: TaskStatus) => void } = $props();

  const cycle: Record<TaskStatus, TaskStatus> = {
    todo: 'in_progress',
    in_progress: 'done',
    done: 'cancelled',
    cancelled: 'todo',
  };

  function handleClick() {
    onchange(cycle[status]);
  }
</script>

<button class="task-checkbox {status}" onclick={handleClick} title={status}
        aria-label="Task status: {status}">
  {#if status === 'todo'}☐{/if}
  {#if status === 'in_progress'}◐{/if}
  {#if status === 'done'}☑{/if}
  {#if status === 'cancelled'}☒{/if}
</button>

<style>
  .task-checkbox {
    background: none;
    border: none;
    cursor: pointer;
    font-size: var(--text-base);
    padding: 0;
    line-height: 1;
    flex-shrink: 0;
  }
  .task-checkbox:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
    border-radius: 2px;
  }
  .todo { color: var(--task-todo); }
  .in_progress { color: var(--task-in-progress); }
  .done { color: var(--task-done); }
  .cancelled { color: var(--task-cancelled); }
</style>
