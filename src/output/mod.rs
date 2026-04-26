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
