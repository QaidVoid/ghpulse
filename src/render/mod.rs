pub mod context;
pub mod nebula;

use anyhow::Result;

use crate::svg::theme::Theme;
use context::RenderContext;

/// Render the stats into an SVG string using the given theme.
pub fn render(ctx: &RenderContext, theme: &Theme) -> Result<String> {
    match theme.name.as_str() {
        n if n.starts_with("Nebula") => nebula::render(ctx, theme),
        _ => nebula::render(ctx, theme),
    }
}
