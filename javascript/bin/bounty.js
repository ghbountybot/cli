#!/usr/bin/env node

const { spawnSync } = require('child_process');
const path = require('path');
const fs = require('fs');

// Determine platform and architecture
const platform = process.platform;
const arch = process.arch;

// Map to binary name
let binaryName;
switch (`${platform}-${arch}`) {
    case 'linux-x64':
        binaryName = 'bounty-x86_64-unknown-linux-gnu';
        break;
    case 'darwin-x64':
        binaryName = 'bounty-x86_64-apple-darwin';
        break;
    case 'darwin-arm64':
        binaryName = 'bounty-aarch64-apple-darwin';
        break;
    case 'win32-x64':
        binaryName = 'bounty-x86_64-pc-windows-msvc.exe';
        break;
    default:
        console.error(`Unsupported platform: ${platform}-${arch}`);
        process.exit(1);
}

const binaryPath = path.join(__dirname, binaryName);

if (!fs.existsSync(binaryPath)) {
    console.error(`Binary not found: ${binaryPath}`);
    process.exit(1);
}

// Execute the binary with the same arguments
const result = spawnSync(binaryPath, process.argv.slice(2), {
    stdio: 'inherit'
});

process.exit(result.status ?? 0); 