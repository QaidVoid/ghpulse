# ghpulse — Plan

> Your GitHub activity, visualized as something actually interesting.

## Why

Every GitHub stats tool generates the same boring stat cards. Rectangles with
numbers. Progress bars. Language percentages. It's been done a thousand times.

`ghpulse` does something different. It turns your GitHub activity into
**generative visual art** — unique to you, based on your actual data. No two
developers produce the same output.

The current tool this replaces (a Zig-based github-stats fork) also suffers
from memory issues in CI, requires users to copy the full source as a template,
and builds from scratch every run. `ghpulse` ships as a pre-built binary and
installs via a one-line GitHub Action.


## Name

**`ghpulse`** — short, implies living/dynamic data, sounds like a tool not a
generic "stats" thing.

Alternatives considered: `ghviz`, `codetrail`, `ghsignature`, `devfingerprint`.


## The Unique Part: Generative Visualization

Instead of card-style output, `ghpulse` renders your GitHub data into themed,
generative SVG art. The data drives the visuals — your repos, languages,
contribution patterns, and activity shape the output.

### Built-in Themes

#### `nebula` (default)

Your code universe as a cosmic map.

- Each repo is a **star** — sized by your contribution intensity, colored by
  primary language, brightness tied to view count
- Repos sharing languages are connected by faint **constellation lines**
- Repos naturally **cluster** by language ecosystem (Rust projects near each
  other, Python projects near each other, etc.)
- Subtle CSS **twinkle animation** in the SVG
- Dark/light theme auto-switching
- Your most-active language gets a subtle **nebula glow** in its color

The result: a unique visual fingerprint. No two developers look the same.

#### `terminal`

Cyberpunk retro-terminal aesthetic.

- Monospace font, green/amber text on black background
- CRT scan-line overlay effect
- Stats displayed as if typed out on a terminal
- ASCII-art borders and decorations
- "Booting up..." loading animation in CSS
- Feels like you're hacking into mainframes in 1987

#### `radar`

Spider/radar chart mapping your skill dimensions.

- Axes generated from your actual language distribution:
  "Systems", "Web Frontend", "Web Backend", "Mobile", "Data/ML",
  "Scripting", "Infrastructure", etc.
- The shape of the radar is unique to your skill mix
- Filled area with language-color gradient
- Repo count and activity intensity scale each axis

#### `heatmap`

Enhanced contribution heatmap.

- Like GitHub's contribution graph but with per-day **language breakdown**
  (each square is a mini stacked bar, not just a green block)
- Row per year, column per week
- Hover tooltip (in supported renderers) showing language breakdown
- Optional: overlay significant events (first PR to a popular repo, etc.)

#### `fingerprint`

A unique visual hash of your developer identity.

- Horizontal bars arranged like an audio waveform / DNA strand
- Each bar represents a dimension (repos, stars, forks, languages, etc.)
- The pattern is deterministic — same data always produces the same image
- Think: GitHub's contribution graph meets a soundwave visualization
- Instantly recognizable as "you" at a glance

### Theme System

Themes are defined in TOML. Users can override colors, fonts, sizes, and layout
parameters. The built-in themes are embedded in the binary, but a `--theme-dir`
flag allows loading custom themes from disk.

```toml
# example: ~/.config/ghpulse/themes/my-theme.toml
[theme]
name = "My Custom Theme"
background = "#0d1117"
foreground = "#c9d1d9"
accent = "#58a6ff"
font = "JetBrains Mono, monospace"

[theme.star]
min_radius = 2
max_radius = 12
glow = true

[theme.connection]
opacity = 0.15
width = 0.5

[theme.animation]
twinkle = true
duration = "4s"
```

### Light/Dark Mode

All themes support automatic light/dark switching in GitHub READMEs using the
existing `#gh-dark-mode-only` / `#gh-light-mode-only` CSS fragment technique.
The SVG renders both variants and hides one via CSS `:target` selectors.


## Architecture

### Crate Layout

