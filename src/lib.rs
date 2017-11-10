//! `arrayfire_serde` provides `serde` serialization and deserialization support for `arrayfire` types.
//!
//! The implementation is still experimental and lacking in many aspects.
//!
//! As of now the supported types are:
//!
//! * `arrayfire::Array` (non-complex internal type)
//! * `arrayfire::Dim4`
//! * `arrayfire::DType`
//!
//! # Examples
//!
//! Using the `derive` generators with structures
//! that contain `arrayfire` types as members.
//!
//! ```rust
//! #[macro_use]
//! extern crate serde_derive;
//! extern crate serde;
//! extern crate arrayfire;
//! extern crate arrayfire_serde;
//!
//! #[derive(Serialize, Deserialize)]
//! struct MyStruct {
//!     #[serde(with = "arrayfire_serde")]
//!     tensor: arrayfire::Array,
//!     #[serde(with = "arrayfire_serde")]
//!     sliding_window: arrayfire::Dim4,
//! }
//! # fn main() {}
//! ```
extern crate arrayfire;
extern crate serde;

use arrayfire::{Array, DType, Dim4, HasAfEnum};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeTuple;
use std::fmt;

/// Exposed serialization function used by the `serde` attributes:
///
/// * `#[serde(with = "arrayfire_serde")]`
/// * `#[serde(serialize_with = "arrayfire_serde::serialize")]`
///
/// ```rust
/// #[macro_use]
/// extern crate serde_derive;
/// extern crate serde;
/// extern crate arrayfire;
/// extern crate arrayfire_serde;
///
/// #[derive(Serialize)]
/// struct MyStruct {
///     #[serde(serialize_with = "arrayfire_serde::serialize")]
///     tensor: arrayfire::Array,
/// }
/// # fn main() {}
/// ```
pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    for<'a> Ser<'a, T>: Serialize,
    S: Serializer,
{
    Ser::new(value).serialize(serializer)
}

/// Exposed deserialization function used by the `serde` attributes:
///
/// * `#[serde(with = "arrayfire_serde")]`
/// * `#[serde(serialize_with = "arrayfire_serde::deserialize")]`
///
/// ```rust
/// #[macro_use]
/// extern crate serde_derive;
/// extern crate serde;
/// extern crate arrayfire;
/// extern crate arrayfire_serde;
///
/// #[derive(Deserialize)]
/// struct MyStruct {
///     #[serde(deserialize_with = "arrayfire_serde::deserialize")]
///     tensor: arrayfire::Array,
/// }
/// # fn main() {}
/// ```
pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    De<T>: Deserialize<'de>,
    D: Deserializer<'de>,
{
    De::deserialize(deserializer).map(De::into_inner)
}

/// Wrapper tuple struct to provide `serde::Serialize` trait for arrayfire types.
pub struct Ser<'a, T: 'a>(&'a T);

impl<'a, T> Ser<'a, T>
where
    Ser<'a, T>: serde::Serialize,
{
    pub fn new(value: &'a T) -> Self {
        Ser(value)
    }
}

/// Wrapper tuple struct to provide `serde::Deserialize` trait for arrayfire types.
pub struct De<T>(T);

impl<'de, T> De<T>
where
    De<T>: Deserialize<'de>,
{
    pub fn into_inner(self) -> T {
        self.0
    }
}

struct Serde<T>(pub T);

impl<T> Serialize for Serde<T>
where
    for<'de> De<T>: Deserialize<'de>,
    for<'a> Ser<'a, T>: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Ser::new(&self.0).serialize(serializer)
    }
}

impl<'b, T> Deserialize<'b> for Serde<T>
where
    for<'de> De<T>: Deserialize<'de>,
    for<'a> Ser<'a, T>: Serialize,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'b>,
    {
        De::deserialize(deserializer).map(De::into_inner).map(Serde)
    }
}

impl<'a> Serialize for Ser<'a, Dim4> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(4)?;
        tup.serialize_element(&self.0.get()[0])?;
        tup.serialize_element(&self.0.get()[1])?;
        tup.serialize_element(&self.0.get()[2])?;
        tup.serialize_element(&self.0.get()[3])?;
        tup.end()
    }
}

