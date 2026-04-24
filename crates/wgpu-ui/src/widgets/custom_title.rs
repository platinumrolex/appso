// use wgpu_text::glyph_brush::{HorizontalAlign, VerticalAlign};
// use crate::{widget, primitives::Primitive};
// 
// widget! {
//     pub CustomTitle<A> {
//         text: String,
//         size: f32,
//         color: [f32; 4],
//         _action: Option<A> = None,
//     }
//     render: |this, prims, _hits| {
//         prims.push(Primitive::Text {
//             content: this.text.clone(),
//             x: this.bounds.x,
//             y: this.bounds.y,
//             color: this.color,
//             size: this.size,
//             h_align: HorizontalAlign::Left,
//             v_align: VerticalAlign::Top,
//         });
//     }
// }

// crates/wgpu-ui/src/widgets/custom_title.rs
use wgpu_text::glyph_brush::{HorizontalAlign, VerticalAlign};
use crate::{widget, primitives::Primitive};

widget! {
    pub CustomTitle<A> {
        text: String,
        size: f32,
        color: [f32; 4],
    }
    render: |this, prims, _hits| {
        prims.push(Primitive::Text {
            content: this.text.clone(),
            x: this.bounds.x,
            y: this.bounds.y,
            color: this.color,
            size: this.size,
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Top,
        });
    }
}