use wgpu_text::glyph_brush::{HorizontalAlign, VerticalAlign};
use crate::{widget, primitives::{HitRegion, HoverEffect, Primitive}, style::ButtonStyle};

widget! {
    pub Button<A> {
        label: String,
        action: A,
        style: ButtonStyle = ButtonStyle::primary(),
    }
    render: |this, prims, hits| {
        let x = this.bounds.x;
        let y = this.bounds.y;
        let w = this.bounds.w;
        let h = this.bounds.h;

        prims.push(Primitive::Rect {
            x, y, w, h,
            color: this.style.bg_idle,
            corner_radius: this.style.border_radius,
        });
        prims.push(Primitive::Text {
            content: this.label.clone(),
            x: x + w / 2.0,
            y: y + h / 2.0,
            color: this.style.text_idle,
            size: this.style.text_size,
            h_align: HorizontalAlign::Center,
            v_align: VerticalAlign::Center,
        });
        hits.push(HitRegion {
            bounds: this.bounds,
            action: this.action,
            hover: this.style.to_hover_effect(),
            // HoverEffect::Button {
            //     bg_idle: this.style.bg_idle,
            //     bg_hover: this.style.bg_hover,
            //     bg_pressed: this.style.bg_pressed,
            //     text_idle: this.style.text_idle,
            //     text_hover: this.style.text_hover,
            //     corner_radius: this.style.border_radius,
            // },
        });
    }
}