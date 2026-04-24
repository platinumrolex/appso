use wgpu_text::glyph_brush::{HorizontalAlign, VerticalAlign};
use crate::{widget, primitives::{Interaction, Primitive}, style::ButtonStyle};

widget! {
    pub Button<A> {
        label: String,
        action: A,
        style: ButtonStyle = ButtonStyle::primary(),
    }
    render: |this, prims| {
        let x = this.bounds.x;
        let y = this.bounds.y;
        let w = this.bounds.w;
        let h = this.bounds.h;

        let interaction = Interaction {
            action: this.action,
            hover_effect: this.style.to_hover_effect(),
            bounds: this.bounds,
        };

        prims.push(Primitive::Rect {
            x, y, w, h,
            color: this.style.bg_idle,
            corner_radius: this.style.border_radius,
            interaction: Some(interaction.clone()),
        });
        prims.push(Primitive::Text {
            content: this.label.clone(),
            x: x + w / 2.0,
            y: y + h / 2.0,
            color: this.style.text_idle,
            size: this.style.text_size,
            h_align: HorizontalAlign::Center,
            v_align: VerticalAlign::Center,
            interaction: Some(interaction),
        });
    }
}