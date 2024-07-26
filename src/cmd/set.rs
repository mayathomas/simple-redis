use crate::{RespArray, RespFrame};

use super::{
    extract_args, validate_command, CmpType, CommandError, CommandExecutor, SAdd, Sismember,
};

impl CommandExecutor for SAdd {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        backend.sadd(self.key, self.value);
        1.into()
    }
}

impl CommandExecutor for Sismember {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        let b = backend.sismember(&self.key, &self.value);
        match b {
            true => 1.into(),
            false => 0.into(),
        }
    }
}

impl TryFrom<RespArray> for SAdd {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["sadd"], 2, CmpType::EQ)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(value))) => Ok(SAdd {
                key: String::from_utf8(key.0)?,
                value: String::from_utf8(value.0)?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}

impl TryFrom<RespArray> for Sismember {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["sismember"], 2, CmpType::EQ)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(value))) => {
                Ok(Sismember {
                    key: String::from_utf8(key.0)?,
                    value: String::from_utf8(value.0)?,
                })
            }
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}
