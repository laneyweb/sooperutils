<script lang="ts">
  import { onMount } from 'svelte';

  interface KeyPressStats {
    day: number;
    week: number;
    month: number;
    year: number;
    total: number;
  }

  interface SpecificKeyTimeframes {
    day: number;
    week: number;
    month: number;
    year: number;
    total: number;
  }

  interface SpecificKeyStats {
    space: SpecificKeyTimeframes;
    backspace: SpecificKeyTimeframes;
    enter: SpecificKeyTimeframes;
    escape: SpecificKeyTimeframes;
  }

  interface KeyDebugInfo {
    key_count: number;
    listener_started: boolean;
    listener_error: string | null;
    last_key_time: number;
    channel_tx_exists: boolean;
    channel_rx_exists: boolean;
    timestamps_count: number;
    space_count: number;
    backspace_count: number;
    enter_count: number;
    escape_count: number;
    mac_permissions: {
      accessibility: boolean;
      input_monitoring: boolean;
      trusted: boolean;
    } | null;
  }

  let stats = $state<KeyPressStats>({ day: 0, week: 0, month: 0, year: 0, total: 0 });
  let specificStats = $state<SpecificKeyStats | null>(null);
  let debug = $state<KeyDebugInfo | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let showDebug = $state(false);

  async function fetchStats() {
    try {
      // @ts-ignore - Tauri API injected at runtime
      const result = await window.__TAURI__.core.invoke<KeyPressStats>('get_keypress_stats');
      stats = result;
      error = null;
    } catch (err) {
      console.error('Failed to fetch keypress stats:', err);
      error = 'Failed to load stats';
    } finally {
      loading = false;
    }
  }

  async function fetchSpecificKeyStats() {
    try {
      // @ts-ignore - Tauri API injected at runtime
      const result = await window.__TAURI__.core.invoke<SpecificKeyStats>('get_specific_key_stats');
      specificStats = result;
    } catch (err) {
      console.error('Failed to fetch specific key stats:', err);
      specificStats = null;
    }
  }

  async function fetchDebug() {
    try {
      // @ts-ignore - Tauri API injected at runtime
      const result = await window.__TAURI__.core.invoke<KeyDebugInfo>('get_keypress_debug');
      debug = result;
    } catch (err) {
      console.error('Failed to fetch debug info:', err);
      debug = null;
    }
  }

  // Initial fetch
  onMount(() => {
    fetchStats();
    fetchSpecificKeyStats();
    fetchDebug();
    // Refresh every 5 seconds
    const interval = setInterval(() => {
      fetchStats();
      fetchSpecificKeyStats();
      fetchDebug();
    }, 5000);
    return () => clearInterval(interval);
  });

  function formatNumber(num: number): string {
    return num.toLocaleString();
  }

  function formatTime(ts: number): string {
    if (ts === 0) return 'Never';
    const date = new Date(ts);
    return date.toLocaleTimeString();
  }

  async function openPermissions() {
    try {
      // @ts-ignore - Tauri API injected at runtime
      await window.__TAURI__.core.invoke('open_permission_settings');
    } catch (err) {
      console.error('Failed to open permission settings:', err);
    }
  }

  // macOS: show a warning if required privacy permissions are missing
  const permissionMissing = $derived(
    debug?.mac_permissions && !debug.mac_permissions.trusted
  );
</script>

