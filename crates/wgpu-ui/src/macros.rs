#[macro_export]
macro_rules! ui {
    // -------------------------------------------------------------------------
    // PERFORMANCE ENTRY POINT: Render directly into an existing &mut Vec
    // Usage: ui!(@to my_primitives_vec, { root { ... } });
    // -------------------------------------------------------------------------
    (@to $prims:ident, { $($body:tt)* }) => {
        $crate::ui!(@parse $prims, $($body)*);
    };

    // Base case: end of parsing
    (@parse $prims:ident, $(,)?) => {};

    // Support nested 'root' block often used in blueprints
    (@parse $prims:ident, root { $($body:tt)* } $($rest:tt)*) => {
        $crate::ui!(@parse $prims, $($body)*);
        $crate::ui!(@parse $prims, $($rest)*);
    };

    // Container arm (must come BEFORE the generic $id:ident { ... } arm)
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
        widget.render($prims);
        $crate::ui!(@parse $prims, $($rest)*);
    }};

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
        widget.render($prims);
        $crate::ui!(@parse $prims, $($rest)*);
    }};

    // Generic zone arm – matches Header { ... }, Dropdown { ... }, etc.
    // (any identifier directly followed by a brace block)
    (@parse $prims:ident,
        $id:ident { $($body:tt)* } $($rest:tt)*
    ) => {
        $crate::ui!(@parse $prims, $($body)*);
        $crate::ui!(@parse $prims, $($rest)*);
    };

    // Children parsing logic
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

    // =========================================================================
    // Standard entry point (allocates new Vec - use sparingly)
    // MUST BE AT THE BOTTOM so it doesn't accidentally intercept internal @parse 
    // or @collect invocations, which are valid tt streams!
    // =========================================================================
    ($($body:tt)+) => {{
        let mut primitives = Vec::new();
        $crate::ui!(@parse primitives, $($body)+);
        primitives
    }};
}