//! `rt_*` macros
//!
//! The central macro is `rt_format_args!`, analogous to `format_args!`.
//! The rest of the macros correspond to the other `std` formatting macros.

#[macro_export]
macro_rules! rt_format_args {
    (@[$spec:expr] [$($args:tt)*] $name:tt = $e:expr, $($rest:tt)*) => {
        rt_format_args!(@[$spec] [$($args)* $crate::Param::named(stringify!($name), &$e),] $($rest)*)
    };
    (@[$spec:expr] [$($args:tt)*] $name:tt = $e:expr) => {
        rt_format_args!(@[$spec] [$($args)* $crate::Param::named(stringify!($name), &$e),])
    };
    (@[$spec:expr] [$($args:tt)*] $e:expr, $($rest:tt)*) => {
        rt_format_args!(@[$spec] [$($args)* $crate::Param::normal(&$e),] $($rest)*)
    };
    (@[$spec:expr] [$($args:tt)*] $e:expr) => {
        rt_format_args!(@[$spec] [$($args)* $crate::Param::normal(&$e),])
    };
    (@[$spec:expr] [$($args:tt)*]) => {
        $crate::FormatBuf::new($spec, &[$($args)*])
    };
    ($spec:expr, $($rest:tt)*) => {
        rt_format_args!(@[$spec] [] $($rest)*)
    };
    ($spec:expr) => {
        $crate::FormatBuf::new($spec, &[])
    };
}

#[macro_export]
macro_rules! rt_format {
    ($($rest:tt)*) => {
        rt_format_args!($($rest)*).map(|x| x.with(::std::fmt::format))
    }
}

#[macro_export]
macro_rules! rt_print {
    ($($rest:tt)*) => {
        rt_format_args!($($rest)*).map(|x| x.with(::std::io::_print))
    }
}

#[macro_export]
macro_rules! rt_println {
    ($($rest:tt)*) => {
        rt_format_args!($($rest)*).map(|mut x| x.newln().with(::std::io::_print))
    }
}

#[macro_export]
macro_rules! rt_write {
    ($dest:expr, $($rest:tt)*) => {
        rt_format_args!($($rest)*).and_then(|x|
            x.with(|args| $dest.write_fmt(args)).map_err(::std::convert::From::from)
        )
    }
}

#[macro_export]
macro_rules! rt_writeln {
    ($dest:expr, $($rest:tt)*) => {
        rt_format_args!($($rest)*).and_then(|mut x|
            x.newln().with(|args| $dest.write_fmt(args)).map_err(::std::convert::From::from)
        )
    }
}
