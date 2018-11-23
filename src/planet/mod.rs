use crate::transform::{Transform, Transformable};

/// Describes the basic properties of a planet.
#[derive(Clone)]
pub struct Description {
    pub radius: f64,
}

mod renderer;
mod quad_tree;

pub use self::renderer::Renderer;