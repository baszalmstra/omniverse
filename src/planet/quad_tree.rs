use std::boxed::Box;
use std::option::Option;

pub struct QuadTree<T> {
    pub content: T,
    pub children: [Option<Box<QuadTree<T>>>; 4],
}

impl<T> QuadTree<T> {
    pub fn new(content: T) -> QuadTree<T> {
        QuadTree {
            children: [None, None, None, None],
            content,
        }
    }
}
