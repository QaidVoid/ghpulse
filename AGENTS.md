# AGENTS.md

Guidelines for AI agents working on the ghpulse project.

## Project Overview

ghpulse is a Rust CLI that collects GitHub user/repo statistics via the GitHub API and renders them into generative SVG visualizations. ghpulse ships as a pre-built binary and integrates via a thin GitHub Action.

## Development Workflow

### Commits

- **Commit in small chunks** — one logical change per commit
- **Never commit broken state** — all code must compile and pass tests
- **Format before commit** — run `cargo fmt` before every commit
- **Fix clippy issues** — run `cargo clippy` and address all warnings

### Commit Messages

Conventional commits, imperative mood:

```
type: message
```

Types: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`, `perf:`, `style:`, `ci:`

## Pre-commit Checklist

1. `cargo fmt`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all-features`
4. `cargo build`

## Project Structure

```
ghpulse/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry (clap)
│   ├── config.rs            # CLI args + TOML config
│   ├── github/
│   │   ├── client.rs        # ureq HTTP, auth, rate limiting
│   │   ├── graphql.rs       # GraphQL queries
│   │   └── rest.rs          # REST endpoints (traffic, emails, contributors)
│   ├── stats/
│   │   ├── types.rs         # User, Repo, Language, Stats (serde)
│   │   ├── collector.rs     # Orchestrate collection, pagination, retry
│   │   └── aggregator.rs    # Aggregate, dedup, filter, sort
│   ├── render/
│   │   ├── mod.rs           # Renderer trait, ThemeLoader
│   │   ├── context.rs       # Normalized data for renderers
│   │   ├── nebula.rs        # Constellation theme
│   │   ├── terminal.rs      # Retro terminal theme
│   │   ├── radar.rs         # Spider/radar chart
│   │   ├── heatmap.rs       # Enhanced contribution grid
│   │   └── fingerprint.rs   # Waveform visualization
│   ├── svg/
│   │   ├── mod.rs           # SVG builder primitives
│   │   └── theme.rs         # Theme struct, TOML loading, embedded defaults
│   └── output/
│       ├── mod.rs           # File/stdout writer
│       └── json.rs          # JSON dump
├── themes/                  # Built-in theme TOML (embedded via include_str!)
└── .github/workflows/
    ├── ci.yml               # fmt + clippy + test
    └── release.yml          # Cross-compile + GitHub Release
```

## Design Principles

- **Sync HTTP** — `ureq`. A CLI making sequential API calls doesn't need async.
- **Programmatic SVG** — no template engine. SVGs are built algorithmically for generative art.
- **Pre-built binary** — users never compile. The Action downloads a release binary.
- **Streaming parsing** — `serde_json::from_reader()` over buffering entire responses.
- **Offline re-render** — `--from-json` renders from cached data without API calls.

## Dependencies

| Purpose | Crate | Why |
|---------|-------|-----|
| HTTP | `ureq` | Sync, tiny, no runtime |
| CLI | `clap` (derive) | Standard |
| JSON | `serde` + `serde_json` | Standard |
| TOML | `toml` | Theme config |
| Logging | `tracing` | Structured |
| Errors | `anyhow` | App-level |

Avoid adding deps for things doable in a few dozen lines. PNG export (`resvg`) is behind a feature flag.

## Additional Notes

- Edition: Rust 2024
- No async runtime — the entire project is synchronous
- Git fallback (for lines-changed) shells out to `git`, no libgit2
- Themes are embedded at compile time via `include_str!`, custom themes loaded at runtime via `--theme-dir`
