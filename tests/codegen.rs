#[macro_use] extern crate runtime_fmt_derive;
extern crate runtime_fmt;

use runtime_fmt::{FormatArgs, PreparedFormat};

#[test]
#[allow(dead_code)]
fn test_struct() {
    #[derive(FormatArgs)]
    struct Struct {
        #[format_args(aliases = "alias1,alias2")]
        field1: &'static str,
        field2: &'static str,
        field3: usize,
        #[format_args(rename = "renamed")]
        field4: &'static str,
        #[format_args(ignore)]
        ignored: &'static str
    }

    assert_eq!(Struct::validate_name("field1").is_some(), true);
    assert_eq!(Struct::validate_name("field1"), Struct::validate_name("alias1"));
    assert_eq!(Struct::validate_name("field1"), Struct::validate_name("alias2"));

    assert_eq!(Struct::validate_name("field2").is_some(), true);
    assert_eq!(Struct::validate_name("field3").is_some(), true);

    assert_eq!(Struct::validate_name("field4").is_some(), false);
    assert_eq!(Struct::validate_name("renamed").is_some(), true);

    assert_eq!(Struct::validate_name("ignored").is_some(), false);

    assert_eq!(Struct::validate_index(0), true);
    assert_eq!(Struct::validate_index(1), true);
    assert_eq!(Struct::validate_index(2), true);
    assert_eq!(Struct::validate_index(3), true);
    assert_eq!(Struct::validate_index(4), false);

    assert_eq!(Struct::as_usize(0).is_some(), false);
    assert_eq!(Struct::as_usize(1).is_some(), false);
    assert_eq!(Struct::as_usize(2).is_some(), true);
    assert_eq!(Struct::as_usize(3).is_some(), false);

    let value = Struct {
        field1: "value1",
        field2: "value2",
        field3: 123456,
        field4: "value4",
        ignored: "ignored"
    };

    let fmt = PreparedFormat::prepare("{field1} {alias1} {alias2} {field2} {field3} {renamed}").unwrap().format(&value);
    assert_eq!(fmt, "value1 value1 value1 value2 123456 value4");

    let fmt = PreparedFormat::prepare("{0} {1} {2} {3}").unwrap().format(&value);
    assert_eq!(fmt, "value1 value2 123456 value4");
}

#[test]
fn test_tuple() {
    #[derive(FormatArgs)]
    struct Tuple(
        #[format_args(aliases = "alias1,alias2")]
        &'static str,
        &'static str,
        usize,
        #[format_args(rename = "renamed")]
        &'static str,
        #[format_args(ignore)]
        &'static str
    );

    assert_eq!(Tuple::validate_name("alias1").is_some(), true);
    assert_eq!(Tuple::validate_name("alias1"), Tuple::validate_name("alias2"));
    assert_eq!(Tuple::validate_name("renamed").is_some(), true);

    assert_eq!(Tuple::validate_index(0), true);
    assert_eq!(Tuple::validate_index(1), true);
    assert_eq!(Tuple::validate_index(2), true);
    assert_eq!(Tuple::validate_index(3), true);
    assert_eq!(Tuple::validate_index(4), false);

    assert_eq!(Tuple::as_usize(0).is_some(), false);
    assert_eq!(Tuple::as_usize(1).is_some(), false);
    assert_eq!(Tuple::as_usize(2).is_some(), true);
    assert_eq!(Tuple::as_usize(3).is_some(), false);

    let value = Tuple("value1", "value2", 123456, "value3", "ignored");

    let fmt = PreparedFormat::prepare("{alias1} {alias2} {renamed}").unwrap().format(&value);
    assert_eq!(fmt, "value1 value1 value3");

    let fmt = PreparedFormat::prepare("{0} {1} {2} {3}").unwrap().format(&value);
    assert_eq!(fmt, "value1 value2 123456 value3");
}

#[test]
fn test_unit() {
    #[derive(FormatArgs)]
    struct Unit;

    assert_eq!(Unit::validate_index(0), false);

    let fmt = PreparedFormat::prepare("").unwrap().format(&Unit);
    assert_eq!(fmt, "");
}