//! Type erasure for formattable types.
use std::fmt;
use Error;

type Func<T> = fn(&T, &mut fmt::Formatter) -> fmt::Result;

macro_rules! traits {
    ($($string:pat, $upper:ident, $lower:ident;)*) => {
        $(
            trait $upper {
                fn $lower(&self) -> Option<Func<Self>>;
            }
            impl<T> $upper for T {
                #[inline]
                default fn $lower(&self) -> Option<Func<Self>> { None }
            }
            impl<T: fmt::$upper> $upper for T {
                #[inline]
                fn $lower(&self) -> Option<Func<Self>> {
                    Some(<Self as fmt::$upper>::fmt)
                }
            }
        )*

        pub trait Format {
            fn by_name<'n>(&self, name: &'n str) -> Result<fmt::ArgumentV1, Error<'n>>;
        }

        impl<T> Format for T {
            fn by_name<'n>(&self, name: &'n str) -> Result<fmt::ArgumentV1, Error<'n>> {
                match name {
                    $(
                        $string => match $upper::$lower(self) {
                            Some(f) => Ok(fmt::ArgumentV1::new(self, f)),
                            None => Err(Error::UnsatisfiedFormat(name)),
                        },
                    )*
                    _ => Err(Error::NoSuchFormat(name)),
                }
            }
        }
    }
}

traits! {
    "", Display, display;
    "?", Debug, debug;
    "e", LowerExp, lower_exp;
    "E", UpperExp, upper_exp;
    "o", Octal, octal;
    "p", Pointer, pointer;
    "b", Binary, binary;
    "x", LowerHex, lower_hex;
    "X", UpperHex, upper_hex;
}
