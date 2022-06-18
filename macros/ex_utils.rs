#[allow(unused)]
macro_rules! trace { ($($x:tt)*) => (::log::trace!($($x)*)) }
#[allow(unused)]
macro_rules! debug { ($($x:tt)*) => (::log::debug!($($x)*)) }
#[allow(unused)]
macro_rules! info { ($($x:tt)*) => (::log::info!($($x)*)) }
#[allow(unused)]
macro_rules! warn { ($($x:tt)*) => (::log::warn!($($x)*)) }
#[allow(unused)]
macro_rules! error { ($($x:tt)*) => (::log::error!($($x)*)) }

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
#[allow(unused)]
macro_rules! output {
    () => {print!("\n")};
    ($fmt:expr) => {::p_utils::output!($fmt)};
    (pure $fmt:expr) => {::p_utils::output!(pure $fmt)};
    (pure $($arg:tt)*) => {{::p_utils::output!(pure $($arg)*)}};
    ($($args:tt);*) => {{::p_utils::output!($($args);*)}};
    ($($args:tt)*) => {{::p_utils::output!($($args)*)}};
}