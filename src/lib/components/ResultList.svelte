<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let results: { name: string; path: string }[] = [];
  export let selectedIndex: number = 0;

  const dispatch = createEventDispatcher();

  const handleClick = (index: number) => {
    dispatch('select', { index });
  };

  const handleMouseEnter = (index: number) => {
    dispatch('hover', { index });
  };
</script>

{#if results.length > 0}
  <div class="result-list">
    {#each results as item, index}
      <button
        type="button"
        class="result-item"
        class:selected={index === selectedIndex}
        on:click={() => handleClick(index)}
        on:mouseenter={() => handleMouseEnter(index)}
      >
        <span class="result-name">{item.name}</span>
        <span class="result-path">{item.path}</span>
      </button>
    {/each}
  </div>
{/if}

<style>
  .result-list {
    width: 100%;
    max-height: 300px;
    overflow-y: auto;
    background: rgba(255, 255, 255, 0.98);
    border-top: 1px solid #e5e5e5;
  }

  .result-item {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 12px 16px;
    border: none;
    background: transparent;
    cursor: pointer;
    text-align: left;
    font-size: 14px;
    border-bottom: 1px solid #f0f0f0;
    transition: background-color 0.1s;
  }

  .result-item:last-child {
    border-bottom: none;
  }

  .result-item:hover,
  .result-item.selected {
    background: #e6f0ff;
  }

  .result-item.selected {
    background: #d0e5ff;
  }

  .result-name {
    font-weight: 500;
    color: #333;
    margin-bottom: 2px;
  }

  .result-path {
    font-size: 12px;
    color: #888;
    word-break: break-all;
  }
</style>