```
ghpulse/
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── main.rs                # CLI entry, clap
│   ├── config.rs              # Args + optional TOML config file
│   ├── github/
│   │   ├── mod.rs
│   │   ├── client.rs          # ureq HTTP, auth headers, rate limiting
│   │   ├── graphql.rs         # All GraphQL queries
│   │   └── rest.rs            # REST endpoints (traffic, emails, contributors)
│   ├── stats/
│   │   ├── mod.rs
│   │   ├── types.rs           # User, Repo, Language, Stats structs (serde)
│   │   ├── collector.rs       # Orchestrate data collection, pagination, retry
│   │   └── aggregator.rs      # Aggregate, dedup, filter, sort, compute totals
│   ├── render/
│   │   ├── mod.rs             # ThemeLoader, Renderer trait
│   │   ├── context.rs         # RenderContext — normalized data for templates
│   │   ├── nebula.rs          # Nebula constellation renderer
│   │   ├── terminal.rs        # Retro terminal renderer
│   │   ├── radar.rs           # Spider/radar chart renderer
│   │   ├── heatmap.rs         # Enhanced contribution heatmap renderer
│   │   └── fingerprint.rs     # Audio-waveform / DNA strand renderer
│   ├── svg/
│   │   ├── mod.rs             # SVG builder primitives (rect, circle, text, etc.)
│   │   └── theme.rs           # Theme struct, TOML loading, embedded defaults
│   └── output/
│       ├── mod.rs             # Output writer (file, stdout)
│       └── json.rs            # JSON dump
├── themes/                    # Built-in theme TOML files (embedded at compile time)
│   ├── nebula-dark.toml
│   ├── nebula-light.toml
│   ├── terminal-dark.toml
│   ├── radar-dark.toml
│   └── ...
├── .github/
│   └── workflows/
│       ├── ci.yml             # test + clippy + fmt check
│       └── release.yml        # Cross-compile + GitHub Release
└── PLAN.md                    # This file
```

### Key Dependencies

| Concern         | Crate           | Why                                        |
|-----------------|-----------------|--------------------------------------------|
| HTTP            | `ureq`          | Sync, tiny, no tokio. Perfect for a CLI.   |
| JSON            | `serde` + `serde_json` | Obvious choice.                     |
| CLI             | `clap` (derive) | Standard, powerful, derive macros.         |
| SVG generation  | Custom builder  | Full control for generative/algorithmic art. No template engine — we generate SVG programmatically. |
| TOML            | `toml`          | Theme config parsing.                      |
| Logging         | `tracing`       | Structured, composable, standard.          |
| Errors          | `anyhow`        | App-level error handling.                  |
| Rate limiting   | Custom (simple) | Parse `x-ratelimit-*` headers, sleep.      |
| Git fallback    | Shell out to `git` | For lines-changed when API fails. Simple, no libgit2 dependency. |
| PNG export      | `resvg`         | SVG → PNG without a browser. Optional feature flag. |

### Data Flow

```
CLI args
  │
  ▼
┌─────────────┐     ┌──────────────┐
│ config.rs   │────▶│ github/      │──▶ GitHub API (GraphQL + REST)
└─────────────┘     │ collector.rs │
                    └──────┬───────┘
                           │ raw data
                           ▼
                    ┌──────────────┐
                    │ stats/       │──▶ Aggregated Stats struct
                    │ aggregator   │    (repos, langs, totals, filtered)
                    └──────┬───────┘
                           │
                    ┌──────┴──────┐
                    │             │
                    ▼             ▼
             ┌──────────┐  ┌──────────┐
             │ render/  │  │ output/  │
             │ <theme>  │  │ json.rs  │──▶ stats.json
             └────┬─────┘  └──────────┘
                  │ SVG string
                  ▼
           ┌────────────┐
           │ output/    │──▶ nebula.svg, terminal.svg, etc.
           │ mod.rs     │    (or .png if --format png)
           └────────────┘
```


## Data Collection

### What We Collect (via GitHub API)

From **GraphQL**:
- User login, display name
- Contribution years
- Per-year contribution counts (repos, issues, commits, PRs, reviews)
- Per-year commit-contributed repos with:
  - Name, stars, forks, isPrivate
  - Languages (name, size, color)

From **REST**:
- User email addresses (for git attribution)
- Per-repo traffic/views (requires push access)
- Per-repo contributor stats (lines added/removed)

### Rate Limiting Strategy

