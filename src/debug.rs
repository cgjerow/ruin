use std::fmt;

pub struct Debug {
    pub enabled: bool,
}

impl Debug {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn log_fmt(&self, args: fmt::Arguments) {
        if self.enabled {
            println!("{}", args);
        }
    }
}

#[macro_export]
macro_rules! debug_log {
    ($dbg:expr, $($arg:tt)*) => {
        $dbg.log_fmt(format_args!($($arg)*))
    };
}
