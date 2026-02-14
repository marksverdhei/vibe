//! Contains the implementation of each component.
pub mod live_wallpaper;

mod aurodio;
mod bars;
mod chessy;
mod circle;
mod fragment_canvas;
mod graph;
mod radial;
mod utils;

pub use aurodio::{Aurodio, AurodioDescriptor, AurodioLayerDescriptor};
pub use bars::{BarVariant, Bars, BarsDescriptor, BarsFormat, BarsPlacement};
pub use chessy::{Chessy, ChessyDescriptor};
pub use circle::{Circle, CircleDescriptor, CircleVariant};
pub use fragment_canvas::{FragmentCanvas, FragmentCanvasDescriptor};
pub use graph::{Graph, GraphDescriptor, GraphFormat, GraphPlacement, GraphVariant};
pub use radial::{Radial, RadialDescriptor, RadialFormat, RadialVariant};

use crate::{Renderable, Renderer};
use serde::{Deserialize, Serialize};
use std::{num::NonZero, path::PathBuf};
use utils::wgsl_types::*;
use vibe_audio::{fetcher::Fetcher, SampleProcessor};

// rgba values are each directly set in the fragment shader
pub type Rgba = Vec4f;
pub type Rgb = Vec3f;
pub type Pixels<N> = NonZero<N>;

/// Every component needs to implement this.
/// It provides methods to update its internal state regarding the current
/// audio and time for example.
pub trait Component: Renderable {
    /// Tells the component the time.
    fn update_time(&mut self, queue: &wgpu::Queue, new_time: f32);

    /// Tells the component which resolution is now used.
    fn update_resolution(&mut self, renderer: &Renderer, new_resolution: [u32; 2]);

    /// Tells the component the mouse position as normalized coordinates.
    ///
    /// Coordinate system (both `update_mouse_position` and `update_mouse_click`):
    ///   - `(0, 0)` = top-left corner of the surface
    ///   - `(1, 1)` = bottom-right corner of the surface
    ///   - Callers (window.rs, output/mod.rs) normalize from pixel coords before calling.
    fn update_mouse_position(&mut self, queue: &wgpu::Queue, new_pos: (f32, f32));

    fn update_colors(&mut self, _queue: &wgpu::Queue, _colors: &[[f32; 3]; 4]) {}

    /// Notify the component of a mouse click at a normalized position.
    ///
    /// `pos`: Normalized `(x, y)` in `[0, 1]` (see `update_mouse_position` for coord system).
    ///        `(-1, -1)` means "clear / no click".
    /// `time`: Elapsed seconds since the renderer started (same timebase as `update_time`).
    fn update_mouse_click(&mut self, _queue: &wgpu::Queue, _pos: (f32, f32), _time: f32) {}

    /// Called after the render pass completes with access to the rendered surface texture.
    ///
    /// This hook enables GPU pixel readback: components can copy pixels from the rendered
    /// frame back to the CPU. Used by FragmentCanvas to read shader-encoded data (e.g.,
    /// the Pokemon shader encodes a clicked species ID at pixel (0,0) for CPU readback).
    fn post_render(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _texture: &wgpu::Texture,
    ) {
    }
}

/// An extended version of `Component` which includes methods related to audio.
pub trait ComponentAudio<F: Fetcher>: Component {
    /// Tells the component to update its bar values with the given `processor`.
    fn update_audio(&mut self, queue: &wgpu::Queue, processor: &SampleProcessor<F>);
}

impl Renderable for Box<dyn Component> {
    fn render_with_renderpass(&self, pass: &mut wgpu::RenderPass) {
        self.as_ref().render_with_renderpass(pass)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ShaderCodeError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Couldn't parse shader code: {0}")]
    ParseError(#[from] wgpu::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShaderSource {
    Path(PathBuf),
    Code(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShaderLanguage {
    Wgsl,
    Glsl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderCode {
    pub language: ShaderLanguage,
    #[serde(flatten)]
    pub source: ShaderSource,
}

impl ShaderCode {
    pub fn source(&self) -> std::io::Result<String> {
        match self.source.clone() {
            ShaderSource::Code(code) => Ok(code),
            ShaderSource::Path(path) => std::fs::read_to_string(path),
        }
    }
}
