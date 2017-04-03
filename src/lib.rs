//! A crate for string formatting using runtime format strings.
//!
//! This crate provides much the same facilities as `std::fmt`, with the
//! additional allowance for format strings which are not known until runtime.
//! Possible applications include internationalization, scripting, or other
//! customization.
//!
//! Each of the standard formatting macros `format_args!`, `format!`,
//! `print!`, `println!`, `write!`, and `writeln!` have corresponding `rt_`
//! variants. Calls which previously succeeded unconditionally now return
//! `Result`, which may indicate a bad format string or arguments.
//!
//! The syntax for format strings and for macro invocations is equivalent to
//! that used by `std::fmt`, including support for positional and named
//! arguments. This crate shells out to the standard library implementations
//! for as much as possible to ensure feature parity.
#![feature(fmt_internals)]
#![feature(specialization)]
#![feature(unicode)]
#![feature(print)]

#[doc(hidden)]
#[inline]
pub fn _print(args: Arguments) {
    std::io::_print(args)
}

#[macro_use] mod macros;
mod erase;

// fmt_macros.rs is from rust/src/libfmt_macros/lib.rs
// copy-pasted rather than externed to avoid dynamically linking libstd
mod fmt_macros;

use std::fmt::{Arguments, ArgumentV1};
use std::fmt::rt::v1;
use std::borrow::Cow;

/// An error during parsing or formatting.
#[derive(Debug)]
pub enum Error<'a> {
    /// Invalid format string syntax.
    BadSyntax(Vec<(String, Option<String>)>),
    /// A format specifier referred to an out-of-range index.
    BadIndex(usize),
    /// A format specifier referred to a non-existent name.
    BadName(&'a str),
    /// A format specifier referred to a non-existent type.
    NoSuchFormat(&'a str),
    /// A format specifier's type was not satisfied by its argument.
    UnsatisfiedFormat {
        idx: usize,
        must_implement: &'static str,
    },
    /// An I/O error from an `rt_write!` or `rt_writeln!` call.
    Io(std::io::Error),
    /// A formatting error from an `rt_write!` or `rt_writeln!` call.
    Fmt(std::fmt::Error),
}

impl<'a> From<std::io::Error> for Error<'a> {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl<'a> From<std::fmt::Error> for Error<'a> {
    fn from(e: std::fmt::Error) -> Self {
        Error::Fmt(e)
    }
}

/// A type-erased parameter, with an optional name.
pub struct Param<'a> {
    name: Option<&'static str>,
    value: &'a erase::Format,
}

impl<'a> Param<'a> {
    /// Create a nameless parameter from the given value.
    pub fn normal<T>(t: &'a T) -> Param<'a> {
        Param {
            name: None,
            value: t,
        }
    }

    /// Create a named parameter from the given value.
    pub fn named<T>(name: &'static str, t: &'a T) -> Param<'a> {
        Param {
            name: Some(name),
            value: t,
        }
    }
}

/// A buffer representing a parsed format string and arguments.
pub struct FormatBuf<'s> {
    pieces: Vec<Cow<'s, str>>,
    args: Vec<ArgumentV1<'s>>,
    fmt: Option<Vec<v1::Argument>>,
}

