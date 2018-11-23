use transform::Transform;
use planet::quad_tree::QuadTree;
use nalgebra::UnitQuaternion;
use super::node::Node;
use transform::Rotation;
use crate::planet;

/// Contains geometry for a single face of the cube that is the planet.
pub struct Face {
    pub face: planet::Face,
    pub root: QuadTree<Node>
}

impl Face {
    pub fn new(node: Node, face: planet::Face) -> Face {
        Face {
            face,
            root: QuadTree::new(node)
        }
    }

    pub fn orientation(&self) -> Rotation {
        self.face.orientation()
    }
}