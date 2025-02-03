const fs = require('fs-extra');
const path = require('path');

// Get the bin directory relative to this script
const binDir = path.join(__dirname, '../bin');

// Platform configurations (same as build.js)
const platforms = [
    {
        target: 'x86_64-unknown-linux-gnu',
        binaryName: 'bounty-x86_64-unknown-linux-gnu',
        sourceFile: 'bounty'
    },
    {
        target: 'x86_64-apple-darwin',
        binaryName: 'bounty-x86_64-apple-darwin',
        sourceFile: 'bounty'
    },
    {
        target: 'aarch64-apple-darwin',
        binaryName: 'bounty-aarch64-apple-darwin',
        sourceFile: 'bounty'
    },
    {
        target: 'x86_64-pc-windows-msvc',
        binaryName: 'bounty-x86_64-pc-windows-msvc.exe',
        sourceFile: 'bounty.exe'
    }
];

// Only run chmod on Unix-like systems
if (process.platform !== 'win32') {
    console.log('Attempting to set executable permissions on binaries...');
    
    try {
        // Set permissions on the wrapper script
        const wrapperPath = path.join(binDir, 'bounty.js');
        if (fs.existsSync(wrapperPath)) {
            fs.chmodSync(wrapperPath, 0o755);
        }

        // Set permissions on all platform binaries
        platforms.forEach(platform => {
            const binaryPath = path.join(binDir, platform.binaryName);
            if (fs.existsSync(binaryPath)) {
                try {
                    fs.chmodSync(binaryPath, 0o755);
                    console.log(`Set permissions for ${platform.binaryName}`);
                } catch (err) {
                    console.log(`Note: Could not set permissions for ${platform.binaryName} - this is OK`);
                }
            }
        });
    } catch (err) {
        console.log('Note: Could not set some executable permissions during install - this is OK');
    }
} else {
    console.log('Skipping permission setting on Windows');
} 