<script lang="ts">
  import { onMount } from 'svelte';
  import KeysPanel from './KeysPanel.svelte';

  type NavItem = {
    id: string;
    label: string;
    icon: string;
  };

  const navItems: NavItem[] = [
    { id: 'keys', label: 'Keys', icon: '⌨️' },
    { id: 'settings', label: 'Settings', icon: '⚙️' },
    { id: 'about', label: 'About', icon: 'ℹ️' }
  ];

  interface WindowState {
    x?: number;
    y?: number;
    width?: number;
    height?: number;
  }

  let activeNav = $state('keys');
  let greeting = $state('');
  let nameInput = $state<HTMLInputElement>();

  async function greet(name: string) {
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
    greet(nameInput.value);
    nameInput.value = '';
  }

  // Load and apply window state on mount
  onMount(async () => {
    try {
      // @ts-ignore - Tauri API injected at runtime
      const state = await window.__TAURI__.core.invoke<WindowState>('load_window_state');
      if (state.width && state.height) {
        // @ts-ignore
        const window = window.__TAURI__.window.getCurrent();
        if (state.x !== undefined && state.y !== undefined) {
          await window.setPosition(state.x, state.y);
        }
        await window.setSize(state.width, state.height);
      }
    } catch (err) {
      console.error('Failed to load window state:', err);
    }
  });
</script>