impl<'de> Deserialize<'de> for De<Dim4> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Dim4Visitor;

        impl<'de> Visitor<'de> for Dim4Visitor {
            type Value = De<Dim4>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "tuple as a seq of 4 elements")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let d0: u64 = visitor.next_element()?.expect("has element");
                let d1: u64 = visitor.next_element()?.expect("has element");
                let d2: u64 = visitor.next_element()?.expect("has element");
                let d3: u64 = visitor.next_element()?.expect("has element");
                let dim = Dim4::new(&[d0, d1, d2, d3]);
                Ok(De(dim))
            }
        }

        deserializer.deserialize_tuple(4, Dim4Visitor)
    }
}

impl<'a> Serialize for Ser<'a, DType> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let enum_value = *self.0 as u8;
        serializer.serialize_u8(enum_value)
    }
}

impl<'de> Deserialize<'de> for De<DType> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DTypeVisitor;

        impl<'de> Visitor<'de> for DTypeVisitor {
            type Value = De<DType>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "u8")
            }

            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let dtype: DType = unsafe { std::mem::transmute(i32::from(value)) };
                Ok(De(dtype))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let dtype: DType = unsafe { std::mem::transmute(value as i32) };
                Ok(De(dtype))
            }
        }

        deserializer.deserialize_u8(DTypeVisitor)
    }
}

impl<'a> Serialize for Ser<'a, Array> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let array: &Array = self.0;
        let dim = array.dims();
        let dtype: DType = array.get_type();

        let mut tup = serializer.serialize_tuple(3)?;
        tup.serialize_element(&Ser::new(&dtype))?;
        tup.serialize_element(&Ser::new(&dim))?;

        fn get_data<T: HasAfEnum>(array: &Array) -> Vec<T> {
            let mut data: Vec<T> = Vec::with_capacity(array.elements());
            unsafe {
                data.set_len(array.elements());
            }
            array.host(&mut data.as_mut_slice());
            data
        }

        match dtype {
            DType::F32 => tup.serialize_element(&get_data::<f32>(array))?,
            DType::F64 => tup.serialize_element(&get_data::<f64>(array))?,
            DType::S16 => tup.serialize_element(&get_data::<i16>(array))?,
            DType::S32 => tup.serialize_element(&get_data::<i32>(array))?,
            DType::S64 => tup.serialize_element(&get_data::<i64>(array))?,
            DType::U16 => tup.serialize_element(&get_data::<u16>(array))?,
            DType::U32 => tup.serialize_element(&get_data::<u32>(array))?,
            DType::U64 => tup.serialize_element(&get_data::<u64>(array))?,
            DType::B8 => tup.serialize_element(&get_data::<bool>(array))?,
            _ => panic!("unimplemented serialization for complex types!"),
        }

        tup.end()
    }
}

impl<'de> Deserialize<'de> for De<Array> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArrayVisitor;

        impl<'de> Visitor<'de> for ArrayVisitor {
            type Value = De<Array>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "struct ArrayStruct")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let dtype: De<DType> = seq.next_element()?.expect("has element");
                let dim: De<Dim4> = seq.next_element()?.expect("has element");

                fn get_array<T: HasAfEnum>(data: Option<Vec<T>>, dim: &Dim4) -> Array {
                    let data: Vec<T> = data.expect("has vector of elements");
                    Array::new::<T>(data.as_slice(), *dim)
                }

                match dtype.0 {
                    DType::F32 => Ok(De(get_array::<f32>(seq.next_element()?, &dim.0))),
                    DType::F64 => Ok(De(get_array::<f64>(seq.next_element()?, &dim.0))),
                    DType::S16 => Ok(De(get_array::<i16>(seq.next_element()?, &dim.0))),
                    DType::S32 => Ok(De(get_array::<i32>(seq.next_element()?, &dim.0))),
                    DType::S64 => Ok(De(get_array::<i64>(seq.next_element()?, &dim.0))),
                    DType::U16 => Ok(De(get_array::<u16>(seq.next_element()?, &dim.0))),
                    DType::U32 => Ok(De(get_array::<u32>(seq.next_element()?, &dim.0))),
                    DType::U64 => Ok(De(get_array::<u64>(seq.next_element()?, &dim.0))),
                    DType::B8 => Ok(De(get_array::<bool>(seq.next_element()?, &dim.0))),
                    _ => panic!("unimplemented deserialization for complex types!"),
                }
            }
        }
        deserializer.deserialize_tuple(3, ArrayVisitor)
    }
}
