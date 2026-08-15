#!/usr/bin/env node

const { spawn } = require('child_process');

function runMcp() {
    // Try distaff mcp first
    const child = spawn('distaff', ['mcp'], {
        stdio: 'inherit',
        shell: true
    });

    child.on('error', () => {
        // Fall back to cargo run if distaff binary is not in PATH
        const fallback = spawn('cargo', ['run', '-q', '--bin', 'threadloom-mcp'], {
            stdio: 'inherit',
            shell: true
        });

        fallback.on('error', (err) => {
            console.error('Failed to start Threadloom MCP server:', err.message);
            console.error('Make sure `cargo` or `distaff` is installed.');
            process.exit(1);
        });

        fallback.on('exit', (code) => {
            process.exit(code || 0);
        });
    });

    child.on('exit', (code) => {
        if (code !== 0 && code !== null) {
            process.exit(code);
        }
    });
}

runMcp();
