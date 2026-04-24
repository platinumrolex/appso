use crate::primitives::{Interaction, Primitive, Rect};
use std::marker::PhantomData;

pub trait Widget<A: Copy> {
    fn render(&self, prims: &mut Vec<Primitive<A>>);
}

#[macro_export]
macro_rules! widget {
    (
        $(#[$meta:meta])*
        $vis:vis $name:ident<$A:ident> {
            $($field:ident : $ty:ty $(= $default:expr)?),* $(,)?
        }
        render: |$this:pat, $prims:ident| $body:block
    ) => {
        $(#[$meta])*
        $vis struct $name<$A> {
            pub bounds: $crate::Rect,
            $(pub $field : $ty,)*
            _marker: std::marker::PhantomData<$A>,
        }

        $vis struct Builder<$A> {
            pub bounds: $crate::Rect,
            $(pub $field: Option<$ty>,)*
            _marker: std::marker::PhantomData<$A>,
        }

        impl<$A> Builder<$A> {
            $(
                pub fn $field(mut self, val: $ty) -> Self {
                    self.$field = Some(val);
                    self
                }
            )*

            pub fn build(self) -> $name<$A> {
                $name {
                    bounds: self.bounds,
                    $(
                        $field: self.$field.unwrap_or_else(|| {
                            $crate::widget!(@get_default $field $($default)?)
                        })
                    ),*
                    , _marker: std::marker::PhantomData,
                }
            }
        }

        impl<$A> $name<$A> {
            pub fn builder(bounds: $crate::Rect) -> Builder<$A> {
                Builder {
                    bounds,
                    $($field: None,)*
                    _marker: std::marker::PhantomData,
                }
            }
        }

        impl<$A: Copy> $crate::Widget<$A> for $name<$A> {
            fn render(&self, $prims: &mut Vec<$crate::Primitive<$A>>) {
                let $this = self;
                $body
            }
        }
    };

    (@get_default $field:ident $default:expr) => { $default };
    (@get_default $field:ident) => {
        panic!(concat!("Missing required field: ", stringify!($field)))
    };
}