- Parse `x-ratelimit-remaining` and `x-ratelimit-reset` from every response
- When approaching limit, sleep until reset time
- For `stats/contributors` (notoriously flaky): retry with random backoff,
  then fall back to shallow `git clone` + `git log --numstat --author`
- Pagination: handle `hasNextPage` / `endCursor` for GraphQL, `Link` header for REST

### Filtering

- `--exclude-repos` — glob patterns (e.g. `"jstrieb/*"`, `"test-*"`)
- `--exclude-langs` — language names (e.g. `"HTML,CSS"`)
- `--exclude-private` — skip private repos entirely
- `--exclude-archived` — skip archived repos
- `--min-stars` / `--min-commits` — only include repos meeting threshold

### Caching

- `--dump-json stats.json` — save collected data
- `--from-json stats.json` — render from cached data (zero API calls)
- This enables: collect once, re-render with different themes without hitting the API
- Also useful for local analysis, debugging, and developing new themes


## SVG Rendering

### Why Programmatic, Not Templates

The current tool uses string-interpolated SVG templates. That works for static
cards but breaks down for generative art where element positions, sizes, and
connections are computed algorithmically.

Instead, `ghpulse` builds SVGs programmatically:

```rust
// sketch of the SVG builder
let mut doc = Svg::new(800, 400);

// background
doc.rect(0, 0, 800, 400).fill(theme.background).rx(12);

// stars (repos)
for repo in &repos {
    let (x, y) = layout.position(repo);
    let r = layout.radius(repo);
    doc.circle(x, y, r)
        .fill(repo.primary_language_color())
        .opacity(repo.activity_opacity())
        .filter("glow");  // SVG filter for glow effect
}

// connections (shared languages)
for (a, b) in layout.connections() {
    doc.line(a.pos, b.pos)
        .stroke(theme.connection_color)
        .stroke_width(theme.connection_width)
        .opacity(theme.connection_opacity);
}
```

This gives us full control over positioning algorithms (force-directed layout,
circular packing, etc.) while still producing clean SVG output.

### Layout Algorithms

Different themes use different spatial layouts:

- **Nebula**: Force-directed graph layout — repos repel each other, shared-language
  repos attract. Results in organic, natural-looking clusters.
- **Radar**: Circular with evenly-spaced axes.
- **Heatmap**: Grid (weeks × years).
- **Fingerprint**: Horizontal bars, centered vertically.
- **Terminal**: Vertical text flow, no spatial algorithm needed.

### Animations

SVG supports CSS animations. We use them:

- Nebula stars gently **pulse** (scale + opacity oscillation)
- Terminal text **types in** character by character
- Radar chart **fills in** on load
- Heatmap squares **fade in** from left to right

These work in GitHub READMEs (GitHub sanitizes SVGs but allows CSS animations
inside `<style>` tags within the SVG).


## Deployment Model

### Two Repositories

```
qaidvoid/ghpulse          →  Rust source + CI + releases (this repo)
qaidvoid/ghpulse-action   →  GitHub Action wrapper (action.yml + install script)
```

### Binary Releases

Every tag (`v1.0.0`, `v1.1.0`, etc.) triggers a release build:

```yaml
# .github/workflows/release.yml
on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: taiki-e/upload-rust-binary-action@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          target: ${{ matrix.target }}
          archive: tar
```

Release binaries are named predictably:
```
ghpulse-x86_64-unknown-linux-gnu.tar.gz
ghpulse-aarch64-unknown-linux-gnu.tar.gz
ghpulse-x86_64-apple-darwin.tar.gz
ghpulse-aarch64-apple-darwin.tar.gz
ghpulse-x86_64-pc-windows-msvc.zip
```

### The Action (`ghpulse-action`)

A tiny composite action. No source code, no build step:

