use crate::{Backend, RespArray, RespFrame, SimpleString};

use super::{extract_args, validate_command, CommandError, CommandExecutor};

#[derive(Debug)]
pub struct Echo {
    message: String,
}

impl CommandExecutor for Echo {
    fn execute(self, _: &Backend) -> RespFrame {
        SimpleString::new(self.message).into()
    }
}

impl TryFrom<RespArray> for Echo {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["echo"], 1, super::CmpType::EQ)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(Echo {
                message: String::from_utf8(key.0)?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}
