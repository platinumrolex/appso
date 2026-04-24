use wgpu_text::glyph_brush::{HorizontalAlign, VerticalAlign};

#[derive(Clone, Debug)]
pub enum Primitive {
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
        corner_radius: f32,
    },
    Text {
        content: String,
        x: f32,
        y: f32,
        color: [f32; 4],
        size: f32,
        h_align: HorizontalAlign,
        v_align: VerticalAlign,
    },
}

#[derive(Clone, Debug)]
pub struct HitRegion<A> {
    pub bounds: Rect,
    pub action: A,
    pub hover: HoverEffect,
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn contains(&self, point: (f32, f32)) -> bool {
        point.0 >= self.x && point.0 <= self.x + self.w &&
        point.1 >= self.y && point.1 <= self.y + self.h
    }
}

#[derive(Clone, Copy, Debug)]
pub enum HoverEffect {
    None,
    Button {
        bg_idle: [f32; 4],
        bg_hover: [f32; 4],
        bg_pressed: [f32; 4],
        text_idle: [f32; 4],
        text_hover: [f32; 4],
        corner_radius: f32,
    },
    Highlight {
        bg_hover: [f32; 4],
        bg_pressed: [f32; 4],
    },
}

impl HoverEffect {
    pub fn resolve_bg(&self, is_hovered: bool, is_pressed: bool) -> Option<[f32; 4]> {
        match self {
            HoverEffect::None => None,
            HoverEffect::Button { bg_idle, bg_hover, bg_pressed, .. } => {
                Some(if is_hovered && is_pressed {
                    *bg_pressed
                } else if is_hovered {
                    *bg_hover
                } else {
                    *bg_idle
                })
            }
            HoverEffect::Highlight { bg_hover, bg_pressed } => {
                if is_hovered && is_pressed {
                    Some(*bg_pressed)
                } else if is_hovered {
                    Some(*bg_hover)
                } else {
                    None
                }
            }
        }
    }

    pub fn resolve_text(&self, is_hovered: bool) -> Option<[f32; 4]> {
        match self {
            HoverEffect::Button { text_idle, text_hover, .. } => {
                Some(if is_hovered { *text_hover } else { *text_idle })
            }
            _ => None,
        }
    }

    pub fn corner_radius(&self) -> f32 {
        match self {
            HoverEffect::Button { corner_radius, .. } => *corner_radius,
            _ => 0.0,
        }
    }
}

// In a common location, e.g., crates/wgpu-ui/src/primitives.rs or a new file
pub trait UiAction: PartialEq + Copy + std::fmt::Debug {
    /// Returns true if this action represents an interactive element (like a button)
    /// and should trigger hover effects/redraws.
    fn is_interactive(&self) -> bool;
}