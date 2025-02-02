const { spawnSync } = require('child_process');
const fs = require('fs-extra');
const path = require('path');

const isRelease = process.argv.includes('--release');
const buildMode = isRelease ? 'release' : 'debug';

// Paths
const projectRoot = path.resolve(__dirname, '../..');
const packageRoot = path.join(__dirname, '..');
const destDir = path.join(packageRoot, 'bin');

// Platform configurations
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

// Ensure the binary directory exists
fs.mkdirpSync(destDir);

// Copy the wrapper script
fs.copyFileSync(
    path.join(__dirname, '../bin/bounty.js'),
    path.join(destDir, 'bounty.js')
);
fs.chmodSync(path.join(destDir, 'bounty.js'), 0o755);

if (process.env.CI) {
    // In CI, the workflow handles building and copying binaries
    console.log('Running in CI - binaries will be handled by the workflow');
} else {
    // For local development, build only for the current platform
    let target;
    const platform = process.platform;
    const arch = process.arch;

    // Map current platform to target triple
    if (platform === 'linux' && arch === 'x64') {
        target = 'x86_64-unknown-linux-gnu';
    } else if (platform === 'darwin') {
        target = arch === 'arm64' ? 'aarch64-apple-darwin' : 'x86_64-apple-darwin';
    } else if (platform === 'win32' && arch === 'x64') {
        target = 'x86_64-pc-windows-msvc';
    } else {
        console.error(`Unsupported platform: ${platform}-${arch}`);
        process.exit(1);
    }

    const platformConfig = platforms.find(p => p.target === target);

    console.log(`Building for current platform (${target})...`);
    
    const buildResult = spawnSync('cargo', ['build', isRelease ? '--release' : ''].filter(Boolean), {
        stdio: 'inherit',
        cwd: projectRoot,
    });

    if (buildResult.status !== 0) {
        console.error('Failed to build');
        process.exit(1);
    }

    const srcPath = path.join(projectRoot, 'target', buildMode, platformConfig.sourceFile);
    const destPath = path.join(destDir, platformConfig.binaryName);
    
    fs.copyFileSync(srcPath, destPath);
    fs.chmodSync(destPath, 0o755);
}

console.log('Build completed successfully!'); 