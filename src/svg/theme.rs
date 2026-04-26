use serde::{Deserialize, Serialize};

/// Complete theme definition loaded from TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub font: String,
    pub width: u32,
    pub height: u32,

    #[serde(default)]
    pub star: StarConfig,

    #[serde(default)]
    pub connection: ConnectionConfig,

    #[serde(default)]
    pub animation: AnimationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarConfig {
    #[serde(default = "default_min_radius")]
    pub min_radius: f64,
    #[serde(default = "default_max_radius")]
    pub max_radius: f64,
    #[serde(default = "default_true")]
    pub glow: bool,
}

impl Default for StarConfig {
    fn default() -> Self {
        Self {
            min_radius: default_min_radius(),
            max_radius: default_max_radius(),
            glow: default_true(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default = "default_width")]
    pub width: f64,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            opacity: default_opacity(),
            width: default_width(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    #[serde(default = "default_true")]
    pub twinkle: bool,
    #[serde(default = "default_duration")]
    pub duration: String,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            twinkle: default_true(),
            duration: default_duration(),
        }
    }
}

fn default_min_radius() -> f64 {
    2.0
}
fn default_max_radius() -> f64 {
    12.0
}
fn default_opacity() -> f64 {
    0.15
}
fn default_width() -> f64 {
    0.5
}
fn default_duration() -> String {
    "4s".to_string()
}
fn default_true() -> bool {
    true
}

/// Load a theme from a TOML string.
pub fn from_toml(input: &str) -> anyhow::Result<Theme> {
    let theme: Theme = toml::from_str(input)?;
    Ok(theme)
}

/// Load a theme from a file path.
pub fn from_file(path: &str) -> anyhow::Result<Theme> {
    let content = std::fs::read_to_string(path)?;
    from_toml(&content)
}

/// Get a built-in theme by name. Returns the dark variant by default.
pub fn builtin(name: &str) -> anyhow::Result<Theme> {
    match name {
        "nebula" | "nebula-dark" => from_toml(include_str!("../../themes/nebula-dark.toml")),
        "nebula-light" => from_toml(include_str!("../../themes/nebula-light.toml")),
        "terminal" | "terminal-dark" => from_toml(include_str!("../../themes/terminal-dark.toml")),
        "radar" | "radar-dark" => from_toml(include_str!("../../themes/radar-dark.toml")),
        "heatmap" | "heatmap-dark" => from_toml(include_str!("../../themes/heatmap-dark.toml")),
        "fingerprint" | "fingerprint-dark" => {
            from_toml(include_str!("../../themes/fingerprint-dark.toml"))
        }
        _ => anyhow::bail!("unknown theme: {name}"),
    }
}

/// Load a theme by name, checking custom dir first, then built-ins.
pub fn load(name: &str, theme_dir: Option<&str>) -> anyhow::Result<Theme> {
    if let Some(dir) = theme_dir {
        let path = std::path::Path::new(dir).join(format!("{name}.toml"));
        if path.exists() {
            tracing::info!("loading custom theme from {}", path.display());
            return from_file(path.to_str().unwrap_or_default());
        }
    }
    builtin(name)
}
