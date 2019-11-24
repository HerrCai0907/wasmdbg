use std::fmt;
use std::str::FromStr;

use bwasm::ValueType;

use crate::vm::{Trap, VMResult};
use crate::{F32, F64};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(F32),
    F64(F64),
}

impl Value {
    pub fn default(value_type: ValueType) -> Self {
        match value_type {
            ValueType::I32 => Value::I32(0),
            ValueType::I64 => Value::I64(0),
            ValueType::F32 => Value::F32(F32::default()),
            ValueType::F64 => Value::F64(F64::default()),
        }
    }

    pub fn value_type(&self) -> ValueType {
        match self {
            Value::I32(_) => ValueType::I32,
            Value::I64(_) => ValueType::I64,
            Value::F32(_) => ValueType::F32,
            Value::F64(_) => ValueType::F64,
        }
    }

    pub fn to<T: Number>(&self) -> Option<T> {
        T::from_value(*self)
    }

    pub fn from_str(s: &str, value_type: ValueType) -> Option<Self> {
        Some(match value_type {
            ValueType::I32 => Value::I32(i64::from_str(s).ok()? as i32),
            ValueType::I64 => Value::I64(i128::from_str(s).ok()? as i64),
            ValueType::F32 => Value::from(f32::from_str(s).ok()?),
            ValueType::F64 => Value::from(f64::from_str(s).ok()?),
        })
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Value::I32(val) => {
                if (val as i32) < 0 {
                    write!(f, "i32 : 0x{0:08x} = {0} = {1}", val, val as i32)
                } else {
                    write!(f, "i32 : 0x{0:08x} = {0}", val)
                }
            }
            Value::I64(val) => {
                if (val as i64) < 0 {
                    write!(f, "i64 : 0x{0:016x} = {0} = {1}", val, val as i64)
                } else {
                    write!(f, "i64 : 0x{0:016x} = {0}", val)
                }
            }
            Value::F32(val) => write!(f, "f32 : 0x{:08x} ~ {:.8}", val.to_bits(), val.to_float()),
            Value::F64(val) => write!(f, "f64 : 0x{:016x} ~ {:.16}", val.to_bits(), val.to_float()),
        }
    }
}

impl From<i32> for Value {
    fn from(val: i32) -> Self {
        Value::I32(val)
    }
}
impl From<u32> for Value {
    fn from(val: u32) -> Self {
        Value::I32(val as i32)
    }
}
impl From<i64> for Value {
    fn from(val: i64) -> Self {
        Value::I64(val)
    }
}
impl From<u64> for Value {
    fn from(val: u64) -> Self {
        Value::I64(val as i64)
    }
}
impl From<f32> for Value {
    fn from(val: f32) -> Self {
        Value::F32(F32::from(val))
    }
}
impl From<f64> for Value {
    fn from(val: f64) -> Self {
        Value::F64(F64::from(val))
    }
}
impl From<F32> for Value {
    fn from(val: F32) -> Self {
        Value::F32(val)
    }
}
impl From<F64> for Value {
    fn from(val: F64) -> Self {
        Value::F64(val)
    }
}

pub trait Number: Into<Value> + Copy + fmt::Display {
    fn value_type() -> ValueType;
    fn from_value(val: Value) -> Option<Self>;
}

macro_rules! impl_number {
    (float $num_t:ident, $value_t:ident) => {
        impl Number for $num_t {
            fn value_type() -> ValueType {
                ValueType::$value_t
            }

            fn from_value(val: Value) -> Option<Self> {
                if let Value::$value_t(val) = val {
                    return Some(val.into());
                }
                None
            }
        }
    };
    (int $num_t:ident, $value_t:ident) => {
        impl Number for $num_t {
            fn value_type() -> ValueType {
                ValueType::$value_t
            }

            fn from_value(val: Value) -> Option<Self> {
                if let Value::$value_t(val) = val {
                    return Some(val as $num_t);
                }
                None
            }
        }
    };
}

impl_number!(int u32, I32);
impl_number!(int i32, I32);
impl_number!(int u64, I64);
impl_number!(int i64, I64);
impl_number!(float f32, F32);
impl_number!(float f64, F64);
impl_number!(float F32, F32);
impl_number!(float F64, F64);

pub trait Integer: Sized {
    fn leading_zeros(self) -> Self;
    fn trailing_zeros(self) -> Self;
    fn count_ones(self) -> Self;
    fn rotl(self, other: Self) -> Self;
    fn rotr(self, other: Self) -> Self;
    fn rem(self, other: Self) -> VMResult<Self>;
    fn div(self, other: Self) -> VMResult<Self>;
    fn from_str_with_radix(s: &str) -> Result<Self, std::num::ParseIntError>;
}

