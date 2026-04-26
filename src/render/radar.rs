use anyhow::Result;

use crate::render::context::RenderContext;
use crate::svg::theme::Theme;
use crate::svg::{ElementBuilder, Svg};

/// Render the spider/radar chart visualization.
pub fn render(ctx: &RenderContext, theme: &Theme) -> Result<String> {
    let mut doc = Svg::new(theme.width, theme.height);
    let w = theme.width as f64;
    let h = theme.height as f64;
    let cx = w / 2.0;
    let cy = h / 2.0;
    let max_r = w.min(h) / 2.0 - 60.0;

    // Background.
    doc.add(doc.rect(0.0, 0.0, w, h).fill(&theme.background).rx(12.0));

    // Use top languages as axes.
    let axes: Vec<_> = ctx.top_languages.iter().take(8).collect();
    let n = axes.len();
    if n < 3 {
        doc.add(
            doc.text(cx, cy, "Not enough language data for radar chart")
                .fill(&theme.foreground)
                .font_size(14.0)
                .font_family(&theme.font)
                .text_anchor("middle"),
        );
        return Ok(doc.to_string());
    }

    // Draw concentric guide polygons.
    for ring in 1..=4 {
        let r = max_r * (ring as f64 / 4.0);
        let pts = polygon_points(cx, cy, r, n);
        doc.add(
            ElementBuilder::new(format!(r#"<polygon points="{pts}""#))
                .fill("none")
                .stroke(&theme.foreground)
                .stroke_width(0.3)
                .opacity(0.2),
        );
    }

    // Draw axis lines and labels.
    for (i, lang) in axes.iter().enumerate() {
        let angle = axis_angle(i, n);
        let ex = cx + max_r * angle.cos();
        let ey = cy + max_r * angle.sin();

        doc.add(
            doc.line(cx, cy, ex, ey)
                .stroke(&theme.foreground)
                .stroke_width(0.3)
                .opacity(0.3),
        );

        let label_r = max_r + 20.0;
        let lx = cx + label_r * angle.cos();
        let ly = cy + label_r * angle.sin();
        doc.add(
            doc.text(lx, ly + 4.0, &lang.name)
                .fill(&lang.color)
                .font_size(10.0)
                .font_family(&theme.font)
                .text_anchor("middle"),
        );
    }

    // Draw data polygon.
    let max_size = axes.iter().map(|l| l.size).max().unwrap_or(1);
    let data_pts: Vec<String> = axes
        .iter()
        .enumerate()
        .map(|(i, lang)| {
            let angle = axis_angle(i, n);
            let ratio = (lang.size as f64 / max_size as f64).sqrt();
            let r = max_r * ratio;
            format!("{},{:.1}", cx + r * angle.cos(), cy + r * angle.sin())
        })
        .collect();
    let data_str = data_pts.join(" ");

    doc.add(
        ElementBuilder::new(format!(r#"<polygon points="{data_str}""#))
            .fill(&theme.accent)
            .opacity(0.2),
    );
    doc.add(
        ElementBuilder::new(format!(r#"<polygon points="{data_str}""#))
            .fill("none")
            .stroke(&theme.accent)
            .stroke_width(1.5),
    );

    // Title.
    doc.add(
        doc.text(cx, 20.0, &ctx.user_name)
            .fill(&theme.foreground)
            .font_size(14.0)
            .font_family(&theme.font)
            .text_anchor("middle")
            .attr("font-weight", "bold"),
    );

    Ok(doc.to_string())
}

/// Compute the angle for axis `i` of `n` total axes.
fn axis_angle(i: usize, n: usize) -> f64 {
    (i as f64 / n as f64) * 2.0 * std::f64::consts::PI - std::f64::consts::FRAC_PI_2
}

/// Build a closed polygon points string for `n` vertices at radius `r`.
fn polygon_points(cx: f64, cy: f64, r: f64, n: usize) -> String {
    (0..n)
        .map(|i| {
            let angle = axis_angle(i, n);
            format!("{:.1},{:.1}", cx + r * angle.cos(), cy + r * angle.sin())
        })
        .collect::<Vec<_>>()
        .join(" ")
}
