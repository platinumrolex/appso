use crate::{widget, primitives::Primitive};

widget! {
    pub CustomDot<A> {
        color: [f32; 4],
        size: f32,
        _action: Option<A> = None,
    }
    render: |this, prims, _hits| {
        prims.push(Primitive::Rect {
            x: this.bounds.x - this.size / 2.0,
            y: this.bounds.y - this.size / 2.0,
            w: this.size,
            h: this.size,
            color: this.color,
            corner_radius: this.size / 2.0,
        });
    }
}