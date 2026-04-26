use anyhow::Result;

use crate::render::context::RenderContext;
use crate::svg::Svg;
use crate::svg::theme::Theme;

/// Render the retro terminal visualization.
pub fn render(ctx: &RenderContext, theme: &Theme) -> Result<String> {
    let mut doc = Svg::new(theme.width, theme.height);
    let w = theme.width as f64;
    let h = theme.height as f64;

    // Background.
    doc.add(doc.rect(0.0, 0.0, w, h).fill(&theme.background).rx(8.0));

    // CRT scan-line overlay.
    doc.style(".scanline { background: repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(0,0,0,0.15) 2px, rgba(0,0,0,0.15) 4px); }");

    // Border.
    doc.add(
        doc.rect(2.0, 2.0, w - 4.0, h - 4.0)
            .fill("none")
            .stroke(&theme.foreground)
            .stroke_width(1.0)
            .opacity(0.3),
    );

    let mut y = 30.0;
    let line_h = 18.0;
    let x = 20.0;

    // Header.
    doc.add(
        doc.text(x, y, "> ghpulse --user")
            .fill(&theme.foreground)
            .font_size(12.0)
            .font_family(&theme.font),
    );
    y += line_h * 1.5;

    // User info.
    doc.add(
        doc.text(x, y, &format!("LOGIN:    {}", ctx.user_login))
            .fill(&theme.foreground)
            .font_size(11.0)
            .font_family(&theme.font),
    );
    y += line_h;

    doc.add(
        doc.text(x, y, &format!("REPOS:    {}", ctx.total_repos))
            .fill(&theme.foreground)
            .font_size(11.0)
            .font_family(&theme.font),
    );
    y += line_h;

    doc.add(
        doc.text(x, y, &format!("STARS:    {}", ctx.total_stars))
            .fill(&theme.accent)
            .font_size(11.0)
            .font_family(&theme.font),
    );
    y += line_h;

    doc.add(
        doc.text(x, y, &format!("CONTRIBS: {}", ctx.total_contributions))
            .fill(&theme.foreground)
            .font_size(11.0)
            .font_family(&theme.font),
    );
    y += line_h;

    // Separator.
    doc.add(
        doc.line(x, y, w - x, y)
            .stroke(&theme.foreground)
            .stroke_width(0.5)
            .opacity(0.3),
    );
    y += line_h;

    // Top languages.
    doc.add(
        doc.text(x, y, "> top languages")
            .fill(&theme.accent)
            .font_size(11.0)
            .font_family(&theme.font),
    );
    y += line_h;

    for lang in ctx.top_languages.iter().take(6) {
        let bar_width = (lang.percentage / 100.0 * (w - 200.0)).max(2.0);
        doc.add(
            doc.text(x, y, &format!("{:>12}", lang.name))
                .fill(&lang.color)
                .font_size(10.0)
                .font_family(&theme.font),
        );
        doc.add(
            doc.rect(x + 100.0, y - 8.0, bar_width, 10.0)
                .fill(&lang.color)
                .opacity(0.7),
        );
        doc.add(
            doc.text(
                x + 105.0 + bar_width,
                y,
                &format!("{:.1}%", lang.percentage),
            )
            .fill(&theme.foreground)
            .font_size(9.0)
            .font_family(&theme.font)
            .opacity(0.6),
        );
        y += line_h * 0.9;
    }

    // Cursor blink at the bottom.
    doc.style("@keyframes blink { 0%, 100% { opacity: 1; } 50% { opacity: 0; } }");
    let cursor_y = h - 25.0;
    doc.add(
        doc.rect(x, cursor_y - 8.0, 8.0, 12.0)
            .fill(&theme.foreground)
            .attr("style", "animation: blink 1s step-end infinite"),
    );

    Ok(doc.to_string())
}
