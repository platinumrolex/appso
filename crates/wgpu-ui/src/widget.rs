 use crate::primitives::{HitRegion, Primitive, Rect};
// 
 pub trait Widget<A> {
     fn render(&self, prims: &mut Vec<Primitive>, hits: &mut Vec<HitRegion<A>>);
 }
// 
// #[macro_export]
// macro_rules! widget {
//     (
//         $(#[$meta:meta])*
//         $vis:vis $name:ident<$A:ident> {
//             $($field:ident : $ty:ty $(= $default:expr)?),* $(,)?
//         }
//         render: |$this:pat, $prims:ident, $hits:ident| $body:block
//     ) => {
//         $(#[$meta])*
//         $vis struct $name<$A> {
//             pub bounds: $crate::Rect,
//             $(pub $field : $ty),*
//         }
// 
//         // The Builder handles optional fields and allows order-independent DSL
//         $vis struct Builder<$A> {
//             pub bounds: $crate::Rect,
//             $(pub $field: Option<$ty>),*
//         }
// 
//         impl<$A> Builder<$A> {
//             $(
//                 pub fn $field(mut self, val: $ty) -> Self {
//                     self.$field = Some(val);
//                     self
//                 }
//             )*
// 
//             pub fn build(self) -> $name<$A> {
//                 $name {
//                     bounds: self.bounds,
//                     $(
//                         $field: self.$field.unwrap_or_else(|| {
//                             $crate::widget!(@get_default $field $($default)?)
//                         })
//                     ),*
//                 }
//             }
//         }
// 
//         impl<$A> $name<$A> {
//             pub fn builder(bounds: $crate::Rect) -> Builder<$A> {
//                 Builder {
//                     bounds,
//                     $($field: None),*
//                 }
//             }
//         }
// 
//         impl<$A: Copy> $crate::Widget<$A> for $name<$A> {
//             fn render(&self, $prims: &mut Vec<$crate::Primitive>, $hits: &mut Vec<$crate::HitRegion<$A>>) {
//                 let $this = self;
//                 $body
//             }
//         }
//     };
// 
//     (@get_default $field:ident $default:expr) => { $default };
//     (@get_default $field:ident) => { 
//         panic!(concat!("Missing required field in DSL: ", stringify!($field))) 
//     };
// }

// crates/wgpu-ui/src/widget.rs
use std::marker::PhantomData; // Add this import

#[macro_export]
macro_rules! widget {
    (
        $(#[$meta:meta])*
        $vis:vis $name:ident<$A:ident> {
            $($field:ident : $ty:ty $(= $default:expr)?),* $(,)?
        }
        render: |$this:pat, $prims:ident, $hits:ident| $body:block
    ) => {
        $(#[$meta])*
        $vis struct $name<$A> {
            pub bounds: $crate::Rect,
            $(pub $field : $ty,)*
            _marker: std::marker::PhantomData<$A>, // Automatically handle <A>
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
            fn render(&self, $prims: &mut Vec<$crate::Primitive>, $hits: &mut Vec<$crate::HitRegion<$A>>) {
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