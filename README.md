<div align="center">

# ghpulse

**Your GitHub activity, visualized as generative SVG art.**

No two developers produce the same output.

</div>

---

## What it does

Every GitHub stats tool generates the same boring stat cards. Rectangles with numbers. Progress bars. Language percentages. It's been done a thousand times.

`ghpulse` turns your GitHub activity into **generative visual art** — unique to you, based on your actual data. Your repos, languages, contribution patterns, and activity shape the output.

## Themes

### `nebula` (default)

Your code universe as a cosmic map. Each repo is a star sized by contribution intensity, colored by primary language, connected by constellation lines when repos share languages. Stars pulse with a subtle twinkle animation.

### `terminal`

Cyberpunk retro-terminal aesthetic. Green-on-black monospace text. Stats typed out like you're hacking into mainframes in 1987. Blinking cursor included.

### `radar`

Spider/radar chart mapping your language dimensions. Axes generated from your actual language distribution. The shape of the radar is unique to your skill mix.

### `heatmap`

Enhanced contribution heatmap. Like GitHub's contribution graph but denser, with month labels, a legend, and your stats summary.

### `fingerprint`

A unique visual hash of your developer identity. Horizontal bars arranged like an audio waveform. The pattern is deterministic — same data always produces the same image.

## Usage

### CLI

```bash
# Generate with default theme (nebula)
ghpulse --token "$ACCESS_TOKEN"

# Pick a theme
ghpulse --token "$ACCESS_TOKEN" --theme terminal

# Exclude repos and languages
ghpulse --token "$ACCESS_TOKEN" --exclude-repos "test-*,forks/*" --exclude-langs "HTML,CSS"

# Save raw data, then re-render offline
ghpulse --token "$ACCESS_TOKEN" --dump-json stats.json
ghpulse --from-json stats.json --theme radar --output ./out

# PNG export (requires --features png)
ghpulse --token "$ACCESS_TOKEN" --format both
```

### All options

```
ghpulse [OPTIONS]

  -t, --token <TOKEN>              GitHub personal access token [env: ACCESS_TOKEN]
      --theme <THEME>              Visualization theme [default: nebula]
      --format <FORMAT>            Output format: svg, png, both [default: svg]
  -o, --output <DIR>               Output directory [default: .]
      --exclude-repos <PATTERNS>   Comma-separated glob patterns
      --exclude-langs <LANGS>      Comma-separated language names
      --exclude-private            Skip private repos
      --exclude-archived           Skip archived repos
      --min-stars <N>              Minimum stars to include a repo
      --min-commits <N>            Minimum commits to include a repo
      --dump-json <FILE>           Save raw stats to JSON
      --from-json <FILE>           Render from cached JSON (no API calls)
      --list-themes                List available themes
      --list-langs                 List detected languages
      --theme-dir <DIR>            Load custom themes from directory
      --max-retries <N>            Max retries for flaky API endpoints [default: 10]
      --verbose                    Verbose output
      --debug                      Debug output
  -V, --version                    Print version
```

### GitHub Action

Add this to your profile README repo:

```yaml
# .github/workflows/ghpulse.yml
name: Generate Stats

on:
  schedule:
    - cron: "0 0 * * *"
  workflow_dispatch:

permissions:
  contents: write

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: QaidVoid/ghpulse@v1
        with:
          token: ${{ secrets.ACCESS_TOKEN }}
          theme: nebula
          exclude-repos: "test-*"

      - name: Commit
        run: |
          git config user.name "ghpulse[bot]"
          git config user.email "ghpulse[bot]@users.noreply.github.com"
          git add .
          git commit -m "chore: update ghpulse stats" || true
          git push
```

Then in your README:

```markdown
![My GitHub Universe](./nebula.svg#gh-dark-mode-only)
![My GitHub Universe](./nebula-light.svg#gh-light-mode-only)
```

## Custom themes

Themes are TOML files. Drop them in a directory and point `--theme-dir` at it:

```toml
name = "My Theme"
background = "#0d1117"
foreground = "#c9d1d9"
accent = "#58a6ff"
font = "JetBrains Mono, monospace"
width = 800
height = 400

[star]
min_radius = 2
max_radius = 12
glow = true

[connection]
opacity = 0.15
width = 0.5

[animation]
twinkle = true
duration = "4s"
```

```bash
ghpulse --token "$TOKEN" --theme my-theme --theme-dir ~/.config/ghpulse/themes
```

## Token permissions

### Classic personal access token

| Scope | What it unlocks |
|-------|----------------|
| `read:user` | Public profile, repos, contribution data, language breakdown |
| `user:email` | Email addresses (for git attribution fallback) |
| `repo` | Private repos, traffic/view counts, contributor line stats |

Minimum for public-only data: `read:user`. Add `repo` if you want private repos and traffic.

### Fine-grained personal access token

Under **Account permissions**:
- **Email addresses** — Read-only

Under **Repository permissions**:
- **Metadata** — Read-only (always required)
- **Commit statuses** — Read-only
- **Content** — Read-only

For private repos and traffic data, set **Repository permissions**:
- **Administration** — Read-only

The default "Public Repositories (read-only)" preset covers most use cases if you only want public data.

## Build from source

```bash
git clone https://github.com/QaidVoid/ghpulse.git
cd ghpulse
cargo build --release

# with PNG support
cargo build --release --features png
```

## License

MIT OR Apache-2.0
