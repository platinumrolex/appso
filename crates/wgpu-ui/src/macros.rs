#[macro_export]
macro_rules! ui {
    // -------------------------------------------------------------------------
    // PERFORMANCE ENTRY POINT
    // -------------------------------------------------------------------------
    (@to $self:expr, $prims:expr, { $($body:tt)* }) => {{
        #[allow(unused_imports)]
        let __ui_self = $self;
        $crate::ui!(@parse __ui_self, $prims, $($body)*);
    }};

    // =========================================================================
    // PARSER
    // =========================================================================

    (@parse $ctx:ident, $prims:expr, $(,)?) => {};

    // root
    (@parse $ctx:ident, $prims:expr,
        root { $($body:tt)* } $($rest:tt)*
    ) => {
        $crate::ui!(@parse $ctx, $prims, $($body)*);
        $crate::ui!(@parse $ctx, $prims, $($rest)*);
    };

    // -------------------------------------------------------------------------
    // SELECTOR ARM
    // MUST COME BEFORE GENERIC WIDGET ARM
    // -------------------------------------------------------------------------
    (@parse $ctx:ident, $prims:expr,
        $id:ident : Selector {
            label: $label:expr,
            current: $current:expr,
            options: $options:expr $(,)?
        } at ($x:expr, $y:expr, $w:expr, $h:expr)
        $($rest:tt)*
    ) => {{
        let bounds = $crate::Rect {
            x: $x,
            y: $y,
            w: $w,
            h: $h,
        };

        let widget = {
            use $crate::Selector;

            Selector::builder(bounds)
                .id(stringify!($id))
                ._is_open($ctx.selector_open(stringify!($id)))
                .toggle_action(
                    EngineHeaderAction::ToggleSelector(
                        stringify!($id)
                    )
                )
                .label($label)
                .current($current)
                .options($options)
                .build()
        };

        #[allow(unused_imports)]
        use $crate::Widget as _;

        widget.render($prims);

        $crate::ui!(@parse $ctx, $prims, $($rest)*);
    }};

    // -------------------------------------------------------------------------
    // CONTAINER ARM
    // -------------------------------------------------------------------------
    (@parse $ctx:ident, $prims:expr,
        $id:ident : Container {
        } children: { $($children:tt)* }
        $($rest:tt)*
    ) => {{
        let children_widgets =
            $crate::ui!(@parse_children $ctx, $($children)*);

        let widget = $crate::widgets::Container::builder(
            $crate::Rect::default()
        )
        .children(children_widgets)
        .build();

        #[allow(unused_imports)]
        use $crate::Widget as _;

        widget.render($prims);

        $crate::ui!(@parse $ctx, $prims, $($rest)*);
    }};

    // -------------------------------------------------------------------------
    // GENERIC WIDGET ARM
    // -------------------------------------------------------------------------
    (@parse $ctx:ident, $prims:expr,
        $id:ident : $widget_ty:path {
            $($field:ident : $value:expr),* $(,)?
        } at ($x:expr, $y:expr, $w:expr, $h:expr)
        $($rest:tt)*
    ) => {{
        let bounds = $crate::Rect {
            x: $x,
            y: $y,
            w: $w,
            h: $h,
        };

        let widget = {
            use $widget_ty as W;

            W::builder(bounds)
                $(.$field($value))*
                .build()
        };

        #[allow(unused_imports)]
        use $crate::Widget as _;

        widget.render($prims);

        $crate::ui!(@parse $ctx, $prims, $($rest)*);
    }};

    // -------------------------------------------------------------------------
    // GENERIC ZONE ARM
    // -------------------------------------------------------------------------
    (@parse $ctx:ident, $prims:expr,
        $id:ident { $($body:tt)* } $($rest:tt)*
    ) => {
        $crate::ui!(@parse $ctx, $prims, $($body)*);
        $crate::ui!(@parse $ctx, $prims, $($rest)*);
    };

    // =========================================================================
    // CHILDREN
    // =========================================================================

    (@parse_children $ctx:ident, $($body:tt)*) => {{
        let mut children: Vec<Box<dyn $crate::Widget<_>>> =
            Vec::new();

        $crate::ui!(@collect_children $ctx, children, $($body)*);

        children
    }};

    (@collect_children $ctx:ident, $vec:ident,
        $id:ident : $widget_ty:path {
            $($field:ident : $value:expr),* $(,)?
        } at ($x:expr, $y:expr, $w:expr, $h:expr)
        $($rest:tt)*
    ) => {{
        let bounds = $crate::Rect {
            x: $x,
            y: $y,
            w: $w,
            h: $h,
        };

        let widget = {
            use $widget_ty as W;

            W::builder(bounds)
                $(.$field($value))*
                .build()
        };

        $vec.push(Box::new(widget));

        $crate::ui!(
            @collect_children
            $ctx,
            $vec,
            $($rest)*
        );
    }};

    (@collect_children $ctx:ident, $vec:ident, $(,)?) => {};

    // =========================================================================
    // STANDARD ENTRY
    // =========================================================================
    ($($body:tt)+) => {{
        let mut primitives = Vec::new();

        let __ui_ctx = ();

        $crate::ui!(
            @parse
            __ui_ctx,
            primitives,
            $($body)+
        );

        primitives
    }};
}

#[macro_export]
macro_rules! section {
    // ----------------------------------------
    // ENTRY
    // ----------------------------------------
    (Action = $action_ty:ident, $($body:tt)*) => {{
        let mut __prims = Vec::new();
        $crate::section!(@zones __prims, $action_ty, $($body)*);
        __prims
    }};

    // ----------------------------------------
    // ZONE PARSER (ONE ZONE AT A TIME)
    // ----------------------------------------

    // done
    (@zones $prims:expr, $action_ty:ident,) => {};

    // zone WITH trailing comma
    (@zones $prims:expr, $action_ty:ident,
        $zone:ident { $($inner:tt)* },
        $($rest:tt)*
    ) => {{
        $crate::section!(@elems $prims, $action_ty, $($inner)*);
        $crate::section!(@zones $prims, $action_ty, $($rest)*);
    }};

    // LAST zone (no comma)
    (@zones $prims:expr, $action_ty:ident,
        $zone:ident { $($inner:tt)* }
    ) => {{
        $crate::section!(@elems $prims, $action_ty, $($inner)*);
    }};

    // ----------------------------------------
    // ELEMENT PARSER (SAFE)
    // ----------------------------------------

    (@elems $prims:expr, $action_ty:ident,) => {};

    // element with comma
    (@elems $prims:expr, $action_ty:ident,
        $elem:ident ( $($args:tt)* ),
        $($rest:tt)*
    ) => {{
        $crate::$elem!(@render &mut $prims, $action_ty, $($args)*);
        $crate::section!(@elems $prims, $action_ty, $($rest)*);
    }};

    // last element (no comma)
    (@elems $prims:expr, $action_ty:ident,
        $elem:ident ( $($args:tt)* ) $(,)?
    ) => {{
        $crate::$elem!(@render &mut $prims, $action_ty, $($args)*);
    }};
}