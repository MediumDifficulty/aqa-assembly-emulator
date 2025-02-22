#[macro_export]
macro_rules! unwrap_or_continue {
    ($opt:expr) => {
        match $opt {
            Some(v) => v,
            None => continue,
        }
    };
}

#[macro_export]
macro_rules! unwrap_or_return {
    ($opt:expr, $ret:expr) => {
        match $opt {
            Some(v) => v,
            None => return $ret,
        }
    };
}