use std::io::Read;
use std::u8;

use super::*;

macro_rules! read_fixed {
    ($buf:expr, $len:expr) => {{
        let mut arr = [Default::default(); $len];
        $buf.read(&mut arr).unwrap();
        arr
    }};
}

macro_rules! num_type {
    ($num:ident) => {
        impl Deserialize for $num {
            fn deserialize(data: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
                Ok(<$num>::from_le_bytes(read_fixed!(data, size_of::<$num>())))
            }
        }
    };
    ($num:ident is $inner:ident) => {
        impl Deserialize for $inner {
            fn deserialize(data: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
                Ok(<$inner>::from_le_bytes(read_fixed!(
                    data,
                    size_of::<$inner>()
                )))
            }
        }

        impl Deserialize for $num {
            fn deserialize(data: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
                Ok(<$inner as Deserialize>::deserialize(data)? as $num)
            }
        }
    };
}

num_type!(u8 is i8);
num_type!(u16 is i16);
num_type!(u32 is i32);
num_type!(u64 is i64);

num_type!(f64);
num_type!(f32);

impl Deserialize for bool {
    fn deserialize(data: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        Ok(u8::deserialize(data)? != 0)
    }
}

// Basic Types
// (java doesn't have uints)
impl Deserialize for Duration {
    fn deserialize(data: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        Ok(Duration::from_micros(u64::deserialize(data)?))
    }
}

impl<T> Deserialize for Option<T>
where
    T: Deserialize,
{
    fn deserialize(data: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        if !bool::deserialize(data)? {
            Ok(None)
        } else {
            Ok(Some(T::deserialize(data)?))
        }
    }
}

impl<T> Deserialize for Vec<T>
where
    T: Deserialize,
{
    fn deserialize(data: &mut Cursor<&[u8]>) -> Result<Self, DeserializeError> {
        let len = u8::deserialize(data)?;
        let mut vec = vec![];

        for _ in 0..len {
            vec.push(T::deserialize(data)?);
        }

        Ok(vec)
    }
}
