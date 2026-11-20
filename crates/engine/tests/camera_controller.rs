//! Camera controller behaviour: movement is time based, not key-event based.

use engine::render::camera::{Camera, CameraController};
use winit::keyboard::KeyCode;

use cgmath::InnerSpace;

fn distance_to_target(camera: &Camera) -> f32 {
    (camera.target - camera.eye).magnitude()
}

fn pressed(controller: &mut CameraController, code: KeyCode) {
    controller.handle_key(code, true);
}

#[test]
fn forward_moves_eye_at_speed_per_second() {
    let mut camera = Camera::default();
    let before = distance_to_target(&camera);

    let mut controller = CameraController::new(2.0);
    pressed(&mut controller, KeyCode::KeyW);
    controller.update_camera(&mut camera, 1.0);

    let moved = before - distance_to_target(&camera);
    assert!(
        (moved - 2.0).abs() < 1e-3,
        "expected 2.0 units in one second, moved {moved}"
    );
}

#[test]
fn half_a_second_moves_half_the_distance() {
    let mut slow = Camera::default();
    let mut fast = Camera::default();

    let mut controller = CameraController::new(2.0);
    pressed(&mut controller, KeyCode::KeyW);
    controller.update_camera(&mut slow, 0.5);
    controller.update_camera(&mut fast, 1.0);

    let slow_moved = distance_to_target(&Camera::default()) - distance_to_target(&slow);
    let fast_moved = distance_to_target(&Camera::default()) - distance_to_target(&fast);
    assert!(
        (slow_moved * 2.0 - fast_moved).abs() < 1e-3,
        "half a second moved {slow_moved}, one second moved {fast_moved}"
    );
}

#[test]
fn strafe_keeps_the_eye_target_distance() {
    let mut camera = Camera::default();
    let before = distance_to_target(&camera);

    let mut controller = CameraController::new(2.0);
    pressed(&mut controller, KeyCode::KeyD);
    controller.update_camera(&mut camera, 0.25);

    let after = distance_to_target(&camera);
    assert!(
        (before - after).abs() < 1e-3,
        "strafe changed the radius: {before} -> {after}"
    );
}

#[test]
fn no_keys_means_no_movement() {
    let mut camera = Camera::default();
    let eye = camera.eye;

    let controller = CameraController::new(2.0);
    controller.update_camera(&mut camera, 1.0);

    assert_eq!(camera.eye, eye, "camera moved without a held key");
}

#[test]
fn is_moving_tracks_held_keys() {
    let mut controller = CameraController::new(2.0);
    assert!(!controller.is_moving());

    pressed(&mut controller, KeyCode::KeyW);
    assert!(controller.is_moving());

    controller.handle_key(KeyCode::KeyW, false);
    assert!(!controller.is_moving());
}
