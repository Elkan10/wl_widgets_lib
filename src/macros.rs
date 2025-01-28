#[macro_export]
macro_rules! wrap {
    ($name:ident, $wraps:ty) => {
        pub struct $name {
            w: $wraps
        }
        impl Into<$wraps> for $name {
            fn into(self) -> $wraps {
                self.w
            }
        }
    };
}
