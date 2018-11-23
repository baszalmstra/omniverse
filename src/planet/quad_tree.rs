use std::option::Option;
use std::boxed::Box;

pub struct QuadTree<T> {
    pub content: T,
    pub children: [Option<Box<QuadTree<T>>>; 4],
}

impl<T> QuadTree<T> {
    pub fn new(content: T) -> QuadTree<T> {
        QuadTree {
            children: [None, None, None, None],
            content
        }
    }
}