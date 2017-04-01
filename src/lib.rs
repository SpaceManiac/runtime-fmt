//!
#![feature(fmt_internals)]
#![feature(specialization)]
#![feature(unicode)]

// feature(print) seems to be incorrectly linted as unused
#![allow(unused_features)]
#![feature(print)]

#[macro_use] mod macros;
mod erase;
// fmt_macros.rs is from rust/src/libfmt_macros/lib.rs
mod fmt_macros;

use std::fmt::{Arguments, ArgumentV1};
use std::fmt::rt::v1;
use std::borrow::Cow;

#[derive(Debug)]
pub enum FormatError<'a> {
    BadIndex(usize),
    BadName(&'a str),
    NoSuchFormat(&'a str),
    UnsatisfiedFormat(&'a str),
    Io(std::io::Error),
    Fmt(std::fmt::Error),
}

impl<'a> From<std::io::Error> for FormatError<'a> {
    fn from(e: std::io::Error) -> Self {
        FormatError::Io(e)
    }
}

impl<'a> From<std::fmt::Error> for FormatError<'a> {
    fn from(e: std::fmt::Error) -> Self {
        FormatError::Fmt(e)
    }
}

pub struct Param<'a> {
    name: Option<&'static str>,
    value: &'a erase::Format,
}

impl<'a> Param<'a> {
    pub fn normal<T>(t: &'a T) -> Param<'a> {
        Param {
            name: None,
            value: erase::erase(t),
        }
    }

    pub fn named<T>(name: &'static str, t: &'a T) -> Param<'a> {
        Param {
            name: Some(name),
            value: erase::erase(t),
        }
    }
}

pub struct FormatBuf<'s> {
    pieces: Vec<Cow<'s, str>>,
    args: Vec<ArgumentV1<'s>>,
    fmt: Option<Vec<v1::Argument>>,
}

impl<'s> FormatBuf<'s> {
    #[inline]
    pub fn new(spec: &'s str, params: &'s [Param<'s>]) -> Result<Self, FormatError<'s>> {
        parse(spec, params)
    }

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

    pub fn with<F: FnOnce(Arguments) -> R, R>(&self, f: F) -> R {
        let pieces: Vec<&str> = self.pieces.iter().map(|r| &**r).collect();
        f(match self.fmt {
            Some(ref fmt) => Arguments::new_v1_formatted(&pieces, &self.args, fmt),
            None => Arguments::new_v1(&pieces, &self.args),
        })
    }
}

fn parse<'s>(spec: &'s str, params: &'s [Param<'s>]) -> Result<FormatBuf<'s>, FormatError<'s>> {
    use fmt_macros as p;

    let mut pieces = Vec::new();
    let mut args = Vec::new();
    let mut fmt = Vec::new();

    let mut str_accum: Cow<str> = "".into();
    for piece in p::Parser::new(spec) {
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
                let convert_count = |c| -> Result<v1::Count, FormatError<'s>> {
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
                    return Err(FormatError::BadIndex(idx))
                }
                let argument = params[idx].value.by_name(arg.format.ty)?;

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

fn lookup<'s, 'n>(params: &'s [Param<'s>], name: &'n str) -> Result<usize, FormatError<'n>> {
    if let Some(idx) = params.iter().position(|p| {
        p.name.map_or(false, |n| n == name)
    }) {
        Ok(idx)
    } else {
        Err(FormatError::BadName(name))
    }
}
