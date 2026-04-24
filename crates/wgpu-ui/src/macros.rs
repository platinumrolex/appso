#[macro_export]
macro_rules! ui {
    (root { $($body:tt)* }) => {{
        let mut primitives = Vec::new();
        $crate::ui!(@parse primitives, $($body)*);
        primitives
    }};

    (@parse $prims:ident, $(,)?) => {};

    // Standard widget arm
    (@parse $prims:ident,
        $id:ident : $widget_ty:path {
            $($field:ident : $value:expr),* $(,)?
        } at ($x:expr, $y:expr, $w:expr, $h:expr)
        $($rest:tt)*
    ) => {{
        let bounds = $crate::Rect { x: $x, y: $y, w: $w, h: $h };
        let widget = {
            use $widget_ty as W;
            W::builder(bounds)
                $(.$field($value))*
                .build()
        };
        #[allow(unused_imports)]
        use $crate::Widget as _;
        widget.render(&mut $prims);
        $crate::ui!(@parse $prims, $($rest)*);
    }};

    // Container arm
    (@parse $prims:ident,
        $id:ident : Container {
        } children: { $($children:tt)* }
        $($rest:tt)*
    ) => {{
        let children_widgets = $crate::ui!(@parse_children $($children)*);
        let widget = $crate::widgets::Container::builder($crate::Rect::default())
            .children(children_widgets)
            .build();
        #[allow(unused_imports)]
        use $crate::Widget as _;
        widget.render(&mut $prims);
        $crate::ui!(@parse $prims, $($rest)*);
    }};

    (@parse_children $($body:tt)*) => {{
        let mut children: Vec<Box<dyn $crate::Widget<_>>> = Vec::new();
        $crate::ui!(@collect_children children, $($body)*);
        children
    }};

    (@collect_children $vec:ident,
        $id:ident : $widget_ty:path {
            $($field:ident : $value:expr),* $(,)?
        } at ($x:expr, $y:expr, $w:expr, $h:expr)
        $($rest:tt)*
    ) => {{
        let bounds = $crate::Rect { x: $x, y: $y, w: $w, h: $h };
        let widget = {
            use $widget_ty as W;
            W::builder(bounds)
                $(.$field($value))*
                .build()
        };
        $vec.push(Box::new(widget));
        $crate::ui!(@collect_children $vec, $($rest)*);
    }};

    (@collect_children $vec:ident, $(,)?) => {};
}