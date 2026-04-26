use anyhow::Result;

use crate::render::context::{RenderContext, RepoContext};
use crate::svg::Svg;
use crate::svg::theme::Theme;

/// Render the nebula constellation visualization.
pub fn render(ctx: &RenderContext, theme: &Theme) -> Result<String> {
    let mut doc = Svg::new(theme.width, theme.height);

    // Background.
    doc.add(
        doc.rect(0.0, 0.0, theme.width as f64, theme.height as f64)
            .fill(&theme.background)
            .rx(12.0),
    );

    // CSS animations.
    if theme.animation.twinkle {
        doc.style(&format!(
            "@keyframes twinkle {{ 0%, 100% {{ opacity: 1; }} 50% {{ opacity: 0.3; }} }}
             .star {{ animation: twinkle {} ease-in-out infinite; }}",
            theme.animation.duration
        ));
    }

    // SVG glow filter.
    doc.def(r#"<filter id="glow"><feGaussianBlur stdDeviation="3" result="blur"/><feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge></filter>"#);

    // Layout: position repos using a simple circular distribution.
    let positions = layout_positions(&ctx.repos, theme.width as f64, theme.height as f64);

    // Draw connections between repos that share languages.
    let connections = find_connections(&ctx.repos);
    for (i, j) in &connections {
        let (x1, y1) = positions[*i];
        let (x2, y2) = positions[*j];
        doc.add(
            doc.line(x1, y1, x2, y2)
                .stroke(&theme.foreground)
                .stroke_width(theme.connection.width)
                .opacity(theme.connection.opacity),
        );
    }

    // Draw stars (repos).
    for (idx, repo) in ctx.repos.iter().enumerate() {
        let (x, y) = positions[idx];
        let r = star_radius(repo, theme);

        let color = repo
            .primary_language_color
            .as_deref()
            .unwrap_or(&theme.accent);

        let mut star = doc.circle(x, y, r).fill(color).opacity(0.9);

        if theme.star.glow {
            star = star.filter("glow");
        }

        if theme.animation.twinkle {
            let delay = (idx as f64 * 0.3) % 4.0;
            star = star.attr("style", &format!("animation-delay: {delay}s"));
            star = star.class("star");
        }

        doc.add(star);
    }

    // Title text.
    doc.add(
        doc.text(theme.width as f64 / 2.0, 30.0, &ctx.user_name)
            .fill(&theme.foreground)
            .font_size(16.0)
            .font_family(&theme.font)
            .text_anchor("middle")
            .attr("font-weight", "bold"),
    );

    // Stats summary.
    let summary = format!(
        "{} repos | {} stars | {} contributions",
        ctx.total_repos, ctx.total_stars, ctx.total_contributions
    );
    doc.add(
        doc.text(
            theme.width as f64 / 2.0,
            theme.height as f64 - 15.0,
            &summary,
        )
        .fill(&theme.foreground)
        .font_size(10.0)
        .font_family(&theme.font)
        .text_anchor("middle")
        .opacity(0.6),
    );

    // Top languages legend with percentages, wrapping to a second row if
    // needed. Percentages share the global language total so they sum to 100%
    // across the visible repo set.
    let legend_left = 20.0;
    let legend_right = theme.width as f64 - 20.0;
    let row_h = 14.0;
    let mut legend_x = legend_left;
    let mut legend_y = theme.height as f64 - 40.0;
    for lang in ctx.top_languages.iter().take(12) {
        let label = format!("{} {:.1}%", lang.name, lang.percentage);
        let w = label.len() as f64 * 6.5 + 20.0;
        if legend_x + w > legend_right {
            legend_x = legend_left;
            legend_y += row_h;
        }
        doc.add(doc.circle(legend_x, legend_y, 4.0).fill(&lang.color));
        doc.add(
            doc.text(legend_x + 8.0, legend_y + 4.0, &label)
                .fill(&theme.foreground)
                .font_size(9.0)
                .font_family(&theme.font)
                .opacity(0.7),
        );
        legend_x += w;
    }

    Ok(doc.to_string())
}

/// Compute repo star positions using a circular packing layout.
fn layout_positions(repos: &[RepoContext], width: f64, height: f64) -> Vec<(f64, f64)> {
    let cx = width / 2.0;
    let cy = height / 2.0;
    let max_r = width.min(height) / 2.0 - 40.0;

    let n = repos.len();
    if n == 0 {
        return Vec::new();
    }

    repos
        .iter()
        .enumerate()
        .map(|(i, _)| {
            // Golden angle spiral for even distribution.
            let angle = i as f64 * 2.399963; // golden angle in radians
            let r = max_r * (i as f64 / n as f64).sqrt();
            let x = cx + r * angle.cos();
            let y = cy + r * angle.sin();
            (x, y)
        })
        .collect()
}

/// Compute star radius from repo stats.
fn star_radius(repo: &RepoContext, theme: &Theme) -> f64 {
    let min = theme.star.min_radius;
    let max = theme.star.max_radius;

    // Scale by commits, clamped.
    let scale = if repo.commits > 0 {
        (repo.commits as f64).ln() / 10.0
    } else {
        0.1
    };

    (min + (max - min) * scale.clamp(0.0, 1.0)).clamp(min, max)
}

/// Find pairs of repos that share at least one language.
fn find_connections(repos: &[RepoContext]) -> Vec<(usize, usize)> {
    let mut connections = Vec::new();

    for i in 0..repos.len() {
        for j in (i + 1)..repos.len() {
            let lang_a: std::collections::HashSet<_> =
                repos[i].languages.iter().map(|l| &l.name).collect();
            let shared = repos[j].languages.iter().any(|l| lang_a.contains(&l.name));

            if shared {
                connections.push((i, j));
            }
        }
    }

    // Limit connections to avoid visual clutter.
    connections.truncate(repos.len() * 2);
    connections
}
