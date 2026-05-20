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



#[macro_export]
macro_rules! button {
    // NO STYLE
    (@render $prims:expr, $action_ty:ident,
        $label:expr,
        @$action:ident,
        $style:ident,
        pos($x:expr, $y:expr, $w:expr, $h:expr)
    ) => {{
        let bounds = $crate::Rect { x: $x, y: $y, w: $w, h: $h };
        let style = $crate::ButtonStyle::$style();

        let widget = $crate::Button::builder(bounds)
            .label($label.into())
            .action($action_ty::$action)
            .style(style)
            .build();

        use $crate::Widget as _;
        widget.render($prims);
    }};

    // EMPTY {}
    (@render $prims:expr, $action_ty:ident,
        $label:expr,
        @$action:ident,
        $style:ident { },
        pos($x:expr, $y:expr, $w:expr, $h:expr)
    ) => {{
        let bounds = $crate::Rect { x: $x, y: $y, w: $w, h: $h };
        let style = $crate::ButtonStyle::$style();

        let widget = $crate::Button::builder(bounds)
            .label($label.into())
            .action($action_ty::$action)
            .style(style)
            .build();

        use $crate::Widget as _;
        widget.render($prims);
    }};

    // WITH ARGS
    (@render $prims:expr, $action_ty:ident,
        $label:expr,
        @$action:ident,
        $style:ident { $($style_args:tt)+ },
        pos($x:expr, $y:expr, $w:expr, $h:expr)
    ) => {{
        let bounds = $crate::Rect { x: $x, y: $y, w: $w, h: $h };
        let mut style = $crate::ButtonStyle::$style();

        $crate::button!(@apply style, $($style_args)+,);

        let widget = $crate::Button::builder(bounds)
            .label($label.into())
            .action($action_ty::$action)
            .style(style)
            .build();

        use $crate::Widget as _;
        widget.render($prims);
    }};

    // ----------------------------------------
    // BASE CASES
    // ----------------------------------------
    (@apply $style:ident,) => {};
    (@apply $style:ident) => {}; // ← IMPORTANT (handles no trailing comma)

    // ----------------------------------------
    // FIELD: text_size
    // ----------------------------------------
    (@apply $style:ident, text_size : $val:expr $(, $($rest:tt)*)? ) => {{
        $style.text_size = $val;
        $crate::button!(@apply $style $(, $($rest)*)?);
    }};
}