```yaml
# qaidvoid/ghpulse-action/action.yml
name: "ghpulse"
description: "Generate unique GitHub stats visualizations"
inputs:
  token:
    description: "GitHub personal access token"
    required: true
  theme:
    description: "Visualization theme (nebula, terminal, radar, heatmap, fingerprint)"
    default: "nebula"
  output-dir:
    description: "Directory to write output files"
    default: "./"
  format:
    description: "Output format (svg, png, both)"
    default: "svg"
  exclude-repos:
    description: "Comma-separated glob patterns of repos to exclude"
    required: false
  exclude-langs:
    description: "Comma-separated language names to exclude"
    required: false
  exclude-private:
    description: "Exclude private repositories"
    default: "false"
  max-retries:
    description: "Max retries for flaky API endpoints"
    default: "10"
  version:
    description: "ghpulse version to use"
    default: "latest"

runs:
  using: "composite"
  steps:
    - name: Install ghpulse
      shell: bash
      run: ${{ github.action_path }}/install.sh
      env:
        GHPULSE_VERSION: ${{ inputs.version }}

    - name: Generate visualizations
      shell: bash
      run: |
        ARGS=(
          --token "${{ inputs.token }}"
          --theme "${{ inputs.theme }}"
          --output "${{ inputs.output-dir }}"
          --format "${{ inputs.format }}"
          --max-retries "${{ inputs.max-retries }}"
        )
        if [ "${{ inputs.exclude-repos }}" != "" ]; then
          ARGS+=(--exclude-repos "${{ inputs.exclude-repos }}")
        fi
        if [ "${{ inputs.exclude-langs }}" != "" ]; then
          ARGS+=(--exclude-langs "${{ inputs.exclude-langs }}")
        fi
        if [ "${{ inputs.exclude-private }}" == "true" ]; then
          ARGS+=(--exclude-private)
        fi
        ghpulse "${ARGS[@]}"

branding:
  icon: "activity"
  color: "purple"
```

### Install Script

```bash
#!/usr/bin/env bash
# install.sh — downloads ghpulse binary from GitHub Releases
set -euo pipefail

VERSION="${GHPULSE_VERSION:-latest}"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

# Normalize
case "$OS" in
  linux)  TARGET_OS="unknown-linux-gnu" ;;
  darwin) TARGET_OS="apple-darwin" ;;
  mingw*|msys*|cygwin*) TARGET_OS="pc-windows-msvc"; EXT=".zip" ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64)  TARGET_ARCH="x86_64" ;;
  aarch64|arm64) TARGET_ARCH="aarch64" ;;
  *) echo "Unsupported arch: $ARCH"; exit 1 ;;
esac

EXT="${EXT:-.tar.gz}"
TARGET="${TARGET_ARCH}-${TARGET_OS}"

if [ "$VERSION" = "latest" ]; then
  DOWNLOAD_URL="https://github.com/qaidvoid/ghpulse/releases/latest/download/ghpulse-${TARGET}${EXT}"
else
  DOWNLOAD_URL="https://github.com/qaidvoid/ghpulse/releases/download/${VERSION}/ghpulse-${TARGET}${EXT}"
fi

echo "Downloading ghpulse ($TARGET) from $DOWNLOAD_URL"
curl -fsSL "$DOWNLOAD_URL" | tar xz -C /usr/local/bin ghpulse
chmod +x /usr/local/bin/ghpulse
ghpulse --version
```

### User Workflow

Users add this to their profile README repo:

```yaml
# .github/workflows/ghpulse.yml
name: Generate Stats

on:
  schedule:
    - cron: "0 0 * * *"   # daily
  workflow_dispatch:

permissions:
  contents: write

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: qaidvoid/ghpulse-action@v1
        with:
          token: ${{ secrets.ACCESS_TOKEN }}
          theme: nebula
          exclude-repos: "ghpulse,private-*"

      - name: Commit
        run: |
          git config user.name "ghpulse[bot]"
          git config user.email "ghpulse[bot]@users.noreply.github.com"
          git add .
          git commit -m "chore: update ghpulse stats" || true
          git push
```

Then in their README:

```markdown
![My GitHub Universe](./nebula.svg#gh-dark-mode-only)
![My GitHub Universe](./nebula-light.svg#gh-light-mode-only)
```


## CLI Reference

