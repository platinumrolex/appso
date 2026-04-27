#![recursion_limit = "4096"]
pub mod core;
pub mod ui;

use ui::header::{EngineHeader, ScaledMetrics, RuntimeZone};
use ui::ui_zone::{determine_active_zone, UiZone};

pub use core::frame_delay::calculate_frame_delay;
pub use core::graphics::{init_graphics, get_window_refresh_rate, WgpuState};

pub struct EngineState {
    pub header: EngineHeader,
    pub active_zone: UiZone,
}

impl EngineState {
    pub fn new() -> Self {
        Self {
            header: EngineHeader::new("DB DIAGRAM PRO"),
            active_zone: UiZone::App,
        }
    }
}