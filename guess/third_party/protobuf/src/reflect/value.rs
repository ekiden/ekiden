use std::any::Any;

#[cfg(feature = "bytes")]
use bytes::Bytes;
#[cfg(feature = "bytes")]
use chars::Chars;

use pbcore::*;
use super::*;

pub trait ProtobufValue: Any + 'static {
    fn as_ref(&self) -> ProtobufValueRef;

    fn as_any(&self) -> &Any {
        unimplemented!()
    }

    fn is_non_zero(&self) -> bool {
        self.as_ref().is_non_zero()
    }

    fn as_ref_copy(&self) -> ProtobufValueRef<'static>
//where Self : Copy // TODO
    {
        match self.as_ref() {
            ProtobufValueRef::Bool(v) => ProtobufValueRef::Bool(v),
            ProtobufValueRef::U32(v) => ProtobufValueRef::U32(v),
            ProtobufValueRef::U64(v) => ProtobufValueRef::U64(v),
            ProtobufValueRef::I32(v) => ProtobufValueRef::I32(v),
            ProtobufValueRef::I64(v) => ProtobufValueRef::I64(v),
            ProtobufValueRef::F32(v) => ProtobufValueRef::F32(v),
            ProtobufValueRef::F64(v) => ProtobufValueRef::F64(v),
            ProtobufValueRef::Enum(v) => ProtobufValueRef::Enum(v),
            ProtobufValueRef::String(..) |
            ProtobufValueRef::Bytes(..) |
            ProtobufValueRef::Message(..) => unreachable!(),
        }
    }
}

impl ProtobufValue for u32 {
    fn as_ref(&self) -> ProtobufValueRef {
        ProtobufValueRef::U32(*self)
    }
}

impl ProtobufValue for u64 {
    fn as_ref(&self) -> ProtobufValueRef {
        ProtobufValueRef::U64(*self)
    }
}

impl ProtobufValue for i32 {
    fn as_ref(&self) -> ProtobufValueRef {
        ProtobufValueRef::I32(*self)
    }
}

impl ProtobufValue for i64 {
    fn as_ref(&self) -> ProtobufValueRef {
        ProtobufValueRef::I64(*self)
    }
}

impl ProtobufValue for f32 {
    fn as_ref(&self) -> ProtobufValueRef {
        ProtobufValueRef::F32(*self)
    }
}

impl ProtobufValue for f64 {
    fn as_ref(&self) -> ProtobufValueRef {
        ProtobufValueRef::F64(*self)
    }
}

impl ProtobufValue for bool {
    fn as_ref(&self) -> ProtobufValueRef {
        ProtobufValueRef::Bool(*self)
    }
}

impl ProtobufValue for String {
    fn as_ref(&self) -> ProtobufValueRef {
        ProtobufValueRef::String(*&self)
    }
}

impl ProtobufValue for str {
    fn as_ref(&self) -> ProtobufValueRef {
        ProtobufValueRef::String(self)
    }
}

impl ProtobufValue for Vec<u8> {
    fn as_ref(&self) -> ProtobufValueRef {
        ProtobufValueRef::Bytes(*&self)
    }
}

#[cfg(feature = "bytes")]
impl ProtobufValue for Bytes {
    fn as_ref(&self) -> ProtobufValueRef {
        ProtobufValueRef::Bytes(&*self)
    }
}

#[cfg(feature = "bytes")]
impl ProtobufValue for Chars {
    fn as_ref(&self) -> ProtobufValueRef {
        ProtobufValueRef::String(&*self)
    }
}

// conflicting implementations, so generated code is used instead
/*
impl<E : ProtobufEnum> ProtobufValue for E {
    fn as_ref(&self) -> ProtobufValueRef {
        ProtobufValueRef::Enum(self.descriptor())
    }
}

impl<M : Message> ProtobufValue for M {
    fn as_ref(&self) -> ProtobufValueRef {
        ProtobufValueRef::Message(self)
    }
}
*/


pub enum ProtobufValueRef<'a> {
    U32(u32),
    U64(u64),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(&'a str),
    Bytes(&'a [u8]),
    Enum(&'static EnumValueDescriptor),
    Message(&'a Message),
}

impl<'a> ProtobufValueRef<'a> {
    pub fn is_non_zero(&self) -> bool {
        match *self {
            ProtobufValueRef::U32(v) => v != 0,
            ProtobufValueRef::U64(v) => v != 0,
            ProtobufValueRef::I32(v) => v != 0,
            ProtobufValueRef::I64(v) => v != 0,
            ProtobufValueRef::F32(v) => v != 0.,
            ProtobufValueRef::F64(v) => v != 0.,
            ProtobufValueRef::Bool(v) => v,
            ProtobufValueRef::String(v) => !v.is_empty(),
            ProtobufValueRef::Bytes(v) => !v.is_empty(),
            ProtobufValueRef::Enum(v) => v.value() != 0,
            ProtobufValueRef::Message(_) => true,
        }
    }
}
