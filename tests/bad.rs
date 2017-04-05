#[macro_use] extern crate runtime_fmt;

use runtime_fmt::Error::*;

macro_rules! err_with {
    ($err:expr; $($rest:tt)*) => {
        assert_eq!(
            format!("Err({:?})", $err),
            format!("{:?}", rt_format!($($rest)*))
        )
    }
}

macro_rules! err_any {
    ($($rest:tt)*) => {
        assert!(rt_format!($($rest)*).is_err())
    }
}

#[test]
fn bad_index() {
    err_with!(BadIndex(0); "{}");
    err_with!(BadIndex(7); "{7}");
    err_with!(BadIndex(2); "{} {} {}", "", "");
}

#[test]
fn bad_usize() {
    err_with!(BadCount(0); "{:.*}", "Not A Usize", "aaaa");
}

#[test]
fn bad_syntax() {
    err_any!("{-1}");
}

#[test]
fn bad_format() {
    struct Foo;

    err_with!(NoSuchFormat("q"); "{:q}", "");
    err_with!(UnsatisfiedFormat { idx: 0, must_implement: "Debug" };
        "{:?}", Foo);
}
