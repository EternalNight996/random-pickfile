//! utils的标准库
//! 😵 hello tuils 🐢
//!

/// # Example
/// ``` no_run
/// fn main() {
/// output!("hello world");
/// 输出 [>] 😵 hello world 🐢
///
/// output!(1;2;34; 5);
/// 输出 [>] 😵 hello world 🐢
///
/// let list = [1,2,34,5];
/// 输出 [>] 😵 12345 🐢
///
/// output!("{:#?}",list);
/// 输出 [>] 😵 [
/// 1,
/// 2,
/// 34,
/// 5,
/// ] 🐢
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
