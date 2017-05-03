#[macro_use] extern crate runtime_fmt_derive;
extern crate runtime_fmt;

use runtime_fmt::PreparedFormat;

#[derive(FormatArgs)]
struct Struct {
    left: i32,
    right: &'static str,
}

#[derive(FormatArgs)]
struct TupleStruct(i32, &'static str);

#[derive(FormatArgs)]
struct UnitStruct;

#[derive(FormatArgs)]
struct Alignable {
    text: &'static str,
    width: usize,
}

#[derive(FormatArgs)]
struct WithBounds<'a, T: std::fmt::Display + 'a>(&'a T);

fn main() {
    let mut prepared = PreparedFormat::prepare("{left}: {right}").unwrap();
    prepared.newln();
    prepared.print(&Struct {
        left: 42,
        right: "Hello, world!"
    });

    PreparedFormat::prepare("{0}: {1}\n").unwrap().print(
        &TupleStruct(5, "Hello, TupleStruct")
    );

    PreparedFormat::prepare("Hello, UnitStruct\n").unwrap().print(&UnitStruct);

    let prepared = PreparedFormat::prepare("({text:^width$})\n").unwrap();
    prepared.print(&Alignable {
        text: "Wow, aligned!",
        width: 15
    });
    prepared.print(&Alignable {
        text: "Wow, aligned!",
        width: 20
    });

    PreparedFormat::prepare("{}").unwrap().newln().print(&WithBounds(&256));
}
