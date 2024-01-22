#[macro_export]
macro_rules! format_err {
    ($($arg:tt)*) => {{
        use owo_colors::OwoColorize;
        format!("{} {}", "error:".red().bold(), format_args!($($arg)*))
    }};
}