<div class="app-layout">
  <nav class="sidebar" aria-label="Main navigation">
    <div class="sidebar-header">
      <h1 class="app-title">TestTray</h1>
    </div>
    <ul class="nav-list">
      {#each navItems as item}
        <li>
          <button
            class="nav-item {activeNav === item.id ? 'active' : ''}"
            onclick={() => activeNav = item.id}
            aria-current={activeNav === item.id ? 'page' : undefined}
          >
            <span class="nav-icon">{item.icon}</span>
            <span class="nav-label">{item.label}</span>
          </button>
        </li>
      {/each}
    </ul>
    <div class="sidebar-footer">
      <p class="version">v1.0.0</p>
    </div>
  </nav>

  <main class="main-content">
    <div class="content-wrapper">
      {#if activeNav === 'keys'}
        <KeysPanel />
      {:else if activeNav === 'settings'}
        <div class="content-panel">
          <h2>Settings</h2>
          <p class="demo-text">Configure your application preferences here.</p>
          <div class="demo-card">
            <h3>General Settings</h3>
            <label>
              <input type="checkbox" /> Enable notifications
            </label>
            <label>
              <input type="checkbox" /> Auto-save changes
            </label>
            <label>
              <input type="checkbox" /> Dark mode
            </label>
          </div>
          <div class="demo-card">
            <h3>Account</h3>
            <form onsubmit={handleSubmit} class="settings-form">
              <input
                id="greet-input"
                placeholder="Enter your name..."
                bind:this={nameInput}
              />
              <button type="submit">Save Name</button>
            </form>
            {#if greeting}
              <p class="greeting">{greeting}</p>
            {/if}
          </div>
        </div>
      {:else if activeNav === 'about'}
        <div class="content-panel">
          <h2>About</h2>
          <p class="demo-text">Learn more about this application.</p>
          <div class="demo-card">
            <h3>App Info</h3>
            <p><strong>Name:</strong> TestTray</p>
            <p><strong>Version:</strong> 1.0.0</p>
            <p><strong>Framework:</strong> Tauri 2.0 + Svelte 5 + Vite</p>
          </div>
          <div class="demo-card">
            <h3>Tech Stack</h3>
            <ul>
              <li>Rust (backend)</li>
              <li>TypeScript + Svelte 5 (frontend)</li>
              <li>Vite (build tool)</li>
              <li>Tauri 2.0 (desktop framework)</li>
            </ul>
          </div>
          <div class="demo-card">
            <h3>Links</h3>
            <ul>
              <li><a href="https://tauri.app" target="_blank" rel="noopener noreferrer">Tauri Documentation</a></li>
              <li><a href="https://svelte.dev" target="_blank" rel="noopener noreferrer">Svelte Documentation</a></li>
              <li><a href="https://vite.dev" target="_blank" rel="noopener noreferrer">Vite Documentation</a></li>
            </ul>
          </div>
        </div>
      {/if}
    </div>
  </main>
</div>

<style>
  .app-layout {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  .sidebar {
    width: 240px;
    min-width: 240px;
    background: var(--code-bg);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .sidebar-header {
    padding: 1.5rem;
    border-bottom: 1px solid var(--border);
  }

  .app-title {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text-h);
  }

  .nav-list {
    list-style: none;
    padding: 1rem;
    margin: 0;
    flex: 1;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    padding: 0.75rem 1rem;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: left;
  }

  .nav-item:hover {
    background: var(--accent-bg);
    color: var(--accent);
  }

  .nav-item.active {
    background: var(--accent);
    color: white;
  }

  .nav-item.active:hover {
    background: var(--accent-border);
    color: white;
  }

  .nav-icon {
    font-size: 1.1rem;
    width: 24px;
    text-align: center;
    flex-shrink: 0;
  }

  .sidebar-footer {
    padding: 1rem;
    border-top: 1px solid var(--border);
  }

  .version {
    margin: 0;
    font-size: 0.75rem;
    color: var(--text);
    opacity: 0.6;
    text-align: center;
  }

  .main-content {
    flex: 1;
    overflow-y: auto;
    background: var(--bg);
  }

  .content-wrapper {
    padding: 2rem;
    max-width: 800px;
    margin: 0 auto;
  }

  .content-panel {
    animation: fadeIn 0.2s ease;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .content-panel h2 {
    margin: 0 0 0.5rem;
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text-h);
  }

  .demo-text {
    color: var(--text);
    opacity: 0.8;
    margin-bottom: 1.5rem;
    font-size: 1rem;
  }

  .demo-card {
    background: var(--code-bg);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 1rem;
  }

  .demo-card h3 {
    margin: 0 0 1rem;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-h);
  }

  .demo-card ul {
    margin: 0;
    padding-left: 1.25rem;
  }

  .demo-card li {
    margin: 0.5rem 0;
    color: var(--text);
  }

  .demo-card label {
    display: block;
    margin: 0.5rem 0;
    cursor: pointer;
    color: var(--text);
  }

  .demo-card input[type="checkbox"] {
    margin-right: 0.5rem;
    accent-color: var(--accent);
  }

  .settings-form {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .settings-form input {
    flex: 1;
    font-size: 1rem;
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    min-width: 200px;
  }

  .settings-form button {
    font-size: 1rem;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    border: none;
    background: var(--accent);
    color: white;
    cursor: pointer;
    font-weight: 500;
    transition: background 0.2s;
  }

  .settings-form button:hover {
    background: var(--accent-border);
  }

  .greeting {
    margin: 0.5rem 0 0;
    font-size: 1rem;
    color: var(--accent);
    font-weight: 500;
  }

  .demo-card a {
    color: var(--accent);
    text-decoration: none;
  }

  .demo-card a:hover {
    text-decoration: underline;
  }

  @media (max-width: 768px) {
    .app-layout {
      flex-direction: column;
    }

    .sidebar {
      width: 100%;
      min-width: 0;
      border-right: none;
      border-bottom: 1px solid var(--border);
    }

    .nav-list {
      display: flex;
      overflow-x: auto;
      padding: 0.5rem;
      gap: 0.5rem;
    }

    .nav-item {
      white-space: nowrap;
      padding: 0.5rem 1rem;
    }

    .nav-label {
      display: none;
    }

    .sidebar-header,
    .sidebar-footer {
      display: none;
    }

    .content-wrapper {
      padding: 1.5rem;
    }
  }
</style>