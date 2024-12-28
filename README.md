# bounty-cli

A CLI tool for ghbountybot.

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
