#[macro_use] extern crate runtime_fmt_derive;
extern crate runtime_fmt;

#[derive(FormatArgs)]
struct Args {
    left: i32,
    right: &'static str,
}

fn main() {
    let prepared = runtime_fmt::prepare::<Args>("{left}: {right}").unwrap();
    let formatted = prepared.format_args(&Args {
        left: 42,
        right: "Hello, world!"
    }).to_string();
    println!("{}", formatted);
}
