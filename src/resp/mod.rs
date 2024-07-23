mod decode;
mod encode;

use enum_dispatch::enum_dispatch;
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

#[enum_dispatch(RespEncode)]
#[derive(Debug, PartialEq, PartialOrd)]
pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),
    Integer(i64),
    BulkString(BulkString),
    NullBulkString(NullBulkString),
    Array(RespArray),
    Null(RespNull),
    NullArray(RespNullArray),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}

#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Vec<u8>;
}

pub trait RespDecode {
    fn decode(buf: Self) -> Result<RespFrame, String>;
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct SimpleString(String);
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct SimpleError(String);
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct BulkString(Vec<u8>);
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct RespNull;
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct RespNullArray;
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct NullBulkString;
#[derive(Debug, PartialEq, PartialOrd)]
pub struct RespArray(Vec<RespFrame>);
#[derive(Debug, PartialEq, PartialOrd)]
pub struct RespMap(BTreeMap<String, RespFrame>);
#[derive(Debug, PartialEq, PartialOrd)]
pub struct RespSet(Vec<RespFrame>);

impl Deref for SimpleString {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SimpleError {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespMap {
    type Target = BTreeMap<String, RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RespMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for RespSet {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SimpleString {
    fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

#[allow(dead_code)]
impl SimpleError {
    fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}

#[allow(dead_code)]
impl BulkString {
    fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

#[allow(dead_code)]
impl RespArray {
    fn new(v: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(v.into())
    }
}

#[allow(dead_code)]
impl RespMap {
    fn new() -> Self {
        RespMap(BTreeMap::new())
    }
}

#[allow(dead_code)]
impl RespSet {
    fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespSet(s.into())
    }
}
