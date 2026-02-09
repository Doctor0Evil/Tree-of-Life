#[macro_export]
macro_rules! neuroprint {
    ($input:expr) => {
        $crate::neuroprint_from_snapshot(&$input)
    };
}
