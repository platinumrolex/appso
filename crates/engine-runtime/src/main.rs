mod app;

use app::App;
use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::<app::AppEvent>::with_user_event().build().unwrap();
    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app).unwrap();
}

