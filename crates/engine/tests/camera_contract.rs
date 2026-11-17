//! Camera contract: the uniform must actually reach the vertex shader.

mod common;

use common::render_pentagon;
use engine::render::camera::Camera;

#[test]
fn camera_uniform_moves_the_rendered_scene() {
    let centred = render_pentagon(&Camera::default()).0;

    // Same eye, but look at a point to the right of the origin, which pushes the
    // pentagon off-centre without changing the camera position or the mesh.
    let offset_camera = Camera {
        target: (0.5, 0.0, 0.0).into(),
        ..Camera::default()
    };
    let offset = render_pentagon(&offset_camera).0;

    let centred_x = centred.centroid_x();
    let offset_x = offset.centroid_x();

    assert!(
        centred_x >= 0.0 && offset_x >= 0.0,
        "nothing was drawn: centred={centred_x} offset={offset_x}"
    );
    assert!(
        (centred_x - offset_x).abs() > 3.0,
        "camera uniform ignored: centred_x={centred_x} offset_x={offset_x}"
    );
}
