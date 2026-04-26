use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, Icon};

pub fn create_window(event_loop: &ActiveEventLoop, title: &str) -> Arc<Window> {
    Arc::new(
        event_loop
            .create_window(
                Window::default_attributes()
                    .with_title(title)
                    .with_window_icon(Some(create_test_icon()))
                    .with_decorations(true)
            )
            .unwrap(),
    )
}

fn create_test_icon() -> Icon {
    let width = 32;
    let height = 32;
    let mut rgba = Vec::with_capacity(width * height * 4);
    for _ in 0..(width * height) {
        rgba.extend_from_slice(&[0, 0, 180, 255]); 
    }
    Icon::from_rgba(rgba, width as u32, height as u32).expect("Failed to create icon")
}

pub fn apply_platform_style(window: &Window) {
    apply_platform_visual(window);
}

pub fn apply_platform_visual(window: &Window) {
    #[cfg(target_os = "windows")]
    {
        use winit::platform::windows::WindowExtWindows;
        window.set_undecorated_shadow(true);
    }
}

#[derive(Clone)]
pub struct WindowStyle {
    pub border_color: [f32; 4],
    pub border_width: f32,
    pub corner_radius: f32,
    pub title_bar_height: f32,
    pub title_bar_color: [f32; 4],
    pub background_color: [f32; 4],
}

impl Default for WindowStyle {
    fn default() -> Self {
        Self {
            border_color: [0.15, 0.15, 0.15, 1.0],
            border_width: 1.0,
            corner_radius: 12.0,
            title_bar_height: 32.0,
            title_bar_color: [0.10, 0.10, 0.10, 1.0],
            background_color: [0.08, 0.08, 0.08, 1.0],
        }
    }
}