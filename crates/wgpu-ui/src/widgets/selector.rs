use wgpu_text::glyph_brush::{HorizontalAlign, VerticalAlign};
use crate::{widget, primitives::{HitRegion, HoverEffect, Primitive}, style::ButtonStyle};

#[derive(Clone)]
pub struct SelectorOption<A> {
    pub label: String,
    pub selected: bool,
    pub action: A,
}

widget! {
    pub Selector<A> {
        label: String,
        current: String,
        toggle_action: A,
        expanded: bool = false,
        style: ButtonStyle = ButtonStyle::primary(),
        options: Vec<SelectorOption<A>> = Vec::new(),
    }
    render: |this, prims, hits| {
        let x = this.bounds.x;
        let y = this.bounds.y;
        let w = this.bounds.w;
        let h = this.bounds.h;

        let label_x = x + 16.0;
        let box_x = x + 120.0;
        let box_w = w;
        let box_h = h - 8.0;
        let box_y = y + 4.0;

        // Main panel background
        prims.push(Primitive::Rect {
            x, y,
            w: 120.0 + box_w,
            h,
            color: [0.12, 0.12, 0.12, 0.98],
            corner_radius: 0.0,
        });

        // Label text
        prims.push(Primitive::Text {
            content: this.label.clone(),
            x: label_x,
            y: y + h / 2.0,
            color: [0.9, 0.9, 0.9, 1.0],
            size: 14.0,
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Center,
        });

        // Selector box background
        let box_bg_color = if this.expanded { this.style.bg_pressed } else { this.style.bg_idle };
        prims.push(Primitive::Rect {
            x: box_x, y: box_y, w: box_w, h: box_h,
            color: box_bg_color,
            corner_radius: this.style.border_radius,
        });

        // Current value text
        prims.push(Primitive::Text {
            content: this.current.clone(),
            x: box_x + 12.0,
            y: box_y + box_h / 2.0,
            color: this.style.text_idle,
            size: this.style.text_size,
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Center,
        });

        // Dropdown arrow
        prims.push(Primitive::Text {
            content: "⌵".to_string(),
            x: box_x + box_w - 20.0,
            y: box_y + box_h / 2.0,
            color: [0.6, 0.6, 0.6, 1.0],
            size: this.style.text_size,
            h_align: HorizontalAlign::Center,
            v_align: VerticalAlign::Center,
        });

        // Hit region for selector box
        hits.push(HitRegion {
            bounds: crate::Rect { x: box_x, y: box_y, w: box_w, h: box_h },
            action: this.toggle_action,
            hover: HoverEffect::Button {
                bg_idle: this.style.bg_idle,
                bg_hover: this.style.bg_hover,
                bg_pressed: this.style.bg_pressed,
                text_idle: this.style.text_idle,
                text_hover: this.style.text_hover,
                corner_radius: this.style.border_radius,
            },
        });

        // Options (if expanded)
        if this.expanded {
            let opt_count = this.options.len() as f32;
            prims.push(Primitive::Rect {
                x: box_x, y: box_y, w: box_w, h: opt_count * h,
                color: [0.18, 0.18, 0.18, 1.0],
                corner_radius: 0.0,
            });

            let mut opt_y = box_y;
            for opt in &this.options {
                let is_selected = opt.selected;
                let bg_color = if is_selected {
                    [0.25, 0.25, 0.25, 1.0]
                } else {
                    [0.18, 0.18, 0.18, 1.0]
                };
                prims.push(Primitive::Rect {
                    x: box_x, y: opt_y, w: box_w, h,
                    color: bg_color,
                    corner_radius: 0.0,
                });
                if is_selected {
                    prims.push(Primitive::Rect {
                        x: box_x + 2.0, y: opt_y + 4.0, w: 3.0, h: h - 8.0,
                        color: [0.0, 0.8, 0.2, 1.0],
                        corner_radius: 1.5,
                    });
                }
                let text_color = if is_selected {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    [0.8, 0.8, 0.8, 1.0]
                };
                prims.push(Primitive::Text {
                    content: opt.label.clone(),
                    x: box_x + 12.0,
                    y: opt_y + h / 2.0,
                    color: text_color,
                    size: this.style.text_size,
                    h_align: HorizontalAlign::Left,
                    v_align: VerticalAlign::Center,
                });
                if is_selected {
                    prims.push(Primitive::Text {
                        content: "✓".to_string(),
                        x: box_x + box_w - 16.0,
                        y: opt_y + h / 2.0,
                        color: [0.0, 0.8, 0.2, 1.0],
                        size: this.style.text_size,
                        h_align: HorizontalAlign::Center,
                        v_align: VerticalAlign::Center,
                    });
                }
                hits.push(HitRegion {
                    bounds: crate::Rect { x: box_x, y: opt_y, w: box_w, h },
                    action: opt.action,
                    hover: HoverEffect::Highlight {
                        bg_hover: if is_selected { [0.30, 0.30, 0.30, 1.0] } else { [0.22, 0.22, 0.22, 1.0] },
                        bg_pressed: [0.35, 0.35, 0.35, 1.0],
                    },
                });
                opt_y += h;
            }
        }
    }
}