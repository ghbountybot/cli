# bounty

[![CI](https://github.com/ghbountybot/cli/actions/workflows/rust.yml/badge.svg)](https://github.com/ghbountybot/cli/actions/workflows/rust.yml)
[![Nix](https://github.com/ghbountybot/cli/actions/workflows/nix.yml/badge.svg)](https://github.com/ghbountybot/cli/actions/workflows/nix.yml)
[![Npm](https://github.com/ghbountybot/cli/actions/workflows/npm-publish.yml/badge.svg)](https://github.com/ghbountybot/cli/actions/workflows/npm-publish.yml)


A CLI tool for ghbountybot.

## Running

### cargo

```bash
cargo install --git https://github.com/ghbountybot/cli
bounty
```

### nix

```bash
nix run github:ghbountybot/cli
```

## Usage

### Authentication

Set your GitHub token either via environment variable:
```bash
export GITHUB_TOKEN=your_github_token
```

Or pass it directly with the `--github-token` flag.

### Commands

```bash
bounty --help
```

Generate shell completions:
```bash
bounty completion <shell>
```

### Error Reporting
We use Sentry for error reporting and performance monitoring. If you don't want to send errors to Sentry, you can use `DISABLE_SENTRY=1` before running the CLI.