use bytes::BytesMut;
use enum_dispatch::enum_dispatch;

use super::{
    null::RespNull, BulkString, RespArray, RespDecode, RespError, RespMap, RespSet, SimpleError,
    SimpleString,
};

#[enum_dispatch(RespEncode)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),
    Integer(i64),
    BulkString(BulkString),
    Array(RespArray),
    Null(RespNull),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}

impl RespDecode for RespFrame {
    const PREFIX: &'static str = "";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'+') => {
                let frame = SimpleString::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'-') => {
                let frame = SimpleError::decode(buf)?;
                Ok(frame.into())
            }
            Some(b':') => {
                let frame = i64::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'#') => {
                let frame = bool::decode(buf)?;
                Ok(frame.into())
            }
            Some(b',') => {
                let frame = f64::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'*') => {
                let frame = RespArray::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'%') => {
                let frame = RespMap::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'~') => {
                let frame = RespSet::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'$') => {
                let frame = BulkString::decode(buf)?;
                Ok(frame.into())
            }
            None => Err(RespError::NotComplete),
            _ => Err(RespError::InvalidFrame(format!(
                "expect_length: unknown frame type: {:?}",
                buf
            ))),
        }
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'*') => RespArray::expect_length(buf),
            Some(b'~') => RespSet::expect_length(buf),
            Some(b'%') => RespMap::expect_length(buf),
            Some(b'$') => BulkString::expect_length(buf),
            Some(b':') => i64::expect_length(buf),
            Some(b'+') => SimpleString::expect_length(buf),
            Some(b'-') => SimpleError::expect_length(buf),
            Some(b'#') => bool::expect_length(buf),
            Some(b',') => f64::expect_length(buf),
            Some(b'_') => RespNull::expect_length(buf),
            _ => Err(RespError::NotComplete),
        }
    }
}

impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec()).into()
    }
}

#[cfg(test)]
mod tests {}
