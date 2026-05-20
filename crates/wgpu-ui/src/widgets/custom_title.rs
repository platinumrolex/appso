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

#[macro_export]
macro_rules! title {
    (@render $prims:expr, $action_ty:ident,
        $text:expr,
        pos($x:expr, $y:expr)
    ) => {{
        let bounds = $crate::Rect { x: $x, y: $y, w: 0.0, h: 0.0 };

        let widget = $crate::CustomTitle::builder(bounds)
            .text(($text).into())
            .size(13.0)
            .color([0.9, 0.9, 0.9, 1.0])
            .build();

        use $crate::Widget as _;
        widget.render($prims);
    }};
}