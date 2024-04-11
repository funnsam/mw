macro_rules! info {
    ($fmt: tt $($args: tt)*) => {
        println!("\x1b[1;32mInfo:\x1b[0m {}", format!($($fmt)*))
    };
}

macro_rules! warn {
    ($($fmt: tt)*) => {
        println!("\x1b[1;33mWarn:\x1b[0m {}", format!($($fmt)*))
    };
}

macro_rules! error {
    ($fmt: tt $($args: tt)*) => {
        println!("\x1b[1;31mError:\x1b[0m {}", format!($($fmt)*))
    };
}
