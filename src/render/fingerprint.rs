use anyhow::Result;

use crate::render::context::RenderContext;
use crate::svg::Svg;
use crate::svg::theme::Theme;

/// Render the audio waveform / DNA strand fingerprint visualization.
pub fn render(ctx: &RenderContext, theme: &Theme) -> Result<String> {
    let mut doc = Svg::new(theme.width, theme.height);
    let w = theme.width as f64;
    let h = theme.height as f64;
    let cy = h / 2.0;

    // Background.
    doc.add(doc.rect(0.0, 0.0, w, h).fill(&theme.background).rx(12.0));

    let margin_x = 40.0;
    let margin_y = 40.0;
    let usable_w = w - margin_x * 2.0;
    let usable_h = h - margin_y * 2.0;
    let max_bar_h = usable_h / 2.0;

    // Generate bars from various stats dimensions.
    let dimensions = build_dimensions(ctx);
    let bar_count = dimensions.len();
    if bar_count == 0 {
        doc.add(
            doc.text(w / 2.0, cy, "No data for fingerprint")
                .fill(&theme.foreground)
                .font_size(14.0)
                .font_family(&theme.font)
                .text_anchor("middle"),
        );
        return Ok(doc.to_string());
    }

    let bar_w = (usable_w / bar_count as f64) * 0.6;
    let gap = (usable_w / bar_count as f64) * 0.4;

    // Center line.
    doc.add(
        doc.line(margin_x, cy, w - margin_x, cy)
            .stroke(&theme.foreground)
            .stroke_width(0.3)
            .opacity(0.2),
    );

    // Draw bars.
    for (i, dim) in dimensions.iter().enumerate() {
        let x = margin_x + i as f64 * (bar_w + gap) + gap / 2.0;
        let bar_h = max_bar_h * dim.value;
        let color = &dim.color;

        // Bar going up.
        doc.add(
            doc.rect(x, cy - bar_h, bar_w, bar_h)
                .fill(color)
                .opacity(0.8)
                .rx(1.0),
        );

        // Bar going down (mirror, slightly dimmer).
        doc.add(
            doc.rect(x, cy, bar_w, bar_h * 0.6)
                .fill(color)
                .opacity(0.4)
                .rx(1.0),
        );
    }

    // Title.
    doc.add(
        doc.text(
            w / 2.0,
            20.0,
            &format!("{} // developer fingerprint", ctx.user_name),
        )
        .fill(&theme.foreground)
        .font_size(12.0)
        .font_family(&theme.font)
        .text_anchor("middle")
        .attr("font-weight", "bold"),
    );

    // Stats at bottom.
    doc.add(
        doc.text(
            w / 2.0,
            h - 15.0,
            &format!(
                "{} repos | {} stars | {} contributions | {} languages",
                ctx.total_repos,
                ctx.total_stars,
                ctx.total_contributions,
                ctx.top_languages.len()
            ),
        )
        .fill(&theme.foreground)
        .font_size(9.0)
        .font_family(&theme.font)
        .text_anchor("middle")
        .opacity(0.5),
    );

    Ok(doc.to_string())
}

struct Dimension {
    color: String,
    value: f64,
}

fn build_dimensions(ctx: &RenderContext) -> Vec<Dimension> {
    let mut dims = Vec::new();

    // Normalize each stat to 0..1.
    let max_commits = 1000.0;
    let max_stars = 500.0;
    let max_repos = 50.0;

    dims.push(Dimension {
        color: ctx
            .top_languages
            .first()
            .map(|l| l.color.clone())
            .unwrap_or_else(theme_accent),
        value: (ctx.total_commits as f64 / max_commits).min(1.0),
    });

    dims.push(Dimension {
        color: ctx
            .top_languages
            .get(1)
            .map(|l| l.color.clone())
            .unwrap_or_else(|| "#8b949e".to_string()),
        value: (ctx.total_stars as f64 / max_stars).min(1.0),
    });

    dims.push(Dimension {
        color: ctx
            .top_languages
            .get(2)
            .map(|l| l.color.clone())
            .unwrap_or_else(|| "#8b949e".to_string()),
        value: (ctx.total_repos as f64 / max_repos).min(1.0),
    });

    // One bar per top language.
    for lang in ctx.top_languages.iter().take(12) {
        dims.push(Dimension {
            color: lang.color.clone(),
            value: (lang.percentage / 100.0).min(1.0),
        });
    }

    // Contribution years.
    for year in ctx.contribution_years.iter().take(3) {
        let intensity = (year.total_count as f64 / 1000.0).min(1.0);
        dims.push(Dimension {
            color: theme_accent(),
            value: intensity,
        });
    }

    dims
}

fn theme_accent() -> String {
    "#bc8cff".to_string()
}
