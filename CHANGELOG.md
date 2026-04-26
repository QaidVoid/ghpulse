
## 0.1.0 — 2026-04-26

### Bug Fixes

- Discover repos via contributions, correct stat totals, harden client
- Correct SVG text element rendering and always run aggregator

### Documentation

- Rewrite token permissions section
- Fix token permissions for classic and fine-grained PATs
- Add README

### Features

- Show up to 12 top languages across themes
- Add GitHub Action and install script to same repo
- Add custom theme loading from --theme-dir
- Add PNG export via resvg behind feature flag
- Add fingerprint waveform renderer
- Add enhanced contribution heatmap renderer
- Add spider/radar chart renderer
- Add retro terminal theme renderer
- Wire up collection, rendering, and output in main
- Add render module with nebula constellation renderer
- Add GraphQL and REST queries for GitHub data collection
- Add theme system with TOML loading and embedded defaults
- Add SVG builder primitives
- Add stats types, aggregator, and collector stubs
- Add GitHub HTTP client with auth and rate limiting
- Add CLI argument parsing with clap

### Refactor

- Replace blanket dead_code allows with surgical expect annotations


