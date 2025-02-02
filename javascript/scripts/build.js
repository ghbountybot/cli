const { spawnSync } = require('child_process');
const fs = require('fs-extra');
const path = require('path');

const isRelease = process.argv.includes('--release');
const buildMode = isRelease ? 'release' : 'debug';

// Determine platform-specific binary extension
const extension = process.platform === 'win32' ? '.exe' : '';

// Paths
const projectRoot = path.resolve(__dirname, '../..');
const targetDir = path.join(projectRoot, 'target', buildMode);
const binaryName = `bounty${extension}`;
const binaryPath = path.join(targetDir, binaryName);

// The bin directory should be relative to the package root
const packageRoot = path.join(__dirname, '..');
const destDir = path.join(packageRoot, 'bin');
const destPath = path.join(destDir, binaryName);

// Ensure the binary directory exists
fs.mkdirpSync(destDir);

// Build the Rust project
console.log(`Building Rust project in ${buildMode} mode...`);
const buildArgs = ['build'];
if (isRelease) buildArgs.push('--release');

const buildResult = spawnSync('cargo', buildArgs, {
  stdio: 'inherit',
  cwd: projectRoot,
});

if (buildResult.status !== 0) {
  console.error('Failed to build Rust project');
  process.exit(1);
}

// Copy the binary
console.log(`Copying binary to ${destPath}`);
fs.copySync(binaryPath, destPath);
fs.chmodSync(destPath, 0o755); // Make the binary executable

console.log('Build completed successfully!'); 