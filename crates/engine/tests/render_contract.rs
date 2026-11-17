//! Headless GPU tests for the render contract: Vertex struct ↔ VertexBufferLayout ↔ WGSL,
//! plus the index buffer and texture bind group path.

mod common;

use common::{Frame, render_pentagon};
use engine::render::camera::Camera;

#[test]
fn pentagon_renders_from_vertex_and_index_buffers() {
    let (frame, texture) = render_pentagon(&Camera::default());

    assert_background_is_black(&frame);
    assert_center_samples_the_texture(&frame, &texture);
}

fn assert_background_is_black(frame: &Frame) {
    let bg = frame.px(2, 2);
    assert!(
        bg[0] < 10 && bg[1] < 10 && bg[2] < 10,
        "background not black: {bg:?}"
    );
}

fn assert_center_samples_the_texture(
    frame: &Frame,
    texture: &engine::asset::texture::TextureObject,
) {
    // The pentagon center sits at uv (0.5, 0.5), so the fragment color must come
    // from the middle of the diffuse texture. Allow for linear filtering between texels.
    let expected = texture
        .rgba()
        .get_pixel(texture.width() / 2, texture.height() / 2)
        .0;
    let center = frame.px(32, 32);
    assert!(
        center != [0, 0, 0, 255],
        "pentagon center not drawn: {center:?}"
    );
    for channel in 0..3 {
        let diff = center[channel].abs_diff(expected[channel]);
        assert!(
            diff <= 60,
            "channel {channel}: rendered {:?} vs texture {expected:?}",
            center
        );
    }
}
