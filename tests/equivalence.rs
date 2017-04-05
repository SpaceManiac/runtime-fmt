#[macro_use] extern crate runtime_fmt;

macro_rules! case {
    ($($rest:tt)*) => {{
        let string = format!($($rest)*);
        println!("{}", string);
        assert_eq!(
            string,
            rt_format!($($rest)*).unwrap()
        )
    }}
}

#[test]
fn simple_equivalence() {
    case!("Literal string!");
    case!("Hello, {}", "world");
    case!("Hello, {}!", "world");
    case!("2 + 2 = {}", 2 + 2);
    case!("{0:?} {0}", "A \\ B");
    case!("{} {x}", x="Foo");
    case!("{x} {} {}", "Foo", x="Bar");
    case!("{x} {x} {}", "Foo", x="Bar");
    case!("{x} {} {x}", "Foo", x="Bar");
    case!("{} {x} {}", "Foo", x="Bar");
    case!("{} {x} {x}", "Foo", x="Bar");
    case!("{} {} {x}", "Foo", x="Bar");
    case!("{:x}", 0x3feebb77);
    case!("{:X}", 0x3feebb77);
    case!("Hex: {:.>4x}", 17);
    case!("{:p}", "Hello");
    case!("{}{}{}", "(A)", "_ _", "(B)");
}