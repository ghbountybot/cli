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

```bash
bounty-cli --help
```

Generate shell completions:
```bash
bounty-cli completion <shell>
```
