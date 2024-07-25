use std::ops::Deref;

use bytes::BytesMut;

use super::{extract_simple_frame_data, RespDecode, RespEncode, RespError, CRLF_LEN};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct SimpleError(pub(crate) String);

//error: "-Error message\r\n"
impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

impl RespDecode for SimpleError {
    const PREFIX: &'static str = "-";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;

        //split the buffer
        let data = buf.split_to(end + 2);
        let s = String::from_utf8_lossy(&data[1..end]);
        Ok(SimpleError::new(s.to_string()))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}

impl Deref for SimpleError {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_simple_error_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"-Error message\r\n");

        let frame = SimpleError::decode(&mut buf)?;
        assert_eq!(frame, SimpleError::new("Error message".to_string()));

        Ok(())
    }

    #[test]
    fn test_error_encode() {
        let frame: RespFrame = SimpleError::new("Error".to_string()).into();
        assert_eq!(frame.encode(), b"-Error\r\n")
    }
}
