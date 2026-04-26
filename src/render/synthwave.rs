use anyhow::Result;

use crate::render::context::RenderContext;
use crate::svg::Svg;
use crate::svg::theme::Theme;

/// Render the synthwave / retrowave visualization.
pub fn render(ctx: &RenderContext, theme: &Theme) -> Result<String> {
    let mut doc = Svg::new(theme.width, theme.height);
    let w = theme.width as f64;
    let h = theme.height as f64;
    let horizon = h * 0.55;

    doc.def(
        r##"<linearGradient id="sw-sky" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#0a0118"/>
            <stop offset="55%" stop-color="#28114a"/>
            <stop offset="100%" stop-color="#5b1e6b"/>
        </linearGradient>"##,
    );
    doc.def(
        r##"<linearGradient id="sw-ground" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#1a0428"/>
            <stop offset="100%" stop-color="#000007"/>
        </linearGradient>"##,
    );
    doc.def(
        r##"<linearGradient id="sw-sun" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#fef08a"/>
            <stop offset="50%" stop-color="#ff6b3d"/>
            <stop offset="100%" stop-color="#ff2bd6"/>
        </linearGradient>"##,
    );
    doc.def(
        r##"<filter id="sw-neon" x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="3" result="blur"/>
            <feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge>
        </filter>"##,
    );

    // Sky and ground.
    doc.add(doc.rect(0.0, 0.0, w, horizon).fill("url(#sw-sky)"));
    doc.add(doc.rect(0.0, horizon, w, h - horizon).fill("url(#sw-ground)"));

    // Retrowave sun: filled circle clipped to its own bounds, with horizontal
    // stripes painted on the lower half in the sky color to slice it up.
    let sun_cx = w / 2.0;
    let sun_r = (h * 0.32).min(140.0);
    let sun_cy = horizon - sun_r * 0.55;
    let sun_clip = format!(
        r##"<clipPath id="sw-sun-clip"><circle cx="{sun_cx:.1}" cy="{sun_cy:.1}" r="{sun_r:.1}"/></clipPath>"##
    );
    doc.def(&sun_clip);
    doc.add(
        doc.circle(sun_cx, sun_cy, sun_r)
            .fill("url(#sw-sun)")
            .filter("sw-neon"),
    );
    let stripe_color = "#0a0118";
    for i in 0..7 {
        let offset = sun_r * 0.25 + i as f64 * (sun_r * 0.12);
        if offset >= sun_r {
            break;
        }
        let band_y = sun_cy + offset;
        let band_h = (sun_r * 0.12 - 6.0).max(2.0);
        doc.add(
            doc.rect(sun_cx - sun_r, band_y, sun_r * 2.0, band_h)
                .fill(stripe_color)
                .attr("clip-path", "url(#sw-sun-clip)"),
        );
    }

    // Perspective grid below the horizon. Vanishing point sits at the sun.
    let vp_x = sun_cx;
    let vp_y = horizon;
    let grid_color = "#ff2bd6";
    let grid_opacity = 0.55;

    // Horizontal grid lines: spacing tightens as we approach the horizon.
    let row_count = 9;
    for i in 1..=row_count {
        let t = (i as f64 / row_count as f64).powf(2.2);
        let y = horizon + t * (h - horizon);
        doc.add(
            doc.line(0.0, y, w, y)
                .stroke(grid_color)
                .stroke_width(0.7)
                .opacity(grid_opacity * (0.4 + 0.6 * t)),
        );
    }

    // Vertical grid lines: radiate from the vanishing point to the bottom.
    let col_count = 13;
    let span = w * 1.5;
    for i in 0..=col_count {
        let frac = i as f64 / col_count as f64;
        let bottom_x = -span / 2.0 + w / 2.0 + frac * span;
        doc.add(
            doc.line(vp_x, vp_y, bottom_x, h)
                .stroke(grid_color)
                .stroke_width(0.7)
                .opacity(grid_opacity),
        );
    }

    // Title.
    doc.add(
        doc.text(w / 2.0, 32.0, &ctx.user_name.to_uppercase())
            .fill("#fef08a")
            .font_size(20.0)
            .font_family(&theme.font)
            .text_anchor("middle")
            .attr("font-weight", "bold")
            .attr("filter", "url(#sw-neon)")
            .attr("letter-spacing", "4"),
    );

    // Stats line, just under the title.
    doc.add(
        doc.text(
            w / 2.0,
            52.0,
            &format!(
                "{} repos // {} stars // {} contributions",
                ctx.total_repos, ctx.total_stars, ctx.total_contributions
            ),
        )
        .fill("#a8d8ff")
        .font_size(10.0)
        .font_family(&theme.font)
        .text_anchor("middle")
        .opacity(0.85),
    );

    // Language equalizer: neon pillars rising from the horizon.
    let bar_count = ctx.top_languages.len().min(8);
    if bar_count > 0 {
        let pad = 60.0;
        let zone_w = w - pad * 2.0;
        let bar_w = (zone_w / bar_count as f64) * 0.55;
        let gap = (zone_w / bar_count as f64) * 0.45;
        let max_pct = ctx
            .top_languages
            .first()
            .map(|l| l.percentage)
            .unwrap_or(1.0)
            .max(1.0);
        let max_bar_h = horizon * 0.38;

        for (i, lang) in ctx.top_languages.iter().take(bar_count).enumerate() {
            let frac = (lang.percentage / max_pct).clamp(0.05, 1.0);
            let bar_h = max_bar_h * frac;
            let x = pad + i as f64 * (bar_w + gap) + gap / 2.0;
            let y = horizon - bar_h;
            doc.add(
                doc.rect(x, y, bar_w, bar_h)
                    .fill(&lang.color)
                    .opacity(0.92)
                    .filter("sw-neon")
                    .rx(2.0),
            );
            doc.add(
                doc.text(x + bar_w / 2.0, y - 8.0, &lang.name)
                    .fill("#f0a8e8")
                    .font_size(9.0)
                    .font_family(&theme.font)
                    .text_anchor("middle")
                    .opacity(0.85),
            );
            doc.add(
                doc.text(
                    x + bar_w / 2.0,
                    horizon + 14.0,
                    &format!("{:.1}%", lang.percentage),
                )
                .fill("#a8d8ff")
                .font_size(8.0)
                .font_family(&theme.font)
                .text_anchor("middle")
                .opacity(0.75),
            );
        }
    }

    Ok(doc.to_string())
}
