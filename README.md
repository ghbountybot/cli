# bounty-cli

A CLI tool for managing GitHub bounty workflows, making it easier to work on GitHub issues by automating repository forking and branch setup.

## Features

- 🔄 Automatically fork repositories
- 📦 Clone repositories locally
- 🌿 Create and manage issue-specific branches
- 🔑 GitHub token integration
- 🐚 Shell completion support

## Installation

### From Git

```bash
cargo install --git https://github.com/ghbountybot/cli
```

### Requirements

- Rust (nightly toolchain)
- Git

## Usage

### Authentication

Set your GitHub token either via environment variable:
```bash
export GITHUB_TOKEN=your_github_token
```

Or pass it directly with the `--github-token` flag.

### Commands

Start working on a bounty:
```bash
bounty-cli bounty start owner/repo issue_number
```

Generate shell completions:
```bash
bounty-cli completion <shell>
```

## Development

This project uses:
- Rust nightly toolchain
- `clippy` with strict lints
- `tracing` for logging
- `eyre` for error handling
- `octocrab` for GitHub API integration

## License

MIT