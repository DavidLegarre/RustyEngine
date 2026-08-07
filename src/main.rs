mod app;

use crate::app::App;
fn main() {
    env_logger::init();
    let app = App::new();
    println!("Hello, world!");
}
