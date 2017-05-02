//! Support for the codegen module.
#![doc(hidden)]

use std::mem::{size_of, zeroed};
use std::fmt::*;

/// Implementors correspond to formatting traits which may apply to values.
pub trait FormatTrait {
    /// Return whether this format trait is applicable to a type.
    fn allowed<T>() -> bool;
    /// Format a value of the given trait using this format trait.
    /// Must panic if `allowed::<T>()` is false.
    fn perform<T>(t: &T, f: &mut Formatter) -> Result;
}

// Abuse specialization to provide the `FormatTrait` impl for the actual
// format traits without requiring HKT or other deep chicanery.
trait Specialized<T> {
    fn allowed() -> bool;
    fn perform(t: &T, f: &mut Formatter) -> Result;
}

macro_rules! impl_format_trait {
    ($($name:ident,)*) => {
        $(
            impl<T> Specialized<T> for $name {
                default fn allowed() -> bool { false }
                default fn perform(_: &T, _: &mut Formatter) -> Result {
                    panic!()
                }
            }

            impl<T: $name> Specialized<T> for $name {
                fn allowed() -> bool { true }
                fn perform(t: &T, f: &mut Formatter) -> Result {
                    t.fmt(f)
                }
            }

            impl FormatTrait for $name {
                fn allowed<T>() -> bool { <Self as Specialized<T>>::allowed() }
                fn perform<T>(t: &T, f: &mut Formatter) -> Result {
                    <Self as Specialized<T>>::perform(t, f)
                }
            }
        )*
    }
}

impl_format_trait! {
    Display, Debug, LowerExp, UpperExp, Octal, Pointer, Binary, LowerHex,
    UpperHex,
}

fn get_formatter<T, F: FormatTrait + ?Sized>()
    -> Option<impl Fn(&T, &mut Formatter) -> Result>
{
    if F::allowed::<T>() {
        Some(F::perform::<T>)
    } else {
        None
    }
}

// The combined function which will be returned by `make_combined`.
fn combined<A, B, LHS, RHS>(a: &A, f: &mut Formatter) -> Result
    where LHS: Fn(&A) -> &B, RHS: Fn(&B, &mut Formatter) -> Result
{
    let lhs = unsafe { zeroed::<LHS>() };
    let rhs = unsafe { zeroed::<RHS>() };
    rhs(lhs(a), f)
}

// Local type alias for the formatting function pointer type.
type FormatFn<T> = fn(&T, &mut Formatter) -> Result;

// Accepts dummy arguments to allow type parameter inference, and returns
// `combined` instantiated with those arguments.
fn make_combined<A, B, LHS, RHS>(_: LHS, _: RHS) -> FormatFn<A>
    where LHS: Fn(&A) -> &B, RHS: Fn(&B, &mut Formatter) -> Result
{
    // check that both function
    assert!(size_of::<LHS>() == 0,
        "Mapper from parent to child must be zero-sized, instead size was {}",
        size_of::<LHS>());
    assert!(size_of::<RHS>() == 0,
        "Formatting function must be zero-sized, instead size was {}",
        size_of::<RHS>());
    combined::<A, B, LHS, RHS>
}

/// Combine a function from `&A` to `&B` and a formatting trait applicable to
/// `B` and return a function pointer which will convert a `&A` to `&B` and
/// then format it with the given trait.
///
/// Returns `None` if the formatting trait is not applicable to `B`.
/// Panics if `func` is not zero-sized.
pub fn combine<F, A, B, Func>(func: Func)
    -> Option<FormatFn<A>>
    where F: FormatTrait + ?Sized, Func: Fn(&A) -> &B
{
    // Combines `get_formatter` and `make_combined` in one.
    get_formatter::<B, F>().map(|r| make_combined(func, r))
}

/// A trait for types against which formatting specifiers may be pre-checked.
///
/// Implementations may be generated automatically using `runtime-fmt-derive`
/// and `#[derive(FormatArgs)]`.
pub trait FormatArgs {
    /// Find the index within this type corresponding to the provided name.
    ///
    /// If this function returns `Some`, `get_child` with the returned index
    /// must not panic.
    fn validate_name(name: &str) -> Option<usize>;

    /// Validate that a given index is within range for this type.
    ///
    /// If this function returns `true`, `get_child` with the given index must
    /// not panic.
    fn validate_index(index: usize) -> bool;

    /// Return the formatter function for the given format trait, accepting
    /// `&Self` and using the given format trait on the value at that index.
    ///
    /// Returns `None` if the given format trait cannot format the child at
    /// that index. Panics if the index is invalid.
    fn get_child<F: FormatTrait + ?Sized>(index: usize) -> Option<FormatFn<Self>>;

    /// Return the value at the given index interpreted as a `usize`.
    ///
    /// Returns `None` if the child at the given index cannot be interpreted
    /// as a `usize`. Panics if the index is invalid.
    fn as_usize(index: usize) -> Option<fn(&Self) -> &usize>;
}
