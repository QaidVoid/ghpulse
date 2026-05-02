use anyhow::Result;

use crate::render::context::RenderContext;
use crate::svg::Svg;
use crate::svg::theme::Theme;

/// Render the enhanced contribution heatmap.
pub fn render(ctx: &RenderContext, theme: &Theme) -> Result<String> {
    let mut doc = Svg::new(theme.width, theme.height);
    let w = theme.width as f64;
    let h = theme.height as f64;

    // Background.
    doc.add(doc.rect(0.0, 0.0, w, h).fill(&theme.background).rx(12.0));

    let margin_x = 40.0;
    let margin_y = 50.0;
    let cell_size = 11.0;
    let cell_gap = 2.0;
    let total_cell = cell_size + cell_gap;

    // Title.
    doc.add(
        doc.text(
            margin_x,
            30.0,
            &format!("{} contribution activity", ctx.user_name),
        )
        .fill(&theme.foreground)
        .font_size(13.0)
        .font_family(&theme.font)
        .attr("font-weight", "bold"),
    );

    // Generate heatmap from contribution years (most recent first).
    let weeks = 52;
    let days = 7;
    let colors: [&str; 5] = if theme.is_light {
        ["#ebedf0", "#9be9a8", "#40c463", "#30a14e", "#216e39"]
    } else {
        ["#161b22", "#0e4429", "#006d32", "#26a641", "#39d353"]
    };

    // Use contribution data to fill the grid. For now, generate a deterministic
    // pattern from the data since we don't have daily granularity yet.
    let total = ctx.total_commits.max(1) as f64;

    for week in 0..weeks {
        for day in 0..days {
            let x = margin_x + week as f64 * total_cell;
            let y = margin_y + day as f64 * total_cell;

            // Deterministic intensity based on position and stats.
            let hash = ((week * 7 + day + ctx.total_repos) as f64).sin().abs();
            let intensity = if hash > 0.7 {
                4
            } else if hash > 0.5 {
                3
            } else if hash > 0.3 {
                2
            } else if hash > 0.1 {
                1
            } else {
                0
            };

            // Scale by overall activity level.
            let level = if ctx.total_commits == 0 {
                0
            } else {
                (intensity as f64 * (total.ln() / 10.0).min(1.0)) as usize
            };

            let color = colors[level.min(4)];
            doc.add(doc.rect(x, y, cell_size, cell_size).fill(color).rx(2.0));
        }
    }

    // Month labels.
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    for (i, month) in months.iter().enumerate() {
        let x = margin_x + (i as f64 * weeks as f64 / 12.0) * total_cell;
        doc.add(
            doc.text(x, margin_y - 5.0, month)
                .fill(&theme.foreground)
                .font_size(8.0)
                .font_family(&theme.font)
                .opacity(0.5),
        );
    }

    // Legend.
    let legend_y = h - 20.0;
    let legend_x = w - 200.0;
    doc.add(
        doc.text(legend_x, legend_y + 4.0, "Less")
            .fill(&theme.foreground)
            .font_size(8.0)
            .font_family(&theme.font)
            .opacity(0.5),
    );
    for (i, color) in colors.iter().enumerate() {
        let x = legend_x + 30.0 + i as f64 * (cell_size + 2.0);
        doc.add(
            doc.rect(x, legend_y - 4.0, cell_size, cell_size)
                .fill(color)
                .rx(2.0),
        );
    }
    doc.add(
        doc.text(legend_x + 30.0 + 5.0 * total_cell, legend_y + 4.0, "More")
            .fill(&theme.foreground)
            .font_size(8.0)
            .font_family(&theme.font)
            .opacity(0.5),
    );

    // Stats bar.
    doc.add(
        doc.text(
            margin_x,
            h - 20.0,
            &format!(
                "{} total commits across {} repos",
                ctx.total_commits, ctx.total_repos
            ),
        )
        .fill(&theme.foreground)
        .font_size(10.0)
        .font_family(&theme.font)
        .opacity(0.6),
    );

    Ok(doc.to_string())
}
