//! Minimal shared types for diagram apps.

pub const CANVAS_SIZE: f32 = 10000.0;
pub const INITIAL_WORLD_POS: f32 = CANVAS_SIZE / 8.0;

// -----------------------------------------------------------------------------
// Camera – shared by runtime and app
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub pan_x: f32,
    pub pan_y: f32,
    pub zoom: f32,
    pub screen_width: f32,
    pub screen_height: f32,
    pub scale_factor: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            pan_x: -INITIAL_WORLD_POS,
            pan_y: -INITIAL_WORLD_POS,
            zoom: 1.0,
            screen_width: 1.0,
            screen_height: 1.0,
            scale_factor: 1.0,
        }
    }

    pub fn effective_zoom(&self) -> f32 {
        self.zoom * self.scale_factor
    }

    pub fn screen_to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        (
            (sx - self.pan_x) / self.effective_zoom(),
            (sy - self.pan_y) / self.effective_zoom(),
        )
    }

    pub fn world_to_screen(&self, wx: f32, wy: f32) -> (f32, f32) {
        (
            (wx * self.effective_zoom()) + self.pan_x,
            (wy * self.effective_zoom()) + self.pan_y,
        )
    }

    pub fn is_visible(&self, wx: f32, wy: f32, w: f32, h: f32) -> bool {
        let (sx, sy) = self.world_to_screen(wx, wy);
        let padding = 100.0;
        sx < self.screen_width + padding
            && (sx + w * self.effective_zoom()) > -padding
            && sy < self.screen_height + padding
            && (sy + h * self.effective_zoom()) > -padding
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

// -----------------------------------------------------------------------------
// Node data – app specific
// -----------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct NodeData {
    pub id: i32,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub entries: Vec<String>,
}

impl NodeData {
    pub fn new(id: i32, name: &str, x: f32, y: f32, entries: Vec<String>) -> Self {
        let height = 30.0 + (entries.len() as f32 * 24.0) + 10.0;
        Self {
            id,
            name: name.to_string(),
            x,
            y,
            width: 180.0,
            height,
            entries,
        }
    }
}