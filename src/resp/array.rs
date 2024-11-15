use std::ops::Deref;

use bytes::{Buf, BytesMut};

use super::{
    calc_total_length, extract_fixed_data, parse_length, RespDecode, RespEncode, RespError,
    RespFrame, BUF_CAP, CRLF_LEN,
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespArray(pub(crate) Vec<RespFrame>);

//array: "*<number-of-elements>\r\n<element-1>...<element-n>" - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        //nullarray
        if self.0.len() <= 1 {
            return b"*-1\r\n".to_vec();
        }
        buf.extend_from_slice(&format!("*{}\r\n", self.0.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
//     - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        match extract_fixed_data(buf, "*-1\r\n", "NullArray") {
            Ok(()) => Ok(()),
            Err(RespError::NotComplete) => Err(RespError::NotComplete),
            _ => Ok(()),
        }?;

        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;
        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);
        let mut frames = Vec::with_capacity(len);
        for _ in 0..len {
            let frame = RespFrame::decode(buf)?;
            frames.push(frame);
        }
        Ok(RespArray::new(frames))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl RespArray {
    pub fn new(v: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(v.into())
    }
}

impl From<Vec<RespFrame>> for RespArray {
    fn from(v: Vec<RespFrame>) -> Self {
        RespArray(v)
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::BulkString;

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_array_encode() {
        let frame: RespFrame = RespArray(vec![
            BulkString::new("set".to_string()).into(),
            BulkString::new(b"hello".to_vec()).into(),
            BulkString::new(b"world".to_vec()).into(),
        ])
        .into();
        assert_eq!(
            frame.encode(),
            b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
        )
    }

    #[test]
    fn test_null_array_encode() {
        let frame: RespFrame = RespArray(vec![BulkString::new("set".to_string()).into()]).into();
        assert_eq!(frame.encode(), b"*-1\r\n");

        let frame: RespFrame = RespArray(vec![]).into();
        assert_eq!(frame.encode(), b"*-1\r\n");
    }

    #[test]
    fn test_array_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n");
        let ret = RespArray::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        Ok(())
    }
}
