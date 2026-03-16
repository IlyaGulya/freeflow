/// CSS styles for the Wrenflow settings UI.
/// Designed to match the SwiftUI look as closely as possible in a WebView.

pub const GLOBAL_CSS: &str = r#"
:root {
    --bg-primary: #1e1e1e;
    --bg-secondary: #2a2a2a;
    --bg-card: #2d2d2d;
    --bg-input: #3a3a3a;
    --bg-hover: #3d3d3d;
    --text-primary: #e0e0e0;
    --text-secondary: #a0a0a0;
    --text-tertiary: #707070;
    --accent: #007AFF;
    --accent-hover: #0062cc;
    --accent-bg: rgba(0, 122, 255, 0.15);
    --green: #34c759;
    --green-bg: rgba(52, 199, 89, 0.12);
    --red: #ff3b30;
    --red-bg: rgba(255, 59, 48, 0.12);
    --orange: #ff9500;
    --orange-bg: rgba(255, 149, 0, 0.12);
    --yellow: #ffcc00;
    --yellow-bg: rgba(255, 204, 0, 0.14);
    --border: rgba(255, 255, 255, 0.06);
    --border-input: rgba(255, 255, 255, 0.12);
    --divider: rgba(255, 255, 255, 0.08);
    --radius-sm: 6px;
    --radius-md: 10px;
    --radius-lg: 12px;
    --shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
}

@media (prefers-color-scheme: light) {
    :root {
        --bg-primary: #f5f5f5;
        --bg-secondary: #ffffff;
        --bg-card: #ffffff;
        --bg-input: #f0f0f0;
        --bg-hover: #e8e8e8;
        --text-primary: #1a1a1a;
        --text-secondary: #666666;
        --text-tertiary: #999999;
        --border: rgba(0, 0, 0, 0.06);
        --border-input: rgba(0, 0, 0, 0.15);
        --divider: rgba(0, 0, 0, 0.08);
        --shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    }
}

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    font-size: 13px;
    color: var(--text-primary);
    background: var(--bg-primary);
    -webkit-font-smoothing: antialiased;
}

/* Layout */
.settings-layout {
    display: flex;
    height: 100vh;
}

.sidebar {
    width: 180px;
    padding: 10px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--divider);
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex-shrink: 0;
}

.sidebar-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    border: none;
    background: transparent;
    color: var(--text-primary);
    font-size: 13px;
    width: 100%;
    text-align: left;
}

.sidebar-item:hover {
    background: var(--bg-hover);
}

.sidebar-item.active {
    background: var(--accent-bg);
    color: var(--accent);
}

.content-area {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
}

/* Cards */
.card {
    padding: 16px;
    background: var(--bg-card);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    margin-bottom: 20px;
}

.card-header {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 12px;
}

.card-icon {
    font-size: 14px;
}

/* Form elements */
.input, .textarea, .select {
    width: 100%;
    padding: 8px 10px;
    background: var(--bg-input);
    border: 1px solid var(--border-input);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 13px;
    font-family: inherit;
    outline: none;
    transition: border-color 0.15s;
}

.input:focus, .textarea:focus, .select:focus {
    border-color: var(--accent);
}

.input-mono {
    font-family: "SF Mono", "Menlo", "Monaco", "Consolas", monospace;
}

.textarea {
    resize: vertical;
    min-height: 80px;
    font-family: "SF Mono", "Menlo", "Monaco", "Consolas", monospace;
}

.input-row {
    display: flex;
    gap: 8px;
    align-items: center;
}

/* Buttons */
.btn {
    padding: 6px 14px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-input);
    background: var(--bg-input);
    color: var(--text-primary);
    font-size: 12px;
    cursor: pointer;
    transition: background 0.15s;
    white-space: nowrap;
}

.btn:hover {
    background: var(--bg-hover);
}

.btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.btn-primary {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
}

.btn-primary:hover {
    background: var(--accent-hover);
}

.btn-danger {
    color: var(--red);
}

.btn-danger:hover {
    background: var(--red-bg);
}

/* Toggle */
.toggle-row {
    display: flex;
    align-items: center;
    gap: 10px;
}

.toggle {
    position: relative;
    width: 40px;
    height: 22px;
    cursor: pointer;
}

.toggle input {
    opacity: 0;
    width: 0;
    height: 0;
}

.toggle-slider {
    position: absolute;
    inset: 0;
    background: var(--bg-hover);
    border-radius: 11px;
    transition: background 0.2s;
}

.toggle-slider::before {
    content: "";
    position: absolute;
    width: 18px;
    height: 18px;
    left: 2px;
    top: 2px;
    background: white;
    border-radius: 50%;
    transition: transform 0.2s;
}

.toggle input:checked + .toggle-slider {
    background: var(--accent);
}

.toggle input:checked + .toggle-slider::before {
    transform: translateX(18px);
}

/* Segmented control */
.segmented {
    display: flex;
    background: var(--bg-input);
    border-radius: var(--radius-sm);
    padding: 2px;
    gap: 2px;
}

.segmented-option {
    flex: 1;
    padding: 6px 12px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    transition: all 0.15s;
    text-align: center;
}

.segmented-option.active {
    background: var(--accent);
    color: white;
}

