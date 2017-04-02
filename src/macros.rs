//! `rt_*` macros
//!
//! The central macro is `rt_format_args!`, analogous to `format_args!`.
//! The rest of the macros correspond to the other `std` formatting macros.

/// The core macro for runtime formatting.
///
/// This macro produces a value of type `Result<FormatBuf, Error>`. Invalid
/// format strings are indicated by an error result. The resulting value can
/// be converted to a `std::fmt::Arguments` via the `with()` method.
///
/// The syntax accepted is the same as `format_args!`. See the module-level
/// docs for more detail.
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

/// Format a value of type `String` with a runtime format string.
///
/// Returns a `Result<String, Error>`. See the module-level docs for more
/// information.
#[macro_export]
macro_rules! rt_format {
    ($($rest:tt)*) => {
        rt_format_args!($($rest)*).map(|x| x.with(::std::fmt::format))
    }
}

/// Print to standard output with a runtime format string.
///
/// Returns a `Result<(), Error>`. Panics if writing to stdout fails. See the
/// module-level docs for more information.
#[macro_export]
macro_rules! rt_print {
    ($($rest:tt)*) => {
        rt_format_args!($($rest)*).map(|x| x.with($crate::_print))
    }
}

/// Print to standard output with a runtime format string and trailing newline.
///
/// Returns a `Result<(), Error>`. Panics if writing to stdout fails. See the
/// module-level docs for more information.
#[macro_export]
macro_rules! rt_println {
    ($($rest:tt)*) => {
        rt_format_args!($($rest)*).map(|mut x| x.newln().with($crate::_print))
    }
}

/// Write runtime-formatted data into a buffer.
///
/// Like `write!`, implementations of either `std::fmt::Write` or
/// `std::io::Write` are accepted. `Error` variants of the appropriate type may
/// be returned.
///
/// Returns a `Result<(), Error>`. See the module-level docs for more
/// information.
#[macro_export]
macro_rules! rt_write {
    ($dest:expr, $($rest:tt)*) => {
        rt_format_args!($($rest)*).and_then(|x|
            x.with(|args| $dest.write_fmt(args)).map_err(::std::convert::From::from)
        )
    }
}

/// Write runtime-formatted data into a buffer with a trailing newline.
///
/// Like `writeln!`, implementations of either `std::fmt::Write` or
/// `std::io::Write` are accepted. `Error` variants of the appropriate type may
/// be returned.
///
/// Returns a `Result<(), Error>`. See the module-level docs for more
/// information.
#[macro_export]
macro_rules! rt_writeln {
    ($dest:expr, $($rest:tt)*) => {
        rt_format_args!($($rest)*).and_then(|mut x|
            x.newln().with(|args| $dest.write_fmt(args)).map_err(::std::convert::From::from)
        )
    }
}
