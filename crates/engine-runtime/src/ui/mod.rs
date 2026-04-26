// pub mod header;
// pub mod dropdown;
// pub mod button;
// pub mod ui_zone;
// pub mod window;
// 
// pub use header::EngineHeader;
// pub use ui_zone::{determine_active_zone, UiZone, RuntimeZone};
// pub use window::{create_window, apply_platform_style};

// pub mod header;
// pub mod button;          // only ButtonStyle remains
// pub mod ui_zone;
// pub mod window;
// pub mod primitives;      // new
// #[macro_use]
// pub mod ui_macro;        // new
// 
// pub use header::EngineHeader;
// pub use ui_zone::{determine_active_zone, UiZone, RuntimeZone};
// pub use window::{create_window, apply_platform_style};

// pub mod dropdown;
pub mod header;
pub mod ui_zone;
pub mod window;

pub use header::EngineHeader;
pub use ui_zone::{determine_active_zone,  UiZone, RuntimeZone};
pub use window::{create_window, apply_platform_style};

// Re-export wgpu_ui for convenience
pub use wgpu_ui::{ButtonStyle, Primitive, HoverEffect, ui};