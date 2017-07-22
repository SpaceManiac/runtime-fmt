runtime-fmt [![](https://meritbadge.herokuapp.com/runtime-fmt)](https://crates.io/crates/runtime-fmt) [![](https://img.shields.io/badge/docs-online-2020ff.svg)](https://docs.rs/runtime-fmt)
==========

A crate for string formatting using runtime format strings.

This crate provides much the same facilities as `std::fmt`, with the
additional allowance for format strings which are not known until runtime.
Possible applications include internationalization, scripting, or other
customization.

Each of the standard formatting macros `format_args!`, `format!`,
`print!`, `println!`, `write!`, and `writeln!` have corresponding `rt_`
variants. Calls which previously succeeded unconditionally now return
`Result`, which may indicate a bad format string or arguments.

The syntax for format strings and for macro invocations is equivalent to
that used by `std::fmt`, including support for positional and named
arguments. This crate shells out to the standard library implementations
for as much as possible to ensure feature parity.

This crate makes extensive use of the unstable formatting machinery and
therefore **requires nightly**.
