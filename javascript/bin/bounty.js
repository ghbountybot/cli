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
    case 'darwin-arm64':
        binaryName = 'bounty-aarch64-apple-darwin';
        break;
    case 'win32-x64':
        binaryName = 'bounty-x86_64-pc-windows-msvc.exe';
        break;
    default:
        console.error(`It doesn't seem that BountyBot supports your platform (${platform}-${arch}) yet!`);
        console.log(`Please make a issue here, https://github.com/ghbountybot/cli/issues. Include your platform: ${platform}-${arch}.`)
        process.exit(1);
}

const binaryPath = path.join(__dirname, binaryName);

if (!fs.existsSync(binaryPath)) {
    console.error(`Binary not found: ${binaryPath}`);
    process.exit(1);
}

// Check executable permissions on Unix-like systems
if (platform !== 'win32') {
    try {
        const stat = fs.statSync(binaryPath);
        // Check if file is executable for user
        if ((stat.mode & 0o100) === 0) {
            try {
                fs.chmodSync(binaryPath, 0o755);
            } catch (error) {
                console.error('\nError: Binary is not executable and could not set permissions automatically.');
                console.error(`Please run: sudo chmod +x "${binaryPath}"\n`);
                process.exit(1);
            }
        }
    } catch (error) {
        console.error('\nError: Could not check binary permissions.');
        console.error(`Please run: sudo chmod +x "${binaryPath}"\n`);
        process.exit(1);
    }
}

// Execute the binary with the same arguments
const result = spawnSync(binaryPath, process.argv.slice(2), {
    stdio: 'inherit'
});

process.exit(result.status ?? 0); 