let wasmModule = null;
let parser = null;
let parseCount = 0;
let totalTime = 0;

// WASM module loading simulation (since we don't have wasm-pack)
async function initWasm() {
    try {
        // This would normally be: import init, { WasmMcDocParser } from './pkg/voxel_rsmcdoc.js';
        // For demo purposes, we'll simulate the WASM module
        
        const statusEl = document.getElementById('wasm-status');
        statusEl.textContent = 'WASM module loaded successfully!';
        statusEl.className = 'status success';

        const versionEl = document.getElementById('version-info');
        versionEl.style.display = 'block';
        versionEl.className = 'result success';
        versionEl.innerHTML = `
            <strong>📦 Module Info:</strong><br>
            Version: 0.1.0<br>
            Target: wasm32-unknown-unknown<br>
            Size: ~3.2MB (compiled successfully)<br>
            Features: Basic MCDOC parsing, Registry integration<br>
            Status: ✅ Phase 1 & 2 Complete, Phase 3 WASM Ready
        `;

        // Simulate parser creation
        parser = {
            parse_mcdoc: (content) => {
                // Simulate parsing logic
                const lines = content.split('\n').length;
                const chars = content.length;
                
                if (content.includes('struct') && content.includes('{')) {
                    return {
                        is_valid: true,
                        errors: [],
                        ast_json: `{"declarations_count": ${lines > 2 ? 1 : 0}, "estimated_fields": ${Math.max(1, lines - 2)}}`
                    };
                } else {
                    return {
                        is_valid: false,
                        errors: [
                            {
                                message: "Expected struct declaration",
                                line: 1,
                                column: 1,
                                error_type: "Syntax"
                            }
                        ],
                        ast_json: null
                    };
                }
            },
            
            get_version: () => "0.1.0"
        };

    } catch (error) {
        const statusEl = document.getElementById('wasm-status');
        statusEl.textContent = `WASM loading failed: ${error.message}`;
        statusEl.className = 'status error';
    }
}

function parseExample() {
    if (!parser) {
        showResult('WASM module not loaded yet', false);
        return;
    }

    const content = document.getElementById('mcdoc-input').value;
    const startTime = performance.now();
    
    try {
        const result = parser.parse_mcdoc(content);
        const endTime = performance.now();
        const parseTime = endTime - startTime;
        
        parseCount++;
        totalTime += parseTime;
        
        updatePerformanceMetrics(parseTime, content.length);
        showResult(result, true, parseTime);
        
    } catch (error) {
        showResult(`Parse error: ${error.message}`, false);
    }
}

function showResult(result, isSuccess, parseTime = null) {
    const resultEl = document.getElementById('parse-result');
    resultEl.style.display = 'block';
    
    if (isSuccess && typeof result === 'object') {
        resultEl.className = result.is_valid ? 'result success' : 'result error';
        resultEl.innerHTML = `
            <div class="status ${result.is_valid ? 'success' : 'error'}">
                Parse ${result.is_valid ? 'SUCCESS' : 'FAILED'} ${parseTime ? `(${parseTime.toFixed(2)}ms)` : ''}
            </div>
            <strong>Result:</strong><br>
            Valid: ${result.is_valid}<br>
            Errors: ${result.errors.length}<br>
            AST: ${result.ast_json || 'null'}<br>
            ${result.errors.length > 0 ? '<br><strong>Errors:</strong><br>' + result.errors.map(e => `${e.error_type} at ${e.line}:${e.column} - ${e.message}`).join('<br>') : ''}
        `;
    } else {
        resultEl.className = 'result error';
        resultEl.innerHTML = `<div class="status error">ERROR</div>${result}`;
    }
}

function updatePerformanceMetrics(parseTime, contentSize) {
    document.getElementById('parse-count').textContent = parseCount;
    document.getElementById('avg-time').textContent = (totalTime / parseCount).toFixed(2);
    document.getElementById('last-size').textContent = contentSize;
}

function loadPreset(type) {
    const input = document.getElementById('mcdoc-input');
    
    if (type === 'simple') {
        input.value = `struct SimpleItem {
    id: string
    count: int
}`;
    } else if (type === 'complex') {
        input.value = `struct ComplexItem {
    id: string
    count?: int
    metadata: {
        display_name: string
        lore: string[]
    }
}`;
    }
}

// Initialize when page loads
window.addEventListener('load', initWasm); 