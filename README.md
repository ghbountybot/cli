# bounty

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
