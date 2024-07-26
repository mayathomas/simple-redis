use bytes::BytesMut;

use super::{extract_simple_frame_data, RespDecode, RespEncode, RespError, CRLF_LEN};

//integer: ":[<+|->]<value>\r\n"
impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        // 加上符号，客户端会报错Error: Bad integer value
        // let sign = if self < 0 { "" } else { "" };
        format!(":{}\r\n", self).into_bytes()
    }
}

impl RespDecode for i64 {
    const PREFIX: &'static str = ":";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;

        //split the buffer
        let data = buf.split_to(end + 2);
        let s = String::from_utf8_lossy(&data[1..end]);
        Ok(s.parse()?)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;

    use super::*;

    #[test]
    fn test_integer_encode() {
        let frame: RespFrame = 123.into();
        assert_eq!(frame.encode(), b":123\r\n");
        let frame: RespFrame = (-123).into();
        assert_eq!(frame.encode(), b":-123\r\n")
    }
}
