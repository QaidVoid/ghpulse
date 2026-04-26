use anyhow::Result;
use std::path::Path;

/// Write an SVG string to the output directory.
pub fn write_svg(output_dir: &str, theme_name: &str, svg: &str) -> Result<()> {
    let filename = format!("{theme_name}.svg");
    let path = Path::new(output_dir).join(&filename);

    std::fs::write(&path, svg)?;
    tracing::info!("wrote {}", path.display());

    Ok(())
}

/// Write an SVG string as a PNG file using resvg.
#[cfg(feature = "png")]
pub fn write_png(output_dir: &str, theme_name: &str, svg: &str) -> Result<()> {
    use resvg::tiny_skia::Pixmap;
    use resvg::usvg::{Options, Tree};

    let filename = format!("{theme_name}.png");
    let path = Path::new(output_dir).join(&filename);

    let opt = Options::default();
    let tree = Tree::from_str(svg, &opt)?;
    let size = tree.size();
    let w = size.width() as u32;
    let h = size.height() as u32;

    let mut pixmap = Pixmap::new(w, h).ok_or_else(|| anyhow::anyhow!("failed to create pixmap"))?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );

    let png_data = pixmap.encode_png()?;
    std::fs::write(&path, &png_data)?;
    tracing::info!("wrote {}", path.display());

    Ok(())
}
