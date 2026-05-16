#!/usr/bin/env node
// CodeScope (cs) — npm binary wrapper
const { join, dirname } = require('path');
const { existsSync } = require('fs');
const { platform, arch } = require('process');

function getBinaryName() {
    if (platform() === 'win32') return 'cs.exe';
    return 'cs';
}

const binPath = join(__dirname, '..', 'bin', getBinaryName());

if (!existsSync(binPath)) {
    console.error('CodeScope binary not found. Run: npm install codescope');
    process.exit(1);
}

const { spawn } = require('child_process');
const child = spawn(binPath, process.argv.slice(2), { stdio: 'inherit' });
child.on('exit', (code) => process.exit(code || 0));
