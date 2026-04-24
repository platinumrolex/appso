#[derive(Clone, Copy, Debug)]
pub struct ButtonStyle {
    pub bg_idle: [f32; 4],
    pub bg_hover: [f32; 4],
    pub bg_pressed: [f32; 4],
    pub text_idle: [f32; 4],
    pub text_hover: [f32; 4],
    pub text_size: f32,
    pub border_radius: f32,
}

impl ButtonStyle {
    pub fn primary() -> Self {
        Self {
            bg_idle: [0.18, 0.18, 0.18, 1.0],
            bg_hover: [0.25, 0.25, 0.25, 1.0],
            bg_pressed: [0.35, 0.35, 0.35, 1.0],
            text_idle: [0.9, 0.9, 0.9, 1.0],
            text_hover: [1.0, 1.0, 1.0, 1.0],
            text_size: 14.0,
            border_radius: 4.0,
        }
    }

    pub fn danger() -> Self {
        Self {
            bg_idle: [0.0, 0.0, 0.0, 0.0],
            bg_hover: [0.91, 0.07, 0.07, 1.0],
            bg_pressed: [0.70, 0.05, 0.05, 1.0],
            text_idle: [0.65, 0.65, 0.65, 1.0],
            text_hover: [1.0, 1.0, 1.0, 1.0],
            text_size: 14.0,
            border_radius: 4.0,
        }
    }

    pub fn icon() -> Self {
        Self {
            bg_idle: [0.0, 0.0, 0.0, 0.0],
            bg_hover: [1.0, 1.0, 1.0, 0.1],
            bg_pressed: [1.0, 1.0, 1.0, 0.2],
            text_idle: [0.65, 0.65, 0.65, 1.0],
            text_hover: [1.0, 1.0, 1.0, 1.0],
            text_size: 14.0,
            border_radius: 4.0,
        }
    }
}
use crate::HoverEffect;
impl Default for ButtonStyle {
    fn default() -> Self {
        Self::primary()
    }
}

impl ButtonStyle {
    pub fn to_hover_effect(&self) -> HoverEffect {
        // This is where you map the data once
        HoverEffect::Button {
            bg_idle: self.bg_idle,
            bg_hover: self.bg_hover,
            bg_pressed: self.bg_pressed,
            text_idle: self.text_idle,
            text_hover: self.text_hover,
            corner_radius: self.border_radius,
        }
    }
}