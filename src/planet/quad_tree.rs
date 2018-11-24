use std::boxed::Box;
use std::option::Option;
use std::iter::Iterator;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Child {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight
}

impl Child {
    pub fn index(&self) -> usize {
        match self {
            Child::TopLeft => 0,
            Child::TopRight => 1,
            Child::BottomLeft => 2,
            Child::BottomRight => 3,
        }
    }

    pub fn values() -> impl Iterator<Item = &'static Child> {
        static DIRECTIONS: [Child;  4] = [Child::TopLeft, Child::TopRight, Child::BottomLeft, Child::BottomRight];
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
