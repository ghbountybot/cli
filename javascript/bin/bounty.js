#!/usr/bin/env node

const { spawnSync } = require('child_process');
const path = require('path');
const fs = require('fs');
const Sentry = require("@sentry/node");
const { nodeProfilingIntegration } = require("@sentry/profiling-node");

// Check if Sentry is disabled via environment variable
const isSentryDisabled = process.env.DISABLE_SENTRY !== undefined;

// Initialize Sentry only if not disabled
if (!isSentryDisabled) {
    Sentry.init({
        dsn: "https://2e69e6b06c4b819ed15ca37ad377234b@o4508680996454400.ingest.us.sentry.io/4508854467559424",
        integrations: [
            nodeProfilingIntegration(),
        ],
        // Performance Monitoring
        tracesSampleRate: 1.0, // Capture 100% of the transactions
    });

    // Start profiling for the main CLI execution
    Sentry.profiler.startProfiler();
}

try {
    // Start a transaction for the CLI execution if Sentry is enabled
    const runCli = () => {
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
                console.error(`\nIt doesn't seem that BountyBot supports your platform (${platform}-${arch}) yet!`);
                console.log(`Please make an issue here: https://github.com/ghbountybot/cli/issues`);
                console.log(`Include your platform: ${platform}-${arch}\n`);
                throw new Error("Unsupported platform");
        }

        const binaryPath = path.join(__dirname, binaryName);

        if (!fs.existsSync(binaryPath)) {
            console.error('\nError: Could not find the BountyBot binary.');
            console.error(`Expected location: ${binaryPath}`);
            console.error('Try reinstalling the package with: npm install -g bountybot\n');
            throw new Error("Binary not found");
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
                        console.error('\nError: The BountyBot binary is not executable and we could not set permissions automatically.');
                        console.error(`Please run this command to fix it:`);
                        console.error(`sudo chmod +x "${binaryPath}"\n`);
                        throw new Error("Binary permissions error");
                    }
                }
            } catch (error) {
                console.error('\nError: Could not check binary permissions.');
                console.error(`Please run this command to fix it:`);
                console.error(`sudo chmod +x "${binaryPath}"\n`);
                throw new Error("Binary permissions error");
            }
        }

        // Execute the binary with the same arguments
        const result = spawnSync(binaryPath, process.argv.slice(2), {
            stdio: 'inherit'
        });

        process.exit(result.status ?? 0);
    };

    // Start a transaction for the CLI execution
    if (!isSentryDisabled) {
        Sentry.startSpan({
            name: "CLI Execution",
            op: "cli.run",
        }, runCli);
    } else {
        runCli();
    }
} catch (error) {
    if (!isSentryDisabled) {
        Sentry.captureException(error);
        
        // Stop profiler and ensure all Sentry events are sent before exiting
        Sentry.profiler.stopProfiler();
        Sentry.close(2000).then(() => {
            process.exit(1);
        });
    } else {
        console.error(error);
        process.exit(1);
    }
} 