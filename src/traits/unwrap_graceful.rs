use std::{
    fmt::{Debug, Display},
    process,
};
use colored::Colorize;

pub trait UnwrapGraceful<V, E>
where
    E: Debug,
{
    fn unwrap_graceful<Fn, T: Display>(self, f: Fn) -> V
    where
        Fn: FnOnce(E) -> T;
}

impl<V, E> UnwrapGraceful<V, E> for Result<V, E>
where
    E: Debug,
{
    fn unwrap_graceful<Fn, T: Display>(self, f: Fn) -> V
    where
        Fn: FnOnce(E) -> T,
    {
        match self {
            Ok(val) => val,
            Err(err) => {
                eprintln!("{} {}", "error:".bright_red(), f(err));
                process::exit(1);
            }
        }
    }
}
