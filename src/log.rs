macro_rules! info {
    ($($fmt: tt)*) => {
        println!("\x1b[1;32mInfo:\x1b[0m {}", format!($($fmt)*))
    };
}

macro_rules! warn {
    ($($fmt: tt)*) => {
        println!("\x1b[1;33mWarn:\x1b[0m {}", format!($($fmt)*))
    };
}

macro_rules! error {
    ($($fmt: tt)*) => {
        println!("\x1b[1;31mError:\x1b[0m {}", format!($($fmt)*))
    };
}

pub trait LoggedUnwrap {
    type Output;

    fn logged_unwrap(self) -> Self::Output;
}

impl<T> LoggedUnwrap for Option<T> {
    type Output = T;

    fn logged_unwrap(self) -> T {
        match self {
            Some(v) => v,
            None => {
                error!("called `Option::unwrap()` on a `None` value");
                std::process::exit(1);
            }
        }
    }
}

impl<T, E: std::fmt::Display> LoggedUnwrap for Result<T, E> {
    type Output = T;

    fn logged_unwrap(self) -> T {
        match self {
            Ok(v) => v,
            Err(err) => {
                error!("{err}");
                std::process::exit(1);
            }
        }
    }
}
