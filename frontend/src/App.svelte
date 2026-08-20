<script lang="ts">
  import { onMount } from 'svelte';
  import { getVersion } from '@tauri-apps/api/app';
  import KeysPanel from './KeysPanel.svelte';

  type NavItem = {
    id: string;
    label: string;
    glyph: string;
    color: string;
  };

  const navItems: NavItem[] = [
    { id: 'keys', label: 'Keys', glyph: '⌨', color: '#007AFF' },
    { id: 'settings', label: 'Settings', glyph: '⚙︎', color: '#8E8E93' },
    { id: 'about', label: 'About', glyph: 'ℹ', color: '#5E5CE6' }
  ];

  let activeNav = $state('keys');
  let greeting = $state('');
  let nameInput = $state<HTMLInputElement>();
  let version = $state('');

  onMount(() => {
    getVersion()
      .then((v) => (version = v))
      .catch((err) => console.error('Failed to get app version:', err));
  });

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

</script>

<div class="app-layout">
  <nav class="sidebar" aria-label="Main navigation">
    <!-- Empty strip under the traffic lights; also lets the window be dragged -->
    <div class="sidebar-drag" data-tauri-drag-region></div>
    <ul class="nav-list">
      {#each navItems as item}
        <li>
          <button
            class="nav-item {activeNav === item.id ? 'active' : ''}"
            onclick={() => activeNav = item.id}
            aria-current={activeNav === item.id ? 'page' : undefined}
          >
            <span class="nav-tile" style="background: {item.color}">
              <span class="nav-glyph">{item.glyph}</span>
            </span>
            <span class="nav-label">{item.label}</span>
          </button>
        </li>
      {/each}
    </ul>
    <div class="sidebar-footer">
      <p class="version">SooperUtils <span class="version-num">v{version}</span></p>
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
            <p><strong>Name:</strong> SooperUtils</p>
            <p><strong>Version:</strong> {version || '…'}</p>
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

  /* macOS System Settings-style sidebar */
  .sidebar {
    width: 224px;
    min-width: 224px;
    background: var(--sidebar-bg);
    display: flex;
    flex-direction: column;
    height: 100%;
    -webkit-user-select: none;
    user-select: none;
  }

  /* Traffic-light clearance strip (overlay titlebar); drag handle for the window */
  .sidebar-drag {
    height: 50px;
    flex-shrink: 0;
  }

  .nav-list {
    list-style: none;
    padding: 8px 10px;
    margin: 0;
    flex: 1;
    overflow-y: auto;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    height: 30px;
    padding: 0 8px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    cursor: default;
    text-align: left;
    margin-bottom: 2px;
    transition: background 0.1s ease;
  }

  .nav-item:hover {
    background: var(--selected-bg);
  }

  .nav-item.active {
    background: var(--accent);
    color: #fff;
  }

  .nav-item.active .nav-glyph {
    color: #fff;
  }

  .nav-tile {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: #fff;
  }

  .nav-glyph {
    font-size: 15px;
    line-height: 1;
  }

  .nav-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sidebar-footer {
    padding: 8px 16px 12px;
  }

  .version {
    margin: 0;
    font-size: 11px;
    color: var(--text-secondary);
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
    .sidebar {
      width: 180px;
      min-width: 180px;
    }

    .sidebar-drag {
      height: 50px;
    }

    .content-wrapper {
      padding: 1.5rem;
    }
  }
</style>