<div class="content-panel">
  {#snippet SpecificKeyCard(keyName: string, keyIcon: string, stats: SpecificKeyTimeframes)}
    <div class="specific-key-card">
      <div class="specific-key-header">
        <span class="specific-key-icon">{keyIcon}</span>
        <span class="specific-key-name">{keyName}</span>
      </div>
      <div class="specific-key-stats">
        <div class="specific-stat">
          <span class="specific-stat-label">Today</span>
          <span class="specific-stat-value">{formatNumber(stats.day)}</span>
        </div>
        <div class="specific-stat">
          <span class="specific-stat-label">Week</span>
          <span class="specific-stat-value">{formatNumber(stats.week)}</span>
        </div>
        <div class="specific-stat">
          <span class="specific-stat-label">Month</span>
          <span class="specific-stat-value">{formatNumber(stats.month)}</span>
        </div>
        <div class="specific-stat">
          <span class="specific-stat-label">Year</span>
          <span class="specific-stat-value">{formatNumber(stats.year)}</span>
        </div>
        <div class="specific-stat total">
          <span class="specific-stat-label">Total</span>
          <span class="specific-stat-value">{formatNumber(stats.total)}</span>
        </div>
      </div>
    </div>
  {/snippet}
  <div class="panel-header">
    <h2>Key Press Statistics</h2>
    <button class="debug-toggle" onclick={() => showDebug = !showDebug}>
      {showDebug ? 'Hide Debug' : 'Show Debug'}
    </button>
  </div>

  {#if permissionMissing}
    <div class="perm-warning">
      <strong>⚠️ Missing permissions</strong>
      <p>
        SooperUtils needs <strong>Accessibility</strong> and <strong>Input Monitoring</strong>
        permissions to count keystrokes. Without them macOS silently blocks keyboard events.
      </p>
      <p class="perm-hint">
        After granting, reopen the window or press “Refresh Debug” to verify. Note: ad-hoc signed
        apps lose these permissions every time the app is rebuilt/reinstalled — re-grant if it stops counting.
      </p>
      <button class="retry-btn" onclick={openPermissions}>Open Permission Settings</button>
    </div>
  {/if}

  {#if loading}
    <p class="demo-text">Loading stats...</p>
  {:else if error}
    <p class="demo-text" style="color: var(--accent);">{error}</p>
    <button class="retry-btn" onclick={fetchStats}>Retry</button>
  {:else}
    <div class="stats-grid">
      <div class="stat-card">
        <span class="stat-label">Today</span>
        <span class="stat-value">{formatNumber(stats.day)}</span>
      </div>
      <div class="stat-card">
        <span class="stat-label">This Week</span>
        <span class="stat-value">{formatNumber(stats.week)}</span>
      </div>
      <div class="stat-card">
        <span class="stat-label">This Month</span>
        <span class="stat-value">{formatNumber(stats.month)}</span>
      </div>
      <div class="stat-card">
        <span class="stat-label">This Year</span>
        <span class="stat-value">{formatNumber(stats.year)}</span>
      </div>
      <div class="stat-card total">
        <span class="stat-label">Total</span>
        <span class="stat-value">{formatNumber(stats.total)}</span>
      </div>
    </div>
    <div class="stats-info">
      <p>Stats update automatically every 5 seconds. Data persists across app restarts.</p>
    </div>

    {#if specificStats}
      <div class="specific-keys-section">
        <h3>Specific Key Counts</h3>
        <div class="specific-keys-table-container">
          <table class="specific-keys-table">
            <thead>
              <tr>
                <th class="key-col">Key</th>
                <th>Today</th>
                <th>Week</th>
                <th>Month</th>
                <th>Year</th>
                <th class="total-col">Total</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td class="key-col"><span class="key-icon">␣</span><span class="key-name">Space</span></td>
                <td>{formatNumber(specificStats.space.day)}</td>
                <td>{formatNumber(specificStats.space.week)}</td>
                <td>{formatNumber(specificStats.space.month)}</td>
                <td>{formatNumber(specificStats.space.year)}</td>
                <td class="total-col">{formatNumber(specificStats.space.total)}</td>
              </tr>
              <tr>
                <td class="key-col"><span class="key-icon">⌫</span><span class="key-name">Backspace</span></td>
                <td>{formatNumber(specificStats.backspace.day)}</td>
                <td>{formatNumber(specificStats.backspace.week)}</td>
                <td>{formatNumber(specificStats.backspace.month)}</td>
                <td>{formatNumber(specificStats.backspace.year)}</td>
                <td class="total-col">{formatNumber(specificStats.backspace.total)}</td>
              </tr>
              <tr>
                <td class="key-col"><span class="key-icon">↵</span><span class="key-name">Enter</span></td>
                <td>{formatNumber(specificStats.enter.day)}</td>
                <td>{formatNumber(specificStats.enter.week)}</td>
                <td>{formatNumber(specificStats.enter.month)}</td>
                <td>{formatNumber(specificStats.enter.year)}</td>
                <td class="total-col">{formatNumber(specificStats.enter.total)}</td>
              </tr>
              <tr>
                <td class="key-col"><span class="key-icon">⎋</span><span class="key-name">Escape</span></td>
                <td>{formatNumber(specificStats.escape.day)}</td>
                <td>{formatNumber(specificStats.escape.week)}</td>
                <td>{formatNumber(specificStats.escape.month)}</td>
                <td>{formatNumber(specificStats.escape.year)}</td>
                <td class="total-col">{formatNumber(specificStats.escape.total)}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    {/if}
  {/if}

  {#if showDebug && debug}
    <div class="debug-panel">
      <h3>Debug Information</h3>
      <div class="debug-grid">
        <div class="debug-item">
          <span class="debug-label">Key Count (atomic)</span>
          <span class="debug-value">{formatNumber(debug.key_count)}</span>
        </div>
        <div class="debug-item">
          <span class="debug-label">Listener Started</span>
          <span class="debug-value">{debug.listener_started ? '✅ Yes' : '❌ No'}</span>
        </div>
        <div class="debug-item">
          <span class="debug-label">Channel TX Exists</span>
          <span class="debug-value">{debug.channel_tx_exists ? '✅ Yes' : '❌ No'}</span>
        </div>
        <div class="debug-item">
          <span class="debug-label">Channel RX Exists</span>
          <span class="debug-value">{debug.channel_rx_exists ? '✅ Yes' : '❌ No'}</span>
        </div>
        <div class="debug-item">
          <span class="debug-label">Stored Timestamps</span>
          <span class="debug-value">{formatNumber(debug.timestamps_count)}</span>
        </div>
        <div class="debug-item">
          <span class="debug-label">Last Key Time</span>
          <span class="debug-value">{formatTime(debug.last_key_time)}</span>
        </div>
        {#if debug.listener_error}
          <div class="debug-item error">
            <span class="debug-label">Listener Error</span>
            <span class="debug-value">{debug.listener_error}</span>
          </div>
        {/if}
        {#if debug.mac_permissions}
          <div class="debug-item {debug.mac_permissions.trusted ? '' : 'error'}">
            <span class="debug-label">Accessibility Permission</span>
            <span class="debug-value">{debug.mac_permissions.accessibility ? '✅ Granted' : '❌ Missing'}</span>
          </div>
          <div class="debug-item {debug.mac_permissions.input_monitoring ? '' : 'error'}">
            <span class="debug-label">Input Monitoring Permission</span>
            <span class="debug-value">{debug.mac_permissions.input_monitoring ? '✅ Granted' : '❌ Missing'}</span>
          </div>
        {/if}
      </div>
      <button class="retry-btn" onclick={fetchDebug}>Refresh Debug</button>
    </div>
  {/if}
</div>

<style>
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .panel-header h2 {
    margin: 0;
  }

  .debug-toggle {
    padding: 0.4rem 0.8rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--code-bg);
    color: var(--text);
    cursor: pointer;
    font-size: 0.85rem;
    transition: all 0.15s ease;
  }

  .debug-toggle:hover {
    background: var(--accent-bg);
    border-color: var(--accent-border);
    color: var(--accent);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .stat-card {
    background: var(--code-bg);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 1.5rem 1rem;
    text-align: center;
    transition: transform 0.15s ease, box-shadow 0.15s ease;
  }

  .stat-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .stat-card.total {
    grid-column: 1 / -1;
    max-width: 300px;
    margin: 0 auto;
    background: var(--accent-bg);
    border-color: var(--accent-border);
  }

  .stat-label {
    display: block;
    font-size: 0.85rem;
    color: var(--text);
    opacity: 0.8;
    margin-bottom: 0.5rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .stat-value {
    display: block;
    font-size: 2.5rem;
    font-weight: 700;
    color: var(--text-h);
    line-height: 1.2;
  }

  .stat-card.total .stat-value {
    color: var(--accent);
    font-size: 3rem;
  }

  .stats-info {
    padding: 1rem;
    background: var(--code-bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    text-align: center;
  }

  .stats-info p {
    margin: 0;
    font-size: 0.85rem;
    color: var(--text);
    opacity: 0.7;
  }

  .retry-btn {
    margin-top: 1rem;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    border: none;
    background: var(--accent);
    color: white;
    cursor: pointer;
    font-weight: 500;
    transition: background 0.2s;
  }

  .retry-btn:hover {
    background: var(--accent-border);
  }

  .debug-panel {
    margin-top: 2rem;
    padding: 1.5rem;
    background: var(--code-bg);
    border: 1px solid var(--border);
    border-radius: 12px;
  }

  .debug-panel h3 {
    margin: 0 0 1rem;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-h);
  }

  .debug-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .debug-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.75rem;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  .debug-item.error {
    border-color: var(--accent);
    background: rgba(170, 59, 255, 0.1);
  }

  .debug-label {
    font-size: 0.75rem;
    color: var(--text);
    opacity: 0.7;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .debug-value {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text-h);
    font-family: monospace;
  }

  .debug-item.error .debug-value {
    color: var(--accent);
    word-break: break-word;
  }

  .perm-warning {
    margin-bottom: 1.5rem;
    padding: 1.25rem;
    background: rgba(170, 59, 255, 0.08);
    border: 1px solid var(--accent);
    border-radius: 12px;
  }

  .perm-warning strong {
    color: var(--accent);
    font-size: 1rem;
  }

  .perm-warning p {
    margin: 0.5rem 0;
    color: var(--text);
    font-size: 0.9rem;
  }

  .perm-hint {
    opacity: 0.75;
    font-size: 0.8rem !important;
  }

  .specific-keys-section {
    margin-top: 2rem;
    padding: 1.5rem;
    background: var(--code-bg);
    border: 1px solid var(--border);
    border-radius: 12px;
  }

  .specific-keys-section h3 {
    margin: 0 0 1rem;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-h);
  }

  .specific-keys-table-container {
    overflow-x: auto;
  }

  .specific-keys-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
  }

  .specific-keys-table th,
  .specific-keys-table td {
    padding: 0.75rem 1rem;
    text-align: right;
    border-bottom: 1px solid var(--border);
  }

  .specific-keys-table th {
    font-weight: 600;
    color: var(--text);
    opacity: 0.7;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-size: 0.7rem;
    background: var(--bg);
    position: sticky;
    top: 0;
  }

  .specific-keys-table th:first-child,
  .specific-keys-table td:first-child {
    text-align: left;
  }

  .specific-keys-table .key-col {
    width: 140px;
    white-space: nowrap;
  }

  .specific-keys-table .total-col {
    font-weight: 700;
    color: var(--accent);
  }

  .specific-keys-table tbody tr:last-child td {
    border-bottom: none;
  }

  .specific-keys-table tbody tr:hover td {
    background: var(--bg);
  }

  .key-icon {
    font-family: monospace;
    font-size: 1.1rem;
    color: var(--accent);
    margin-right: 0.5rem;
    display: inline-block;
    width: 1.5em;
    text-align: center;
  }

  .key-name {
    font-weight: 500;
    color: var(--text-h);
  }

  @media (max-width: 600px) {
    .specific-keys-table th,
    .specific-keys-table td {
      padding: 0.5rem 0.75rem;
      font-size: 0.85rem;
    }
    .specific-keys-table .key-col {
      width: 120px;
    }
  }
</style>