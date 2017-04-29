#[macro_use] extern crate runtime_fmt;

macro_rules! t {
    ($be:expr; $($rest:tt)*) => {{
        assert_eq!(
            $be,
            rt_format!($($rest)*).unwrap()
        )
    }}
}

#[test]
fn non_usize_pad() {
    t!("aaaa"; "{:.*}", 4u8, "aaaaaaaa");
    t!("aaaa"; "{:.*}", 4u16, "aaaaaaaa");
    t!("aaaa"; "{:.*}", 4u32, "aaaaaaaa");
    t!("aaaa"; "{:.*}", 4u64, "aaaaaaaa");
}

#[test]
fn formatted_format_str() {
    let format = format!("{}{}{}", "Hello, ", "{}", "!");
    // nb: `format` is not `&`d here, so this tests that the macro does it
    // generally, anything Deref<Target=str> should be usable
    t!("Hello, world!"; format, "world");
    drop(format);
}

#[test]
fn unused_is_fine() {
    t!("1"; rt_format!("{a}", a=1, b=2).unwrap());
    t!("2"; rt_format!("{b}", a=1, b=2).unwrap());
    t!("3 1"; rt_format!("{} {a}", 3, 4, a=1, b=2).unwrap());
}
