//! utils็ๆ ๅๅบ
//! ๐ต hello tuils ๐ข
//!

/// # Example
/// ``` no_run
/// fn main() {
/// output!("hello world");
/// ่พๅบ [>] ๐ต hello world ๐ข
///
/// output!(1;2;34; 5);
/// ่พๅบ [>] ๐ต hello world ๐ข
///
/// let list = [1,2,34,5];
/// ่พๅบ [>] ๐ต 12345 ๐ข
///
/// output!("{:#?}",list);
/// ่พๅบ [>] ๐ต [
/// 1,
/// 2,
/// 34,
/// 5,
/// ] ๐ข
/// }
/// ```
#[macro_export]
#[doc(hidden)]
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
macro_rules! output {
    () => {print!("\n")};
    ($fmt:expr) => {::std::eprintln!("{}", $crate::rgb_format!($fmt))};
    (pure $fmt:expr) => {$crate::rgb_format!(pure $fmt)};
    (pure $($arg:tt)*) => {{$crate::rgb_format!(pure $($arg)*)}};
    ($($args:tt);*) => {{::std::eprintln!("{}", $crate::rgb_format!(::std::concat!($($args),*)))}};
    ($($args:tt)*) => {{::std::eprintln!("{}", $crate::rgb_format!($($args)*))}};
} 
