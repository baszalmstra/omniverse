#![feature(test)]
#![feature(custom_attribute)]
//#![feature(extern_crate_item_prelude)]


#[macro_use]
extern crate imgui;

#[macro_use] extern crate glium;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
extern crate pretty_env_logger;


extern crate alga;
extern crate nalgebra;
extern crate ncollide;

extern crate core;
extern crate test;

mod id_arena;

pub mod camera;
pub mod camera_controller;
pub mod culling;
pub mod frustum;
pub mod planet;
pub mod timeline;
pub mod transform;
pub mod ui;