use transform::Transform;
use planet::quad_tree::QuadTree;
use nalgebra::UnitQuaternion;
use super::node::Node;

/// Contains geometry for a single face of the cube that is the planet.
pub struct Face {
    pub orientation: UnitQuaternion<f64>,
    pub root: QuadTree<Node>
}

impl Face {
    pub fn new(node: Node, orientation: UnitQuaternion<f64>) -> Face {
        Face {
            orientation,
            root: QuadTree::new(node)
        }
    }
}