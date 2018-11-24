/// Describes the basic properties of a planet.
#[derive(Copy, Clone)]
pub struct Description {
    pub radius: f64,
}

mod constants;
mod face;
mod generator;
mod geometry_provider;
mod quad_tree;
mod renderer;

pub use self::face::Face;
pub use self::generator::Generator;
pub use self::geometry_provider::{GeometryProvider, PatchGeometry, PatchLocation};
pub use self::renderer::{DrawParameters, Renderer};
