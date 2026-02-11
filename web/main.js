import init, { validate, view, info } from './pkg/tree_doc_wasm.js';

let wasmReady = false;

async function initWasm() {
    await init();
    wasmReady = true;
}

initWasm().catch(err => {
    console.error('Failed to load WASM:', err);
});

// Drop zone handling
const dropZone = document.getElementById('drop-zone');
const fileInput = document.getElementById('file-input');
const results = document.getElementById('results');

dropZone.addEventListener('dragover', (e) => {
    e.preventDefault();
    dropZone.classList.add('drag-over');
});

dropZone.addEventListener('dragleave', () => {
    dropZone.classList.remove('drag-over');
});

dropZone.addEventListener('drop', (e) => {
    e.preventDefault();
    dropZone.classList.remove('drag-over');
    const file = e.dataTransfer.files[0];
    if (file) processFile(file);
});

dropZone.addEventListener('click', (e) => {
    if (e.target === fileInput || e.target.closest('.file-btn')) return;
    fileInput.click();
});

fileInput.addEventListener('change', () => {
    const file = fileInput.files[0];
    if (file) processFile(file);
});

function processFile(file) {
    if (!wasmReady) {
        alert('WASM module is still loading. Please try again.');
        return;
    }

    const reader = new FileReader();
    reader.onload = (e) => {
        const jsonStr = e.target.result;
        renderResults(jsonStr);
    };
    reader.readAsText(file);
}

function renderResults(jsonStr) {
    results.classList.remove('hidden');

    // Validation
    const valResult = validate(jsonStr);
    renderValidation(valResult);

    // Info
    const infoResult = info(jsonStr);
    renderInfo(infoResult);

    // View
    const viewResult = view(jsonStr);
    renderView(viewResult);
}

function renderValidation(result) {
    const el = document.getElementById('validation-result');

    if (result.error) {
        el.innerHTML = `<span class="valid-badge invalid">Parse Error</span><p style="margin-top:0.5rem;color:#f85149">${escapeHtml(result.error)}</p>`;
        return;
    }

    let html = `<span class="valid-badge ${result.isValid ? 'valid' : 'invalid'}">${result.isValid ? 'Valid' : 'Invalid'}</span>`;

    const allDiags = [
        ...(result.errors || []).map(d => ({ ...d, cls: 'error' })),
        ...(result.warnings || []).map(d => ({ ...d, cls: 'warning' })),
        ...(result.advisories || []).map(d => ({ ...d, cls: 'advisory' })),
    ];

    if (allDiags.length > 0) {
        html += '<ul class="diag-list">';
        for (const d of allDiags) {
            html += `<li class="diag-item ${d.cls}">
                <div>${escapeHtml(d.message)}</div>
                <div class="diag-rule">${escapeHtml(d.rule)}</div>
                <div class="diag-location">${escapeHtml(d.location)}</div>
            </li>`;
        }
        html += '</ul>';
    }

    el.innerHTML = html;
}

function renderInfo(result) {
    const el = document.getElementById('info-result');

    if (result.error) {
        el.innerHTML = `<p style="color:#f85149">${escapeHtml(result.error)}</p>`;
        return;
    }

    el.innerHTML = `<div class="info-grid">
        <span class="info-label">Tier</span><span class="info-value">${result.tier}</span>
        <span class="info-label">Nodes</span><span class="info-value">${result.nodeCount}</span>
        <span class="info-label">Edges</span><span class="info-value">${result.edgeCount}</span>
        <span class="info-label">Trunk length</span><span class="info-value">${result.trunkLength}</span>
        <span class="info-label">Branches</span><span class="info-value">${result.branchCount}</span>
    </div>`;
}

function renderView(result) {
    const el = document.getElementById('view-result');

    if (result.error) {
        el.innerHTML = `<p style="color:#f85149">${escapeHtml(result.error)}</p>`;
        return;
    }

    let html = `<h3 style="color:#f0f6fc;margin-bottom:0.25rem">${escapeHtml(result.title)}</h3>
        <p style="color:#8b949e;font-size:0.85rem;margin-bottom:1rem">${escapeHtml(result.stats)}</p>`;

    for (const step of result.steps) {
        html += `<div class="trunk-step">
            <span class="node-id">[${escapeHtml(step.nodeId)}]</span>
            <div class="content">${escapeHtml(step.content)}</div>`;

        if (step.trunkTarget) {
            html += `<div class="trunk-arrow">[trunk] &rarr; ${escapeHtml(step.trunkTarget)}</div>`;
        }

        if (step.branchCount > 0) {
            html += `<span class="branch-badge">+${step.branchCount} branch${step.branchCount === 1 ? '' : 'es'}</span>`;
            if (step.branchLabels && step.branchLabels.length > 0) {
                html += '<div class="branch-labels">';
                for (const label of step.branchLabels) {
                    html += `&middot; ${escapeHtml(label)}<br>`;
                }
                html += '</div>';
            }
        }

        if (step.isTerminal) {
            html += '<div class="terminal">(end of trunk)</div>';
        }

        html += '</div>';
    }

    el.innerHTML = html;
}

function escapeHtml(str) {
    if (typeof str !== 'string') return '';
    return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
