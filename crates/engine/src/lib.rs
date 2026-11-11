pub mod app;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    event_loop.run_app(&mut app::event_loop::App::default())?;
    Ok(())
}
