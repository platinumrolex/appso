use crate::{widget, Widget};

widget! {
    pub Container<A> {
        children: Vec<Box<dyn Widget<A>>> = Vec::new(),
    }
    render: |this, prims| {
        for child in &this.children {
            child.render(prims);
        }
    }
}