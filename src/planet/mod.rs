/// Describes the basic properties of a planet.
#[derive(Clone)]
pub struct Description {
    pub radius: f64,
    pub terrain: TerrainLayer
}

mod constants;
mod face;
mod generator;
mod geometry_provider;
mod quad_tree;
mod renderer;
mod terrain;
mod async_geometry_provider;

pub use self::face::Face;
pub use self::generator::Generator;
pub use self::geometry_provider::{GeometryProvider, PatchGeometry, PatchLocation};
pub use self::renderer::{DrawParameters, Renderer};
pub use self::terrain::{NoiseFunction, TerrainCombinator, TerrainFunction, TerrainLayer};
