// `camera::camera` matches the tutorial's module layout; the re-exports below keep
// call sites at `render::camera::{Camera, CameraResources}`.
#[allow(clippy::module_inception)]
pub mod camera;
pub mod camera_controller;
pub mod uniform_camera;

pub use camera::Camera;
pub use camera_controller::CameraController;
pub use uniform_camera::{CameraResources, CameraUniform};
