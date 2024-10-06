#![doc = include_str!("README.md")]

/// Derive macro to implement [`Enum`] on enums.
///
/// ```
/// # use discriminant::Enum;
/// # #[allow(unused)]
/// #[derive(Enum)]
/// enum MyEnum {
///     Variant1,
///     Variant2(usize, &'static str),
///     Variant3 {
///         name: String,
///         age: i32,
///     }
/// }
/// ```
///
/// # Supported annotations
///
/// - `#[discriminant([path])]` ... sprcify path to the crate, defaults to `::discriminant`
/// - `#[discriminant_attr = "#[attr1] #[attr2] ..."]` ... specify annotations applied onto
///    [`Enum::Discriminant`] type or its variants, defined by the derive macro.
///
/// # [`Enum::Discriminant`] type
///
/// This macro also generates definition of `Discriminant` type,
/// which has the same layout with representation of the original enum (configurable using
/// `#[repr(...)]` attribute). The discriminant type is an enum which contains the same variants
/// with the original enum, but does not contain any fields (in other words, all of the variants of
/// the discriminant enum is tuple-like variant).
pub use discriminant_macro::Enum;

/// Represents an Enum definition, and supports method to get discriminant of a variant.
///
/// Implemented by `#[derive(Enum)]` derive macro.
pub unsafe trait Enum: Sized {
    /// Discriminant type of the enum, like [`std::mem::Discriminant`].
    type Discriminant: Discriminant;

    /// Get discriminant of the enum value, like [`std::mem::discriminant()`].
    fn discriminant(&self) -> Self::Discriminant;
}

pub unsafe trait Discriminant:
    Clone
    + Copy
    + core::fmt::Debug
    + core::fmt::Display
    + core::hash::Hash
    + PartialEq
    + Eq
    + Ord
    + PartialOrd
    + Sized
    + TryFrom<Self::Repr, Error = ()>
    + Into<Self::Repr>
{
    /// Representation of the discriminant type, like `i8` or `usize`.
    type Repr;

    /// Iterate all discriminants represented by the type.
    fn all() -> impl Iterator<Item = Self>;

    // type Set: DiscriminantSet<Discriminant = Self>;
}

// pub unsafe trait DiscriminantSet:
//     IntoIterator<Item = Self::Discriminant>
//     + Clone
//     + Copy
//     + core::fmt::Display
//     + core::fmt::Debug
//     + core::hash::Hash
//     + PartialEq
//     + Eq
//     + PartialOrd
//     + Sized
//     + core::ops::Add
//     + core::ops::Sub
//     + core::ops::AddAssign
//     + core::ops::SubAssign
//     + core::iter::Sum
// {
//     type Discriminant: Discriminant;
// }
