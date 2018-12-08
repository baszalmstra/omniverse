#![allow(dead_code)]

use crate::transform::Rotation;
use std::f64::consts::PI;

#[derive(Copy, Clone, Debug)]
pub enum Face {
    Left = 0,
    Right = 1,
    Top = 2,
    Bottom = 3,
    Front = 4,
    Back = 5,
}

lazy_static! {
    static ref ORIENTATIONS: [Rotation; 6] = [
        Rotation::from_euler_angles(0.0, 0.5 * PI, 0.0),
        Rotation::from_euler_angles(0.0, -0.5 * PI, 0.0),
        Rotation::from_euler_angles(PI * 0.5, 0.0, 0.0),
        Rotation::from_euler_angles(-PI * 0.5, 0.0, 0.0),
        Rotation::from_euler_angles(0.0, 0.0, 0.0),
        Rotation::from_euler_angles(0.0, PI, 0.0)
    ];
}

impl Face {
    #[inline]
    pub fn orientation(self) -> &'static Rotation {
        &ORIENTATIONS[self as usize]
    }

    pub fn values() -> impl Iterator<Item = &'static Face> {
        static VALUES: [Face; 6] = [
            Face::Left,
            Face::Right,
            Face::Top,
            Face::Bottom,
            Face::Front,
            Face::Back,
        ];
        VALUES.iter()
    }
}
