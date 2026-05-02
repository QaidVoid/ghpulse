use anyhow::Result;

use crate::render::context::RenderContext;
use crate::svg::Svg;
use crate::svg::theme::Theme;

struct Palette {
    sky_top: &'static str,
    sky_mid: &'static str,
    sky_bottom: &'static str,
    ground_top: &'static str,
    ground_bottom: &'static str,
    sun_top: &'static str,
    sun_mid: &'static str,
    sun_bottom: &'static str,
    stripe: &'static str,
    grid: &'static str,
    title: &'static str,
    stats: &'static str,
    label: &'static str,
    pct: &'static str,
}

const DARK: Palette = Palette {
    sky_top: "#0a0118",
    sky_mid: "#28114a",
    sky_bottom: "#5b1e6b",
    ground_top: "#1a0428",
    ground_bottom: "#000007",
    sun_top: "#fef08a",
    sun_mid: "#ff6b3d",
    sun_bottom: "#ff2bd6",
    stripe: "#0a0118",
    grid: "#ff2bd6",
    title: "#fef08a",
    stats: "#a8d8ff",
    label: "#f0a8e8",
    pct: "#a8d8ff",
};

const LIGHT: Palette = Palette {
    sky_top: "#fff5e6",
    sky_mid: "#ffd6e7",
    sky_bottom: "#ffb3d1",
    ground_top: "#f0d4e8",
    ground_bottom: "#e6c2d9",
    sun_top: "#ffea7a",
    sun_mid: "#ff7a3d",
    sun_bottom: "#d63384",
    stripe: "#ffd6e7",
    grid: "#d63384",
    title: "#a04060",
    stats: "#5b1e6b",
    label: "#5b1e6b",
    pct: "#7a3a8a",
};

