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
}