macro_rules! impl_integer {
    ($type:ident) => {
        #[allow(clippy::cast_lossless)]
        impl Integer for $type {
            fn leading_zeros(self) -> Self {
                self.leading_zeros() as $type
            }
            fn trailing_zeros(self) -> Self {
                self.trailing_zeros() as $type
            }
            fn count_ones(self) -> Self {
                self.count_ones() as $type
            }
            fn rotl(self, other: Self) -> Self {
                self.rotate_left(other as u32)
            }
            fn rotr(self, other: Self) -> Self {
                self.rotate_right(other as u32)
            }
            fn rem(self, other: Self) -> VMResult<Self> {
                if other == 0 {
                    Err(Trap::DivisionByZero)
                } else {
                    Ok(self.wrapping_rem(other))
                }
            }
            fn div(self, other: Self) -> VMResult<Self> {
                if other == 0 {
                    return Err(Trap::DivisionByZero);
                }
                let (result, is_overflow) = self.overflowing_div(other);
                if is_overflow {
                    Err(Trap::SignedIntegerOverflow)
                } else {
                    Ok(result)
                }
            }
            fn from_str_with_radix(s: &str) -> Result<Self, std::num::ParseIntError> {
                let radix = if s.len() > 2 {
                    match s[0..2].to_lowercase().as_str() {
                        "0x" => Some(16),
                        "0o" => Some(8),
                        "0b" => Some(2),
                        _ => None,
                    }
                } else {
                    None
                };
                if let Some(radix) = radix {
                    $type::from_str_radix(&s[2..], radix)
                } else {
                    $type::from_str_radix(s, 10)
                }
            }
        }
    };
}

impl_integer!(i16);
impl_integer!(i32);
impl_integer!(u32);
impl_integer!(i64);
impl_integer!(u64);
impl_integer!(i128);

pub trait LittleEndianConvert: Sized {
    fn from_little_endian(buffer: &[u8]) -> Self;
    fn to_little_endian(self, buffer: &mut [u8]);
}

impl LittleEndianConvert for i8 {
    fn from_little_endian(buffer: &[u8]) -> Self {
        buffer[0] as i8
    }

    fn to_little_endian(self, buffer: &mut [u8]) {
        buffer[0] = self as u8;
    }
}

impl LittleEndianConvert for u8 {
    fn from_little_endian(buffer: &[u8]) -> Self {
        buffer[0]
    }

    fn to_little_endian(self, buffer: &mut [u8]) {
        buffer[0] = self;
    }
}

macro_rules! impl_little_endian_convert_int {
    ($t:ident) => {
        impl LittleEndianConvert for $t {
            fn from_little_endian(buffer: &[u8]) -> Self {
                const SIZE: usize = core::mem::size_of::<$t>();
                let mut buf = [0u8; SIZE];
                buf.copy_from_slice(&buffer[0..SIZE]);
                Self::from_le_bytes(buf)
            }

            fn to_little_endian(self, buffer: &mut [u8]) {
                buffer.copy_from_slice(&self.to_le_bytes());
            }
        }
    };
}

macro_rules! impl_little_endian_convert_float {
    ($t:ident, $repr:ident) => {
        impl LittleEndianConvert for $t {
            fn from_little_endian(buffer: &[u8]) -> Self {
                Self::from_bits($repr::from_little_endian(buffer))
            }

            fn to_little_endian(self, buffer: &mut [u8]) {
                self.to_bits().to_little_endian(buffer);
            }
        }
    };
}

impl_little_endian_convert_int!(i16);
impl_little_endian_convert_int!(u16);
impl_little_endian_convert_int!(i32);
impl_little_endian_convert_int!(u32);
impl_little_endian_convert_int!(i64);
impl_little_endian_convert_int!(u64);
impl_little_endian_convert_float!(f32, u32);
impl_little_endian_convert_float!(f64, u64);
impl_little_endian_convert_float!(F32, u32);
impl_little_endian_convert_float!(F64, u64);

pub trait ExtendTo<T> {
    fn extend_to(self) -> T;
}

macro_rules! impl_extend_to {
    ($from:ident, $to:ident) => {
        #[allow(clippy::cast_lossless)]
        impl ExtendTo<$to> for $from {
            fn extend_to(self) -> $to {
                self as $to
            }
        }
    };
}

impl_extend_to!(i8, u32);
impl_extend_to!(u8, u32);
impl_extend_to!(u16, u32);
impl_extend_to!(i16, u32);
impl_extend_to!(i8, u64);
impl_extend_to!(u8, u64);
impl_extend_to!(i16, u64);
impl_extend_to!(u16, u64);
impl_extend_to!(i32, u64);
impl_extend_to!(u32, u64);

pub trait WrapTo<T> {
    fn wrap_to(self) -> T;
}

macro_rules! impl_wrap_to {
    ($from:ident, $to:ident) => {
        impl WrapTo<$to> for $from {
            fn wrap_to(self) -> $to {
                self as $to
            }
        }
    };
}

impl_wrap_to!(u8, u8);
impl_wrap_to!(u16, u8);
impl_wrap_to!(u32, u8);
impl_wrap_to!(u32, u16);
impl_wrap_to!(u64, u8);
impl_wrap_to!(u64, u16);
impl_wrap_to!(u64, u32);
