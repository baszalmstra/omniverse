#![allow(dead_code)]

use crate::transform::Rotation;
use std::f64::consts::PI;

#[derive(Copy, Clone, Debug)]
pub enum Face {
    Left,
    Right,
    Top,
    Bottom,
    Front,
    Back,
}

lazy_static! {
    static ref ORIENTATION_LEFT: Rotation = Rotation::from_euler_angles(0.0, 0.5 * PI, 0.0);
    static ref ORIENTATION_RIGHT: Rotation = Rotation::from_euler_angles(0.0, -0.5 * PI, 0.0);
    static ref ORIENTATION_TOP: Rotation = Rotation::from_euler_angles(PI * 0.5, 0.0, 0.0);
    static ref ORIENTATION_BOTTOM: Rotation = Rotation::from_euler_angles(-PI * 0.5, 0.0, 0.0);
    static ref ORIENTATION_FRONT: Rotation = Rotation::from_euler_angles(0.0, 0.0, 0.0);
    static ref ORIENTATION_BACK: Rotation = Rotation::from_euler_angles(0.0, PI, 0.0);
}

impl Face {
    pub fn orientation(self) -> Rotation {
        match self {
            Face::Left => *ORIENTATION_LEFT,
            Face::Right => *ORIENTATION_RIGHT,
            Face::Top => *ORIENTATION_TOP,
            Face::Bottom => *ORIENTATION_BOTTOM,
            Face::Front => *ORIENTATION_FRONT,
            Face::Back => *ORIENTATION_BACK,
        }
    }

    pub fn values() -> impl Iterator<Item = &'static Face> {
        static VALUES: [Face;  6] = [Face::Left, Face::Right, Face::Top, Face::Bottom, Face::Front, Face::Back];
        VALUES.iter()
    }
}
