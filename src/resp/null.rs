use bytes::BytesMut;

use super::{extract_fixed_data, RespDecode, RespEncode, RespError};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNull;

//null: "_\r\n"
impl RespEncode for RespNull {
    fn encode(self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}

impl RespDecode for RespNull {
    const PREFIX: &'static str = "_";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "_\r\n", "Null")?;
        Ok(RespNull)
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(3)
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;

    use super::*;

    #[test]
    fn test_null_encode() {
        let frame: RespFrame = RespNull.into();
        assert_eq!(frame.encode(), b"_\r\n")
    }
}
