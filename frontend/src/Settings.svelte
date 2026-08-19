<script lang="ts">
  // Settings placeholder items
  const navLabels = ['General', 'Appearance', 'Advanced'];
  const navIcons = ['📝', '🎨', '⚙️'];

  // Track selected tab (0, 1, or 2)
  let selected = 0;
</script>

<div class="settings-container">
  <!-- Left Navigation Panel -->
  <aside class="nav-panel">
    <div class="nav-header">Settings</div>
    <nav>
      <button class="nav-btn active" on:click={() => selected = 0}>
        {{ navIcons[0] }} General
      </button>
      <button class="nav-btn" on:click={() => selected = 1}>
        {{ navIcons[1] }} Appearance
      </button>
      <button class="nav-btn" on:click={() => selected = 2}>
        {{ navIcons[2] }} Advanced
      </button>
    </nav>
  </aside>

  <!-- Right Content Panel -->
  <main class="content-panel">
    {#if selected === 0}
      <div class="panel">
        <h2>General Settings</h2>
        <p>General placeholder content. This is where general settings would appear.</p>
      </div>
    {/if}
    {#if selected === 1}
      <div class="panel">
        <h2>Appearance Settings</h2>
        <p>Appearance placeholder content. This is where appearance settings would appear.</p>
      </div>
    {/if}
    {#if selected === 2}
      <div class="panel">
        <h2>Advanced Settings</h2>
        <p>Advanced placeholder content. This is where advanced settings would appear.</p>
      </div>
    {/if}
  </main>
</div>

<style>
  .settings-container {
    display: flex;
    height: 100%;
    font-family: system-ui, -apple-system, sans-serif;
  }

  .nav-panel {
    width: 160px;
    background: #1a1a2e;
    color: #e0e0e0;
    padding: 1rem 0.75rem;
    display: flex;
    flex-direction: column;
    border-right: 1px solid #2d2d3d;
  }

  .nav-header {
    font-size: 1.2rem;
    font-weight: 600;
    margin-bottom: 1.5rem;
    text-align: center;
    color: #ffffff;
  }

  nav {
    flex: 1;
  }

  .nav-btn {
    width: 100%;
    padding: 0.75rem 1rem;
    background: transparent;
    border: none;
    color: #e0e0e0;
    font-size: 0.95rem;
    cursor: pointer;
    margin-bottom: 0.25rem;
    border-radius: 6px;
    text-align: left;
  }

  .nav-btn:hover {
    background: #16213e;
    color: #ffffff;
  }

  .nav-btn.active {
    background: #0f3460;
    color: #e94d3f;
  }

  .content-panel {
    flex: 1;
    padding: 1.5rem 2rem;
    overflow-y: auto;
    background: #f8f9fa;
  }

  .content-panel h2 {
    color: #1a1a2e;
    margin-top: 0;
    margin-bottom: 1rem;
    font-size: 1.5rem;
  }

  .content-panel p {
    color: #555;
    line-height: 1.6;
  }
</style>