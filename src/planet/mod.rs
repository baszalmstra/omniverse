use crate::transform::{Transform, Transformable};

/// Describes the basic properties of a planet.
#[derive(Clone)]
pub struct Description {
    pub radius: f64,
}

mod renderer;
mod quad_tree;
mod face;

pub use self::renderer::Renderer;
pub use self::face::Face;