/* Radio option */
.radio-option {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px;
    border-radius: 8px;
    cursor: pointer;
    border: 1.5px solid transparent;
    background: var(--bg-input);
    transition: all 0.15s;
}

.radio-option:hover {
    background: var(--bg-hover);
}

.radio-option.selected {
    background: var(--accent-bg);
    border-color: var(--accent);
}

.radio-dot {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: 2px solid var(--text-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
}

.radio-option.selected .radio-dot {
    border-color: var(--accent);
    background: var(--accent);
}

.radio-dot::after {
    content: "";
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: white;
    display: none;
}

.radio-option.selected .radio-dot::after {
    display: block;
}

/* Status badges */
.badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
}

.badge-green {
    color: var(--green);
    background: var(--green-bg);
}

.badge-red {
    color: var(--red);
    background: var(--red-bg);
}

.badge-orange {
    color: var(--orange);
    background: var(--orange-bg);
}

.badge-blue {
    color: var(--accent);
    background: var(--accent-bg);
}

.badge-neutral {
    color: var(--text-secondary);
    background: var(--bg-input);
}

/* Permission row */
.permission-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px;
    background: var(--bg-input);
    border-radius: var(--radius-sm);
}

.permission-icon {
    color: var(--accent);
    font-size: 14px;
    width: 20px;
    text-align: center;
}

.permission-label {
    flex: 1;
}

/* Expandable section */
.expandable-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px;
    cursor: pointer;
    border: none;
    background: transparent;
    color: var(--text-primary);
    width: 100%;
    text-align: left;
}

.expandable-header:hover {
    background: var(--bg-hover);
    border-radius: var(--radius-sm);
}

.chevron {
    transition: transform 0.2s;
    color: var(--text-secondary);
    font-size: 11px;
}

.chevron.expanded {
    transform: rotate(90deg);
}

/* Utility */
.caption {
    font-size: 11px;
    color: var(--text-secondary);
}

.caption-bold {
    font-size: 11px;
    font-weight: 600;
}

.mono {
    font-family: "SF Mono", "Menlo", "Monaco", "Consolas", monospace;
}

.text-green { color: var(--green); }
.text-red { color: var(--red); }
.text-orange { color: var(--orange); }
.text-blue { color: var(--accent); }
.text-secondary { color: var(--text-secondary); }
.text-tertiary { color: var(--text-tertiary); }

.spacer { flex: 1; }

.divider {
    height: 1px;
    background: var(--divider);
    margin: 10px 0;
}

.flex-row {
    display: flex;
    align-items: center;
    gap: 8px;
}

.flex-col {
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.gap-sm { gap: 4px; }
.gap-md { gap: 8px; }
.gap-lg { gap: 16px; }

.w-full { width: 100%; }
.mt-sm { margin-top: 4px; }
.mt-md { margin-top: 8px; }

/* Slider */
.slider-row {
    display: flex;
    align-items: center;
    gap: 8px;
}

.slider {
    flex: 1;
    accent-color: var(--accent);
}

.slider-label {
    min-width: 50px;
    text-align: right;
    color: var(--text-secondary);
    font-size: 12px;
}

/* Pipeline step */
.pipeline-step {
    display: flex;
    gap: 12px;
    padding: 10px;
    background: var(--bg-input);
    border-radius: var(--radius-sm);
}

.step-number {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--accent-bg);
    color: var(--accent);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    font-weight: 600;
    flex-shrink: 0;
}

.step-content {
    flex: 1;
    min-width: 0;
}

.step-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
}

.step-title {
    font-size: 12px;
    font-weight: 600;
}

/* History entry */
.history-entry {
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
}

.history-header {
    display: flex;
    align-items: center;
    padding: 12px;
    cursor: pointer;
    gap: 8px;
}

.history-header:hover {
    background: var(--bg-hover);
}

.history-body {
    padding: 16px;
    border-top: 1px solid var(--divider);
}

/* Code block */
.code-block {
    font-family: "SF Mono", "Menlo", "Monaco", "Consolas", monospace;
    font-size: 11px;
    padding: 8px;
    background: var(--bg-input);
    border-radius: 4px;
    white-space: pre-wrap;
    word-break: break-word;
    user-select: text;
}

/* Wizard */
.wizard-container {
    max-width: 560px;
    margin: 0 auto;
    padding: 40px 32px;
}

.wizard-header {
    text-align: center;
    margin-bottom: 32px;
}

.wizard-title {
    font-size: 24px;
    font-weight: 700;
    margin-bottom: 8px;
}

.wizard-subtitle {
    font-size: 14px;
    color: var(--text-secondary);
}

.wizard-footer {
    display: flex;
    justify-content: space-between;
    margin-top: 32px;
    gap: 12px;
}

.wizard-progress {
    display: flex;
    gap: 4px;
    justify-content: center;
    margin-bottom: 24px;
}

.progress-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--bg-hover);
}

.progress-dot.active {
    background: var(--accent);
}

.progress-dot.completed {
    background: var(--green);
}

/* Empty state */
.empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px;
    color: var(--text-secondary);
    gap: 8px;
}

/* Spinner */
.spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--border-input);
    border-top: 2px solid var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    display: inline-block;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}
"#;