/// Render the synthwave / retrowave visualization.
pub fn render(ctx: &RenderContext, theme: &Theme) -> Result<String> {
    let mut doc = Svg::new(theme.width, theme.height);
    let w = theme.width as f64;
    let h = theme.height as f64;
    let horizon = h * 0.62;
    let p = if theme.is_light { &LIGHT } else { &DARK };

    doc.def(&format!(
        r##"<linearGradient id="sw-sky" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="{}"/>
            <stop offset="55%" stop-color="{}"/>
            <stop offset="100%" stop-color="{}"/>
        </linearGradient>"##,
        p.sky_top, p.sky_mid, p.sky_bottom
    ));
    doc.def(&format!(
        r##"<linearGradient id="sw-ground" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="{}"/>
            <stop offset="100%" stop-color="{}"/>
        </linearGradient>"##,
        p.ground_top, p.ground_bottom
    ));
    doc.def(&format!(
        r##"<linearGradient id="sw-sun" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="{}"/>
            <stop offset="50%" stop-color="{}"/>
            <stop offset="100%" stop-color="{}"/>
        </linearGradient>"##,
        p.sun_top, p.sun_mid, p.sun_bottom
    ));
    doc.def(
        r##"<filter id="sw-neon" x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="2.5" result="blur"/>
            <feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge>
        </filter>"##,
    );

    // Sky.
    doc.add(doc.rect(0.0, 0.0, w, horizon).fill("url(#sw-sky)"));

    // Retrowave sun: bottom edge sits exactly on the horizon so all stripes
    // stay above it. The sun is rendered before the ground rect, so the
    // ground rect provides a clean horizon edge over its bottom curve.
    let sun_cx = w / 2.0;
    let sun_r = (h * 0.18).clamp(48.0, 70.0);
    let sun_cy = horizon - sun_r;

    let sun_clip = format!(
        r##"<clipPath id="sw-sun-clip"><circle cx="{sun_cx:.1}" cy="{sun_cy:.1}" r="{sun_r:.1}"/></clipPath>"##
    );
    doc.def(&sun_clip);

    doc.add(
        doc.circle(sun_cx, sun_cy, sun_r)
            .fill("url(#sw-sun)")
            .filter("sw-neon"),
    );

    // Stripes on the lower half of the sun (between sun_cy and the horizon).
    // Painting them with the sky color matches the gradient at that y so they
    // look cut into the sun rather than overlaid.
    let stripe_count = 5;
    for i in 0..stripe_count {
        let frac = 0.30 + i as f64 * 0.16;
        let band_y = sun_cy + sun_r * frac;
        let band_h = (sun_r * 0.07).max(2.5);
        doc.add(
            doc.rect(sun_cx - sun_r, band_y, sun_r * 2.0, band_h)
                .fill(p.stripe)
                .opacity(0.95)
                .attr("clip-path", "url(#sw-sun-clip)"),
        );
    }

    // Ground covers everything below the horizon (including any sub-pixel
    // bleed from the sun) for a crisp horizon line.
    doc.add(
        doc.rect(0.0, horizon, w, h - horizon)
            .fill("url(#sw-ground)"),
    );

    // Perspective grid below the horizon. Vanishing point at the sun base.
    let vp_x = sun_cx;
    let vp_y = horizon;
    let grid_color = p.grid;
    let grid_opacity = 0.55;

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

    // Title sits comfortably above the sun.
    doc.add(
        doc.text(w / 2.0, 36.0, &ctx.user_name.to_uppercase())
            .fill(p.title)
            .font_size(18.0)
            .font_family(&theme.font)
            .text_anchor("middle")
            .attr("font-weight", "bold")
            .attr("filter", "url(#sw-neon)")
            .attr("letter-spacing", "3"),
    );
    doc.add(
        doc.text(
            w / 2.0,
            56.0,
            &format!(
                "{} repos // {} stars // {} contributions",
                ctx.total_repos, ctx.total_stars, ctx.total_contributions
            ),
        )
        .fill(p.stats)
        .font_size(10.0)
        .font_family(&theme.font)
        .text_anchor("middle")
        .opacity(0.85),
    );

    // Language pillars rising from the horizon. Skips bars that would land
    // behind the sun, and uses sqrt scaling so the dominant language doesn't
    // crush all the others.
    let bars: Vec<_> = ctx.top_languages.iter().take(8).collect();
    if !bars.is_empty() {
        let pad = 50.0;
        let zone_w = w - pad * 2.0;
        let slot_w = zone_w / bars.len() as f64;
        let bar_w = (slot_w * 0.55).min(40.0);
        let max_bar_h = (sun_cy - 70.0).max(40.0);
        let max_pct = bars.first().map(|l| l.percentage).unwrap_or(1.0).max(1.0);
        let sun_left = sun_cx - sun_r - 4.0;
        let sun_right = sun_cx + sun_r + 4.0;

        for (i, lang) in bars.iter().enumerate() {
            let cx = pad + (i as f64 + 0.5) * slot_w;
            if cx > sun_left && cx < sun_right {
                continue;
            }
            let frac = (lang.percentage / max_pct).clamp(0.04, 1.0).sqrt();
            let bar_h = (max_bar_h * frac).max(10.0);
            let x = cx - bar_w / 2.0;
            let y = horizon - bar_h;
            doc.add(
                doc.rect(x, y, bar_w, bar_h)
                    .fill(&lang.color)
                    .opacity(0.92)
                    .filter("sw-neon")
                    .rx(2.0),
            );
            doc.add(
                doc.text(cx, y - 6.0, &lang.name)
                    .fill(p.label)
                    .font_size(9.0)
                    .font_family(&theme.font)
                    .text_anchor("middle")
                    .opacity(0.85),
            );
            doc.add(
                doc.text(cx, horizon + 14.0, &format!("{:.1}%", lang.percentage))
                    .fill(p.pct)
                    .font_size(8.0)
                    .font_family(&theme.font)
                    .text_anchor("middle")
                    .opacity(0.75),
            );
        }
    }

    Ok(doc.to_string())
}
