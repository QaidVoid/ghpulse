use anyhow::Result;

use crate::render::context::RenderContext;
use crate::svg::Svg;
use crate::svg::theme::Theme;

/// Render the prism / language spectrum visualization.
///
/// White light enters a triangular prism and refracts into a fan of beams,
/// one per top language. Beam thickness is proportional to language share.
pub fn render(ctx: &RenderContext, theme: &Theme) -> Result<String> {
    let mut doc = Svg::new(theme.width, theme.height);
    let w = theme.width as f64;
    let h = theme.height as f64;

    doc.def(
        r##"<radialGradient id="prism-bg" cx="50%" cy="40%" r="80%">
            <stop offset="0%" stop-color="#1a1430"/>
            <stop offset="100%" stop-color="#0c0a18"/>
        </radialGradient>"##,
    );
    doc.def(
        r##"<filter id="beam-glow" x="-30%" y="-30%" width="160%" height="160%">
            <feGaussianBlur stdDeviation="2.5" result="b"/>
            <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
        </filter>"##,
    );

    doc.add(doc.rect(0.0, 0.0, w, h).fill("url(#prism-bg)").rx(12.0));

    // Layout: prism sits left-of-center; the spectrum fans out to the right.
    let prism_cx = w * 0.30;
    let prism_cy = h / 2.0;
    let prism_size = (h * 0.45).min(170.0);
    let half = prism_size / 2.0;

    // Incoming beam: white horizontal strip from left edge into the prism.
    doc.add(
        doc.rect(0.0, prism_cy - 3.5, prism_cx - half * 0.5, 7.0)
            .fill("#ffffff")
            .opacity(0.95)
            .filter("beam-glow"),
    );

    // Prism triangle (apex pointing left).
    let triangle = format!(
        "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
        prism_cx - half * 0.6,
        prism_cy,
        prism_cx + half * 0.5,
        prism_cy - half,
        prism_cx + half * 0.5,
        prism_cy + half,
    );
    doc.def(
        r##"<linearGradient id="prism-glass" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="#ffffff" stop-opacity="0.10"/>
            <stop offset="50%" stop-color="#a8d8ff" stop-opacity="0.18"/>
            <stop offset="100%" stop-color="#ffffff" stop-opacity="0.10"/>
        </linearGradient>"##,
    );
    doc.add(
        crate::svg::ElementBuilder::new_self_closing(format!(
            r##"<polygon points="{triangle}""##
        ))
        .fill("url(#prism-glass)")
        .stroke("#ffffff")
        .stroke_width(1.2)
        .opacity(0.85),
    );

    // Outgoing beams: one per top language, fanned vertically across the
    // right half of the canvas.
    let beam_origin_x = prism_cx + half * 0.5;
    let beam_end_x = w - 80.0;
    let langs: Vec<_> = ctx.top_languages.iter().take(8).collect();
    if !langs.is_empty() {
        let total_pct: f64 = langs.iter().map(|l| l.percentage).sum::<f64>().max(1.0);
        let max_thickness = (h * 0.10).min(28.0);

        // Spread beams across the right side, top to bottom.
        let zone_top = h * 0.18;
        let zone_bottom = h - h * 0.18;
        let n = langs.len();

        for (i, lang) in langs.iter().enumerate() {
            let frac = if n == 1 {
                0.5
            } else {
                i as f64 / (n - 1) as f64
            };
            let target_y = zone_top + frac * (zone_bottom - zone_top);
            let thickness = max_thickness * (lang.percentage / total_pct).max(0.05).sqrt() * 1.8;

            // Beam as a polygon from the prism exit point to the target y.
            let pts = format!(
                "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                beam_origin_x,
                prism_cy - 1.5,
                beam_end_x,
                target_y - thickness / 2.0,
                beam_end_x,
                target_y + thickness / 2.0,
                beam_origin_x,
                prism_cy + 1.5,
            );
            doc.add(
                crate::svg::ElementBuilder::new_self_closing(format!(
                    r##"<polygon points="{pts}""##
                ))
                .fill(&lang.color)
                .opacity(0.78)
                .filter("beam-glow"),
            );

            // Label at the right edge.
            doc.add(
                doc.text(
                    beam_end_x + 8.0,
                    target_y - 1.0,
                    &format!("{} {:.1}%", lang.name, lang.percentage),
                )
                .fill(&theme.foreground)
                .font_size(10.0)
                .font_family(&theme.font)
                .opacity(0.9),
            );
        }
    }

    // Title.
    doc.add(
        doc.text(40.0, 36.0, &ctx.user_name)
            .fill(&theme.foreground)
            .font_size(16.0)
            .font_family(&theme.font)
            .attr("font-weight", "bold"),
    );
    doc.add(
        doc.text(
            40.0,
            54.0,
            &format!(
                "{} repos · {} stars · {} contributions",
                ctx.total_repos, ctx.total_stars, ctx.total_contributions
            ),
        )
        .fill(&theme.foreground)
        .font_size(10.0)
        .font_family(&theme.font)
        .opacity(0.6),
    );

    Ok(doc.to_string())
}
