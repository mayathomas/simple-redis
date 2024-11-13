use std::collections::BTreeMap;

use winnow::{
    ascii::{digit1, float},
    combinator::{alt, dispatch, fail, opt, preceded, terminated},
    error::{ContextError, ErrMode},
    token::{any, take, take_until},
    PResult, Parser,
};

use crate::{
    BulkString, RespArray, RespError, RespFrame, RespMap, RespNull, SimpleError, SimpleString,
};

const CRLF: &[u8] = b"\r\n";

pub fn parse_frame_length(input: &[u8]) -> Result<usize, RespError> {
    let target = &mut (&*input);
    let ret = parse_frame_len(target);
    match ret {
        Ok(_) => {
            //calculate the distance between target and input
            let start = input.as_ptr();
            let end = (*target).as_ptr();
            let len = end as usize - start as usize;
            Ok(len)
        }
        Err(_) => Err(RespError::NotComplete),
    }
}

fn parse_frame_len(input: &mut &[u8]) -> PResult<()> {
    let mut simple_parser = terminated(take_until(0.., CRLF), CRLF).value(());
    dispatch! {any;
        b'+' => simple_parser,
        b'-' => simple_parser,
        b':' => simple_parser,
        b'$' => bulk_string_len,
        b'*' => array_len,
        b'_' => simple_parser,
        b'#' => simple_parser,
        b',' => simple_parser,
        b'%' => map_len,
        _ => fail::<_, _, _>
    }
    .parse_next(input)
}

pub fn parse_frame(input: &mut &[u8]) -> PResult<RespFrame> {
    dispatch! {any;
        b'+' => simple_string.map(RespFrame::SimpleString),
        b'-' => error.map(RespFrame::Error),
        b':' => integer.map(RespFrame::Integer),
        b'$' => alt((null_bulk_string.map(RespFrame::BulkString), bulk_string.map(RespFrame::BulkString))),
        b'*' => alt((null_array.map(RespFrame::Array), array.map(RespFrame::Array))),
        b'_' => null.map(RespFrame::Null),
        b'#' => boolean.map(RespFrame::Boolean),
        b',' => double.map(RespFrame::Double),
        b'%' => map.map(RespFrame::Map),
        _ => fail::<_, _, _>
    }
    .parse_next(input)
}

fn simple_string(input: &mut &[u8]) -> PResult<SimpleString> {
    parse_string.map(SimpleString).parse_next(input)
}

fn error(input: &mut &[u8]) -> PResult<SimpleError> {
    parse_string.map(SimpleError).parse_next(input)
}

// - integer: ":1234\r\n"
fn integer(input: &mut &[u8]) -> PResult<i64> {
    let sign = opt(alt(('+', '-'))).parse_next(input)?.unwrap_or('+');
    let sign: i64 = if sign == '+' { 1 } else { -1 };
    let v: i64 = terminated(digit1.parse_to(), CRLF).parse_next(input)?;
    Ok(sign * v)
}

fn null_bulk_string(input: &mut &[u8]) -> PResult<BulkString> {
    "-1\r\n"
        .value(BulkString("-1\r\n".as_bytes().to_vec()))
        .parse_next(input)
}

#[allow(clippy::comparison_chain)]
// - bulk string: "$5\r\nhello\r\n"
fn bulk_string(input: &mut &[u8]) -> PResult<BulkString> {
    let len = integer.parse_next(input)?;
    if len == 0 {
        return Ok(BulkString(vec![]));
    } else if len < 0 {
        return Err(err_cut("bulk string length must be non-negative"));
    }
    let data = terminated(take(len as usize), CRLF).parse_next(input)?;
    Ok(BulkString(data.to_vec()))
}

#[allow(clippy::comparison_chain)]
fn bulk_string_len(input: &mut &[u8]) -> PResult<()> {
    let len: i64 = integer.parse_next(input)?;
    if len == 0 || len == -1 {
        return Ok(());
    } else if len < -1 {
        return Err(err_cut("bulk string length must be non-negative"));
    }
    terminated(take(len as usize), CRLF)
        .value(())
        .parse_next(input)
}

//- null array: *-1\r\n"
fn null_array(input: &mut &[u8]) -> PResult<RespArray> {
    "-1\r\n".value(RespArray(vec![])).parse_next(input)
}

#[allow(clippy::comparison_chain)]
// - array: *3\r\n$3\r\nfoo\r\n$3\r\nbar\r\n$5\r\nhello\r\n"
fn array(input: &mut &[u8]) -> PResult<RespArray> {
    let len = integer.parse_next(input)?;
    if len == 0 || len == -1 {
        return Ok(RespArray(vec![]));
    } else if len < -1 {
        return Err(err_cut("array length must be non-negative"));
    }
    let mut arr = Vec::with_capacity(len as usize);
    for _ in 0..len {
        arr.push(parse_frame(input)?);
    }
    Ok(RespArray(arr))
}

#[allow(clippy::comparison_chain)]
fn array_len(input: &mut &[u8]) -> PResult<()> {
    let len = integer.parse_next(input)?;
    if len == 0 || len == -1 {
        return Ok(());
    } else if len < -1 {
        return Err(err_cut("array length must be non-negative"));
    }
    for _ in 0..len {
        parse_frame_len(input)?
    }
    Ok(())
}

// boolean: "#t\r\n"
fn boolean(input: &mut &[u8]) -> PResult<bool> {
    let b = alt(('t', 'f')).parse_next(input)?;
    Ok(b == 't')
}

// float: ",3.14\r\n"
fn double(input: &mut &[u8]) -> PResult<f64> {
    terminated(float, CRLF).parse_next(input)
}

// map: "%2\r\n+foo\r\n-bar\r\n"
fn map(input: &mut &[u8]) -> PResult<RespMap> {
    let len: i64 = integer.parse_next(input)?;
    if len <= 0 {
        return Err(err_cut("map length must be non-nagetive"));
    }
    let len = len as usize / 2;
    let mut map = BTreeMap::new();
    for _ in 0..len {
        let key = preceded('+', parse_string).parse_next(input)?;
        let value = parse_frame(input)?;
        map.insert(key, value);
    }
    Ok(RespMap(map))
}

fn map_len(input: &mut &[u8]) -> PResult<()> {
    let len: i64 = integer.parse_next(input)?;
    if len <= 0 {
        return Err(err_cut("map length must be non-nagetive"));
    }
    let len = len as usize / 2;
    for _ in 0..len {
        terminated(take_until(0.., CRLF), CRLF)
            .value(())
            .parse_next(input)?;
        parse_frame_len(input)?;
    }
    Ok(())
}

// null: "_\r\n"
fn null(input: &mut &[u8]) -> PResult<RespNull> {
    CRLF.value(RespNull).parse_next(input)
}

fn parse_string(input: &mut &[u8]) -> PResult<String> {
    terminated(take_until(0.., CRLF), CRLF)
        .map(|s: &[u8]| String::from_utf8_lossy(s).into_owned())
        .parse_next(input)
}

fn err_cut(_s: impl Into<String>) -> ErrMode<ContextError> {
    let context = ContextError::default();
    ErrMode::Cut(context)
}
