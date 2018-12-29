use ncollide::bounding_volume::AABB;
use std::boxed::Box;
use std::iter::Iterator;
use std::option::Option;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Child {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Child {
    pub fn index(self) -> usize {
        match self {
            Child::TopLeft => 0,
            Child::TopRight => 1,
            Child::BottomLeft => 2,
            Child::BottomRight => 3,
        }
    }

    pub fn values() -> impl Iterator<Item = &'static Child> {
        static DIRECTIONS: [Child; 4] = [
            Child::TopLeft,
            Child::TopRight,
            Child::BottomLeft,
            Child::BottomRight,
        ];
        DIRECTIONS.iter()
    }
}

pub struct QuadTree<T> {
    pub content: T,
    pub children: Option<Box<[QuadTree<T>; 4]>>,
}

impl<T> QuadTree<T> {
    pub fn new(content: T) -> QuadTree<T> {
        QuadTree {
            children: None,
            content,
        }
    }

    pub fn has_children(&self) -> bool {
        self.children.is_some()
    }
}

pub trait HasAABB<N> {
    fn bounding_box(&self) -> AABB<N>;
}

impl<N, T: HasAABB<N>> HasAABB<N> for QuadTree<T> {
    fn bounding_box(&self) -> AABB<N> {
        self.content.bounding_box()
    }
}