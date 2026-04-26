use wgpu_text::glyph_brush::{HorizontalAlign, VerticalAlign};
use crate::{widget, primitives::Primitive};

widget! {
    pub CustomTitle<A> {
        text: String,
        size: f32,
        color: [f32; 4],
    }
    render: |this, prims| {
        prims.push(Primitive::Text {
            content: this.text.clone(),
            x: this.bounds.x,
            y: this.bounds.y,
            color: this.color,
            size: this.size,
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Top,
            interaction: None,
        });
    }
}