impl<'s> FormatBuf<'s> {
    /// Construct a new buffer from the given format string and arguments.
    ///
    /// This method should usually not be called directly. Instead use the
    /// `rt_format_args!` macro.
    #[inline]
    pub fn new(spec: &'s str, params: &'s [Param<'s>]) -> Result<Self, Error<'s>> {
        let mut parser = fmt_macros::Parser::new(spec);
        let result = parse(&mut parser, params);
        if parser.errors.is_empty() {
            result
        } else {
            Err(Error::BadSyntax(parser.errors))
        }
    }

    /// Append a linefeed (`\n`) to the end of this buffer.
    pub fn newln(&mut self) -> &mut Self {
        // If fmt is None, the number of implicit formatting specifiers
        // is the same as the number of arguments.
        let len = self.fmt.as_ref().map_or(self.args.len(), |fmt| fmt.len());
        if self.pieces.len() > len {
            // The final piece is after the final formatting specifier, so
            // it's okay to just add to the end of it.
            self.pieces.last_mut().unwrap().to_mut().push_str("\n")
        } else {
            // The final piece is before the final formatting specifier, so
            // a new piece needs to be added at the end.
            self.pieces.push("\n".into())
        }
        self
    }

    /// Call a function accepting `Arguments` with the contents of this buffer.
    pub fn with<F: FnOnce(Arguments) -> R, R>(&self, f: F) -> R {
        let pieces: Vec<&str> = self.pieces.iter().map(|r| &**r).collect();
        f(match self.fmt {
            Some(ref fmt) => Arguments::new_v1_formatted(&pieces, &self.args, fmt),
            None => Arguments::new_v1(&pieces, &self.args),
        })
    }
}

fn parse<'s>(parser: &mut fmt_macros::Parser<'s>, params: &'s [Param<'s>])
    -> Result<FormatBuf<'s>, Error<'s>>
{
    use fmt_macros as p;

    let mut pieces = Vec::new();
    let mut args = Vec::new();
    let mut fmt = Vec::new();

    let mut str_accum: Cow<str> = "".into();
    while let Some(piece) = parser.next() {
        match piece {
            p::Piece::String(text) => {
                // append string to accumulator
                if str_accum.is_empty() {
                    str_accum = text.into();
                } else if !text.is_empty() {
                    str_accum.to_mut().push_str(text);
                }
            }
            p::Piece::NextArgument(arg) => {
                // flush accumulator always
                pieces.push(std::mem::replace(&mut str_accum, "".into()));

                // determine the index of the argument in question
                let idx = match arg.position {
                    p::Position::ArgumentIs(idx) => idx,
                    p::Position::ArgumentNamed(name) => lookup(params, name)?,
                };

                // convert the format spec
                let convert_count = |c| -> Result<v1::Count, Error<'s>> {
                    Ok(match c {
                        p::CountIs(val) => v1::Count::Is(val),
                        p::CountIsName(name) => v1::Count::Param(lookup(params, name)?),
                        p::CountIsParam(idx) => v1::Count::Param(idx),
                        p::CountImplied => v1::Count::Implied,
                    })
                };
                let spec = v1::FormatSpec {
                    fill: arg.format.fill.unwrap_or(' '),
                    flags: arg.format.flags,
                    align: match arg.format.align {
                        p::AlignLeft => v1::Alignment::Left,
                        p::AlignRight => v1::Alignment::Right,
                        p::AlignCenter => v1::Alignment::Center,
                        p::AlignUnknown => v1::Alignment::Unknown,
                    },
                    precision: convert_count(arg.format.precision)?,
                    width: convert_count(arg.format.width)?,
                };

                // convert the argument
                if idx >= params.len() {
                    return Err(Error::BadIndex(idx))
                }
                let argument = params[idx].value.by_name(arg.format.ty, idx)?;

                // push the format spec and argument value
                fmt.push(v1::Argument {
                    position: v1::Position::At(args.len()),
                    format: spec,
                });
                args.push(argument);

                // TODO: let fmt be none if all fmts are default.
                // TODO: for params which appear multiple times in the format
                // string, only add them to the args list once.
            }
        }
    }
    // flush accumulator if needed
    if !str_accum.is_empty() {
        pieces.push(str_accum);
    }

    Ok(FormatBuf {
        pieces: pieces,
        args: args,
        fmt: Some(fmt),
    })
}

fn lookup<'s, 'n>(params: &'s [Param<'s>], name: &'n str) -> Result<usize, Error<'n>> {
    if let Some(idx) = params.iter().position(|p| {
        p.name.map_or(false, |n| n == name)
    }) {
        Ok(idx)
    } else {
        Err(Error::BadName(name))
    }
}
