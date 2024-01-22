#[macro_export]
macro_rules! format_err {
    ($($arg:tt)*) => {{
        use owo_colors::OwoColorize;
        format!("{} {}", "error:".red().bold(), format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! unwrap_or_display_err {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => {
                println!("{}", format_err!("{e}"));
                return;
            }
        }
    };
}