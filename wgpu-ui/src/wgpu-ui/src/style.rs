pub struct ButtonStyle {
    pub bg_idle: [f32; 4],
    pub bg_hover: [f32; 4],
    pub bg_pressed: [f32; 4],
    pub bg_active: [f32; 4],            // new (same as pressed by default)
    pub bg_active_hover: [f32; 4],      // new (slightly brighter pressed)
    pub text_idle: [f32; 4],
    pub text_hover: [f32; 4],
    pub text_active: [f32; 4],          // new
    pub text_active_hover: [f32; 4],    // new
    pub text_size: f32,
    pub border_radius: f32,
}

impl ButtonStyle {
    pub fn primary() -> Self {
        Self {
            bg_idle: [0.18, 0.18, 0.18, 1.0],
            bg_hover: [0.25, 0.25, 0.25, 1.0],
            bg_pressed: [0.35, 0.35, 0.35, 1.0],
            bg_active: [0.35, 0.35, 0.35, 1.0],        // like pressed
            bg_active_hover: [0.45, 0.45, 0.45, 1.0],  // brighter
            text_idle: [0.9, 0.9, 0.9, 1.0],
            text_hover: [1.0, 1.0, 1.0, 1.0],
            text_active: [1.0, 1.0, 1.0, 1.0],          // same as hover
            text_active_hover: [1.0, 1.0, 1.0, 1.0],
            text_size: 14.0,
            border_radius: 4.0,
        }
    }


    pub fn danger() -> Self {
        Self {
            bg_idle: [0.0, 0.0, 0.0, 0.0],
            bg_hover: [0.91, 0.07, 0.07, 1.0],
            bg_pressed: [0.91, 0.07, 0.07, 0.6],
            bg_active: [0.91, 0.07, 0.07, 0.4],
            bg_active_hover: [0.91, 0.07, 0.07, 0.8],
            text_idle: [0.65, 0.65, 0.65, 1.0],
            text_hover: [0.65, 0.65, 0.65, 1.0],
            text_active: [0.65, 0.65, 0.65, 1.0],
            text_active_hover: [0.65, 0.65, 0.65, 1.0],
            text_size: 14.0,
            border_radius: 4.0,
        }
    }

     pub fn icon() -> Self {
        Self {
            bg_idle: [0.0, 0.0, 0.0, 0.0],
            bg_hover: [1.0, 1.0, 1.0, 0.1],
            bg_pressed: [1.0, 1.0, 1.0, 0.2],
            bg_active: [1.0, 1.0, 1.0, 0.2],            // like pressed
            bg_active_hover: [1.0, 1.0, 1.0, 0.3],      // slightly more opaque
            text_idle: [0.65, 0.65, 0.65, 1.0],
            text_hover: [1.0, 1.0, 1.0, 1.0],
            text_active: [1.0, 1.0, 1.0, 1.0],
            text_active_hover: [1.0, 1.0, 1.0, 1.0],
            text_size: 14.0,
            border_radius: 4.0,
        }
    }

    pub fn to_hover_effect(&self) -> HoverEffect {
        HoverEffect::Button {
            bg_idle: self.bg_idle,
            bg_hover: self.bg_hover,
            bg_pressed: self.bg_pressed,
            bg_active: self.bg_active,
            bg_active_hover: self.bg_active_hover,
            text_idle: self.text_idle,
            text_hover: self.text_hover,
            text_active: self.text_active,
            text_active_hover: self.text_active_hover,
            corner_radius: self.border_radius,
        }
    }
}
use crate::HoverEffect;
impl Default for ButtonStyle {
    fn default() -> Self {
        Self::primary()
    }
}
