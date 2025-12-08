// src/http/ui/styles.rs

/// Base CSS variables and reset (included on every page)
pub const BASE: &str = r#"
:root {
    --bg-gradient-start: #ff6b35;
    --bg-gradient-end: #f7931e;
    --card-bg: white;
    --text-primary: #2d3748;
    --text-secondary: #718096;
    --border-color: #e2e8f0;
    --accent-color: #ff6b35;
    --accent-hover: #e85a2a;
    --secondary-bg: #f7fafc;
    --secondary-hover: #edf2f7;
}

@media (prefers-color-scheme: dark) {
    :root {
        --bg-gradient-start: #1a202c;
        --bg-gradient-end: #2d3748;
        --card-bg: #2d3748;
        --text-primary: #f7fafc;
        --text-secondary: #cbd5e0;
        --border-color: #4a5568;
        --accent-color: #ff6b35;
        --accent-hover: #ff8555;
        --secondary-bg: #1a202c;
        --secondary-hover: #374151;
    }
}

* { margin: 0; padding: 0; box-sizing: border-box; }

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    line-height: 1.6;
    color: var(--text-primary);
    background: linear-gradient(135deg, var(--bg-gradient-start) 0%, var(--bg-gradient-end) 100%);
    min-height: 100vh;
    padding: 40px 20px;
}

.container { max-width: 1200px; margin: 0 auto; }

footer {
    margin-top: 40px;
    text-align: center;
    color: rgba(255, 255, 255, 0.9);
}
"#;

/// Button styles
pub const BUTTONS: &str = r#"
.btn {
    padding: 10px 16px;
    border-radius: 6px;
    text-decoration: none;
    font-weight: 500;
    font-size: 0.9em;
    transition: all 0.2s;
    display: inline-block;
}
.btn-primary { background: var(--accent-color); color: white; }
.btn-primary:hover { background: var(--accent-hover); }
.btn-secondary {
    background: var(--secondary-bg);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
}
.btn-secondary:hover { background: var(--secondary-hover); }
"#;

/// Header styles
pub const HEADER: &str = r#"
header {
    background: var(--card-bg);
    border-radius: 12px;
    padding: 30px;
    margin-bottom: 30px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
}
header h1 { color: var(--accent-color); margin-bottom: 10px; font-size: 2.5em; }
.subtitle { color: var(--text-secondary); font-size: 1.1em; }
"#;

/// Stats cards (used on home page)
pub const STATS: &str = r#"
.stats { display: flex; gap: 20px; margin-top: 20px; }
.stat-card {
    background: var(--secondary-bg);
    padding: 15px 20px;
    border-radius: 8px;
    border-left: 4px solid var(--accent-color);
}
.stat-value { font-size: 2em; font-weight: bold; color: var(--accent-color); }
.stat-label { color: var(--text-secondary); font-size: 0.9em; }
"#;

/// Node card styles (used on home page)
pub const NODE_CARDS: &str = r#"
.nodes-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(500px, 1fr));
    gap: 20px;
}
.node-card {
    background: var(--card-bg);
    border-radius: 12px;
    padding: 25px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    transition: transform 0.2s, box-shadow 0.2s;
}
.node-card:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 20px rgba(0, 0, 0, 0.3);
}
.node-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    padding-bottom: 15px;
    border-bottom: 1px solid var(--border-color);
}
.node-header h3 { color: var(--text-primary); font-size: 1.3em; }
.status-badge {
    padding: 5px 12px;
    border-radius: 20px;
    font-size: 0.85em;
    font-weight: 600;
    background: var(--accent-color);
    color: white;
}
.node-info { margin-bottom: 20px; }
.node-actions { display: flex; gap: 10px; flex-wrap: wrap; }
.no-data {
    background: var(--card-bg);
    border-radius: 12px;
    padding: 40px;
    text-align: center;
    color: var(--text-secondary);
}
"#;

/// Info row styles (label: value pairs)
pub const INFO_ROWS: &str = r#"
.info-row {
    display: flex;
    justify-content: space-between;
    padding: 8px 0;
    border-bottom: 1px solid var(--border-color);
}
.info-label { color: var(--text-secondary); font-weight: 500; }
.info-value { color: var(--text-primary); font-weight: 600; }
"#;

/// Node dashboard specific styles
pub const NODE_DASHBOARD: &str = r#"
.breadcrumb { color: var(--text-secondary); margin-bottom: 15px; }
.breadcrumb a { color: var(--accent-color); text-decoration: none; }
.breadcrumb a:hover { text-decoration: underline; }

.node-id-full {
    font-family: "Courier New", monospace;
    font-size: 0.9em;
    color: var(--text-secondary);
}

.node-meta {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 15px;
    margin-top: 20px;
}
.meta-item {
    background: var(--secondary-bg);
    padding: 12px 16px;
    border-radius: 8px;
    border-left: 3px solid var(--accent-color);
}
.meta-label { color: var(--text-secondary); font-size: 0.85em; margin-bottom: 4px; }
.meta-value { color: var(--text-primary); font-weight: 600; font-size: 1.1em; }

.charts-section { margin-top: 30px; }
.charts-section h2 { color: var(--text-primary); margin-bottom: 20px; font-size: 1.5em; }
.charts-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(380px, 1fr));
    gap: 20px;
}
.chart-card {
    background: var(--card-bg);
    border-radius: 12px;
    padding: 20px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}
.chart-card h3 { color: var(--text-primary); margin-bottom: 15px; font-size: 1.2em; }
.chart-container {
    background: var(--secondary-bg);
    border-radius: 8px;
    padding: 10px;
    min-height: 280px;
}
.chart-container img { width: 100%; height: auto; display: block; }

.actions { margin-top: 30px; display: flex; gap: 10px; flex-wrap: wrap; }
"#;

/// Error page styles
pub const ERROR_PAGE: &str = r#"
.error-container {
    text-align: center;
    padding: 60px 40px;
    background: var(--card-bg);
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}
.error-container h1 {
    color: #dc3545;
    font-size: 2em;
    margin-bottom: 15px;
}
.error-container p {
    color: var(--text-secondary);
    margin-bottom: 25px;
}
"#;
