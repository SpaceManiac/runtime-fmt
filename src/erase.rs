//! Type erasure for formattable types.
use std::fmt;
use std::convert::TryFrom;
use Error;

type Func<T> = fn(&T, &mut fmt::Formatter) -> fmt::Result;

trait AsUsize {
    fn as_usize(&self) -> Option<usize>;
}
impl<T> AsUsize for T {
    #[inline]
    default fn as_usize(&self) -> Option<usize> { None }
}
impl<T> AsUsize for T where usize: TryFrom<T>, T: Copy {
    #[inline]
    fn as_usize(&self) -> Option<usize> {
        usize::try_from(*self).ok()
    }
}

macro_rules! traits {
    ($($string:pat, $upper:ident, $lower:ident;)*) => {
        $(
            trait $upper {
                fn $lower() -> Option<Func<Self>>;
            }
            impl<T> $upper for T {
                #[inline]
                default fn $lower() -> Option<Func<Self>> { None }
            }
            impl<T: fmt::$upper> $upper for T {
                #[inline]
                fn $lower() -> Option<Func<Self>> {
                    Some(<Self as fmt::$upper>::fmt)
                }
            }
        )*

        pub trait Format {
            fn as_usize(&self) -> Option<usize>;
            fn by_name<'n>(&self, name: &'n str, idx: usize) -> Result<fmt::ArgumentV1, Error<'n>>;
        }

        impl<T> Format for T {
            #[inline]
            fn as_usize(&self) -> Option<usize> {
                AsUsize::as_usize(self)
            }
            fn by_name<'n>(&self, name: &'n str, idx: usize) -> Result<fmt::ArgumentV1, Error<'n>> {
                match name {
                    $(
                        $string => match <Self as $upper>::$lower() {
                            Some(f) => Ok(fmt::ArgumentV1::new(self, f)),
                            None => Err(Error::UnsatisfiedFormat {
                                idx: idx,
                                must_implement: stringify!($upper),
                            }),
                        },
                    )*
                    _ => Err(Error::NoSuchFormat(name)),
                }
            }
        }

        pub fn codegen_get_child<'n, T: ::FormatArgs>(name: &'n str, idx: usize)
            -> Result<fn(&T, &mut fmt::Formatter) -> fmt::Result, Error>
        {
            match name {
                $(
                    $string => match T::get_child::<dyn fmt::$upper>(idx) {
                        Some(f) => Ok(f),
                        None => Err(Error::UnsatisfiedFormat {
                            idx: idx,
                            must_implement: stringify!($upper),
                        })
                    },
                )*
                _ => Err(Error::NoSuchFormat(name)),
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
