pub mod context;
pub mod fingerprint;
pub mod heatmap;
pub mod nebula;
pub mod radar;
pub mod terminal;

use anyhow::Result;

use crate::svg::theme::Theme;
use context::RenderContext;

/// Render the stats into an SVG string using the given theme.
pub fn render(ctx: &RenderContext, theme: &Theme) -> Result<String> {
    let name = theme.name.to_lowercase();

    if name.contains("terminal") {
        terminal::render(ctx, theme)
    } else if name.contains("radar") {
        radar::render(ctx, theme)
    } else if name.contains("heatmap") {
        heatmap::render(ctx, theme)
    } else if name.contains("fingerprint") {
        fingerprint::render(ctx, theme)
    } else {
        nebula::render(ctx, theme)
    }
}