```
ghpulse [OPTIONS]

Options:
  -t, --token <TOKEN>              GitHub personal access token
                                    [env: ACCESS_TOKEN, GITHUB_TOKEN]
      --theme <THEME>              Visualization theme
                                    [default: nebula]
                                    [possible: nebula, terminal, radar, heatmap, fingerprint]
      --format <FORMAT>            Output format [default: svg]
                                    [possible: svg, png, both]
  -o, --output <DIR>               Output directory [default: .]
      --exclude-repos <PATTERNS>   Comma-separated glob patterns
      --exclude-langs <LANGS>      Comma-separated language names
      --exclude-private            Skip private repos
      --exclude-archived           Skip archived repos
      --min-stars <N>              Min stars to include a repo
      --min-commits <N>            Min commits to include a repo
      --dump-json <FILE>           Save raw stats to JSON
      --from-json <FILE>           Render from cached JSON (no API calls)
      --list-themes                List available themes
      --list-langs                 List detected languages (for exclude-langs)
      --theme-dir <DIR>            Load additional themes from directory
      --max-retries <N>            Max retries for flaky API endpoints [default: 10]
      --verbose                    Verbose output
      --debug                      Debug output
  -v, --version                    Print version
  -h, --help                       Print help
```


## Memory Strategy

The current tool runs out of memory in GitHub Actions. `ghpulse` avoids this:

1. **Pre-built binary** — zero compilation in CI. The action downloads a
   ~3-5MB static binary. No build step, no compiler, no dependency resolution.

2. **Streaming JSON parsing** — `ureq` returns a `Read` trait. We can parse
   GitHub API responses with `serde_json::from_reader()` which streams rather
   than buffering entire responses into memory first.

3. **Bounded concurrency** — Since we're using `ureq` (sync), requests are
   naturally sequential. No risk of spawning hundreds of concurrent connections
   that each buffer a response.

4. **Shallow clones for git fallback** — `git clone --depth=1 --single-branch`
   + streaming `git log --numstat` via `std::process::Command`. Pipe the output,
   don't buffer it.

5. **Arena allocation for temporary data** — GitHub API responses are parsed
   into temporary structs, relevant data is extracted into compact owned types,
   then the temporaries are dropped. No accumulation of raw API responses.


## Implementation Phases

### Phase 0 — Scaffolding
- [ ] `cargo init ghpulse` with `Cargo.toml` deps
- [ ] `src/main.rs` with clap CLI skeleton
- [ ] `src/config.rs` for args + config
- [ ] `src/github/client.rs` — ureq HTTP client with auth + rate limit parsing
- [ ] Basic CI workflow (fmt + clippy + test)

### Phase 1 — Data Collection
- [ ] `src/github/graphql.rs` — user info, contribution years, repos query
- [ ] `src/github/rest.rs` — emails, traffic/views, contributor stats
- [ ] `src/stats/types.rs` — core data types with serde
- [ ] `src/stats/collector.rs` — orchestrate collection with pagination + retry
- [ ] `src/stats/aggregator.rs` — aggregate, dedup, filter, sort
- [ ] `--dump-json` and `--from-json` support

### Phase 2 — First Visualization
- [ ] `src/svg/mod.rs` — SVG builder primitives
- [ ] `src/render/mod.rs` — Renderer trait + ThemeLoader
- [ ] `src/render/nebula.rs` — The flagship theme
- [ ] Force-directed layout algorithm
- [ ] Light/dark dual-render
- [ ] Built-in theme TOML files, embedded at compile time

### Phase 3 — More Themes
- [ ] `src/render/terminal.rs` — retro terminal
- [ ] `src/render/radar.rs` — spider/radar chart
- [ ] `src/render/heatmap.rs` — enhanced contribution grid
- [ ] `src/render/fingerprint.rs` — waveform visualization

### Phase 4 — Polish & Output
- [ ] PNG export via `resvg` (behind feature flag)
- [ ] Custom theme loading (`--theme-dir`)
- [ ] `--list-themes` and `--list-langs`
- [ ] Filtering options (`--exclude-archived`, `--min-stars`, etc.)

### Phase 5 — Release Infrastructure
- [ ] Cross-platform release workflow
- [ ] `ghpulse-action` repo: `action.yml` + `install.sh`
- [ ] Test the action end-to-end on a test repo
- [ ] Publish to GitHub Marketplace

### Phase 6 — Extras
- [ ] Contribution heatmap per-repo (deeper data)
- [ ] Trend analysis (compare stats over time from cached JSONs)
- [ ] Config file support (`~/.config/ghpulse/config.toml`)
- [ ] Shell completions (bash, zsh, fish)


## License

MIT or Apache-2.0 (dual, like most Rust projects). TBD.
