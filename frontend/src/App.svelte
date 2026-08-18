<script lang="ts">
  import { onMount } from 'svelte';

  let name = '';
  let greeting = '';

  async function greet() {
    if (!name.trim()) return;
    try {
      // @ts-ignore - Tauri API injected at runtime
      const result = await window.__TAURI__.core.invoke('greet', { name: name.trim() });
      greeting = result;
    } catch (err) {
      console.error(err);
      greeting = 'Error calling greet command';
    }
  }

  function handleSubmit(e: Event) {
    e.preventDefault();
    greet();
  }

  onMount(() => {
    // Focus the input on mount
    const input = document.getElementById('greet-input') as HTMLInputElement;
    input?.focus();
  });
</script>

<main class="container">
  <h1>Welcome to Tauri + Svelte</h1>

  <div class="row">
    <a href="https://tauri.app" target="_blank" rel="noopener noreferrer">
      <img src="https://tauri.app/favicon.ico" class="logo tauri" alt="Tauri logo" />
    </a>
    <a href="https://svelte.dev" target="_blank" rel="noopener noreferrer">
      <img src="https://svelte.dev/svelte-logo.svg" class="logo svelte" alt="Svelte logo" />
    </a>
  </div>

  <p>Click on the logos to learn more about the frameworks</p>

  <form class="row" on:submit={handleSubmit} id="greet-form">
    <input id="greet-input" placeholder="Enter a name..." bind:value={name} />
    <button type="submit">Greet</button>
  </form>

  {#if greeting}
    <p id="greet-msg" class="greeting">{greeting}</p>
  {/if}
</main>

<style>
  .container {
    padding: 2rem;
    text-align: center;
  }

  .row {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1rem;
    margin: 1rem 0;
    flex-wrap: wrap;
  }

  .logo {
    height: 6em;
    padding: 1.5em;
    will-change: filter;
    transition: filter 300ms;
  }

  .logo:hover {
    filter: drop-shadow(0 0 2em #24c8db);
  }

  .logo.svelte:hover {
    filter: drop-shadow(0 0 2em #ff3e00);
  }

  input {
    font-size: 1rem;
    padding: 0.5rem;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: var(--code-bg);
    color: var(--text);
    min-width: 200px;
  }

  button {
    font-size: 1rem;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    border: none;
    background: var(--accent);
    color: white;
    cursor: pointer;
    font-weight: 500;
    transition: background 0.2s;
  }

  button:hover {
    background: var(--accent-border);
  }

  .greeting {
    margin-top: 1rem;
    font-size: 1.2rem;
    color: var(--accent);
    font-weight: 500;
  }

  a {
    color: var(--accent);
    text-decoration: none;
  }

  a:hover {
    text-decoration: underline;
  }
</style>