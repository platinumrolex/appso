use wgpu_text::glyph_brush::{HorizontalAlign, VerticalAlign};
use crate::{
    widget,
    primitives::{Interaction, Primitive, HoverEffect},
    style::ButtonStyle,
};


#[derive(Clone)]
pub struct SelectorOption<A> {
    pub label: String,
    pub selected: bool,
    pub action: A,
}

widget! {
    pub Selector<A> {
        id: &'static str,

        label: String,
        current: String,

        toggle_action: A,

        style: ButtonStyle = ButtonStyle::primary(),
        options: Vec<SelectorOption<A>> = Vec::new(),

        _is_open: bool = false,
    }

    render: |this, prims| {
        let x = this.bounds.x;
        let y = this.bounds.y;
        let w = this.bounds.w;
        let h = this.bounds.h;

        let label_x = x + 16.0;
        let box_x = x + 120.0;
        let box_w = w;
        let box_h = h;
        let box_y = y;

        // background
        prims.push(Primitive::Rect {
            x, y,
            w: 120.0 + box_w,
            h,
            color: [0.12, 0.12, 0.12, 0.98],
            corner_radius: 0.0,
            interaction: None,
        });

        prims.push(Primitive::Text {
            content: this.label.clone(),
            x: label_x,
            y: y + h / 2.0,
            color: [0.9, 0.9, 0.9, 1.0],
            size: 14.0,
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Center,
            interaction: None,
        });

        // OPEN/CLOSE TOGGLE (framework-owned)
       // let toggle = crate::EngineHeaderAction::ToggleSelector(this.id);

        let box_interaction = Interaction {
            action: this.toggle_action,
            hover_effect: this.style.to_hover_effect(),
            bounds: crate::Rect { x: box_x, y: box_y, w: box_w, h: box_h },
        };

        if !this._is_open {
            prims.push(Primitive::Rect {
                x: box_x,
                y: box_y,
                w: box_w,
                h: box_h,
                color: this.style.bg_idle,
                corner_radius: this.style.border_radius,
                interaction: Some(box_interaction.clone()),
            });

            prims.push(Primitive::Text {
                content: this.current.clone(),
                x: box_x + 12.0,
                y: box_y + box_h / 2.0,
                color: this.style.text_idle,
                size: this.style.text_size,
                h_align: HorizontalAlign::Left,
                v_align: VerticalAlign::Center,
                interaction: Some(box_interaction),
            });
        }

        // OPTIONS
        // OPTIONS
if this._is_open {
    let opt_count = this.options.len() as f32;

    let dropdown_y = box_y;
    let dropdown_h = opt_count * h;

    // solid dropdown backdrop
    prims.push(Primitive::Rect {
        x: box_x,
        y: dropdown_y,
        w: box_w,
        h: dropdown_h,
        color: [0.14, 0.14, 0.14, 1.0],
        corner_radius: this.style.border_radius,
        interaction: None,
    });

    let mut opt_y = dropdown_y;

    for opt in &this.options {
        let selected = opt.selected;

        let idle_bg = if selected {
            [0.26, 0.26, 0.26, 1.0]
        } else {
            [0.18, 0.18, 0.18, 1.0]
        };

        let hover_bg = if selected {
            [0.32, 0.32, 0.32, 1.0]
        } else {
            [0.24, 0.24, 0.24, 1.0]
        };

        let pressed_bg = if selected {
            [0.38, 0.38, 0.38, 1.0]
        } else {
            [0.30, 0.30, 0.30, 1.0]
        };

        let inter = Interaction {
            action: opt.action,
            hover_effect: HoverEffect::Highlight {
                bg_hover: hover_bg,
                bg_pressed: pressed_bg,
            },
            bounds: crate::Rect {
                x: box_x,
                y: opt_y,
                w: box_w,
                h,
            },
        };

        // actual clickable/background layer
        prims.push(Primitive::Rect {
            x: box_x,
            y: opt_y,
            w: box_w,
            h,
            color: idle_bg,
            corner_radius: 0.0,
            interaction: Some(inter.clone()),
        });

        prims.push(Primitive::Text {
            content: opt.label.clone(),
            x: box_x + 12.0,
            y: opt_y + h / 2.0,
            color: [0.88, 0.88, 0.88, 1.0],
            size: this.style.text_size,
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Center,
            interaction: Some(inter.clone()),
        });

        opt_y += h;
    }
}
    }
}