#[macro_export]
macro_rules! reexport {
    ($t: ty, $feature: literal) => {
        #[cfg(feature = $feature)]
        pub use $t;
        #[cfg(not(feature = $feature))]
        use $t;
    };
}
