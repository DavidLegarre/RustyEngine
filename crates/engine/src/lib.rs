use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut App { window: None })?;
    Ok(())
}
