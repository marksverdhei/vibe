use serde::Deserialize;
use vibe_renderer::components::GpuColors;

/// Deserialized from `colors.toml`. Each color is [R, G, B] with values 0.0-1.0.
#[derive(Debug, Clone, Deserialize)]
pub struct ColorsConfig {
    #[serde(default = "default_color1")]
    pub color1: [f32; 3],
    #[serde(default = "default_color2")]
    pub color2: [f32; 3],
    #[serde(default = "default_color3")]
    pub color3: [f32; 3],
    #[serde(default = "default_color4")]
    pub color4: [f32; 3],
}

fn default_color1() -> [f32; 3] { [0.0, 1.0, 0.5] }
fn default_color2() -> [f32; 3] { [1.0, 0.0, 1.0] }
fn default_color3() -> [f32; 3] { [0.0, 0.5, 1.0] }
fn default_color4() -> [f32; 3] { [1.0, 1.0, 0.0] }

impl Default for ColorsConfig {
    fn default() -> Self {
        Self {
            color1: default_color1(),
            color2: default_color2(),
            color3: default_color3(),
            color4: default_color4(),
        }
    }
}

impl ColorsConfig {
    pub fn to_gpu(&self) -> GpuColors {
        GpuColors {
            color1: [self.color1[0], self.color1[1], self.color1[2], 1.0],
            color2: [self.color2[0], self.color2[1], self.color2[2], 1.0],
            color3: [self.color3[0], self.color3[1], self.color3[2], 1.0],
            color4: [self.color4[0], self.color4[1], self.color4[2], 1.0],
        }
    }
}

/// Load colors from colors.toml, falling back to defaults if not found.
pub fn load() -> ColorsConfig {
    let path = crate::get_colors_path();
    match std::fs::read_to_string(&path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(colors) => colors,
            Err(err) => {
                tracing::warn!("Failed to parse {}: {}. Using defaults.", path.display(), err);
                ColorsConfig::default()
            }
        },
        Err(_) => {
            tracing::info!("No colors.toml found, using defaults.");
            ColorsConfig::default()
        }
    }
}
