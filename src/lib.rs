mod ast;
use crate::ast::*;
use nom::{
    branch::alt,
    bytes::streaming::{tag, take},
    error::ParseError,
    multi::many_m_n,
    sequence::tuple,
    IResult,
};

type Input<'t> = &'t [u8];
type IParserResult<'t, O> = IResult<Input<'t>, O>;

trait Parser<I, O, E>: Fn(I) -> IResult<I, O, E> {}
impl<I, O, E, F> Parser<I, O, E> for F
where
    I: Clone + PartialEq,
    E: ParseError<I>,
    F: Fn(I) -> IResult<I, O, E>,
{
}

trait MapOutput<I, O, E> {
    fn map_output<T, F>(self, f: F) -> IResult<I, T, E>
    where
        F: Fn(O) -> T;
}

impl<I, O, E> MapOutput<I, O, E> for IResult<I, O, E> {
    fn map_output<T, F>(self, f: F) -> IResult<I, T, E>
    where
        F: Fn(O) -> T,
    {
        self.map(|(next, value)| (next, f(value)))
    }
}

/// takes 1 byte from the input stream and returns it
fn take1(input: &[u8]) -> IParserResult<u8> {
    take(1u8)(input).map(|(rest, output)| (rest, output[0]))
}

fn tag_return<'t, T: Clone>(t: u8, ret: T) -> impl Fn(&'t [u8]) -> IParserResult<T> {
    move |input: &'t [u8]| {
        let (next, byte) = take1(input)?;
        if byte == t {
            Ok((next, ret.clone()))
        } else {
            Err(nom::Err::Error((next, nom::error::ErrorKind::Tag)))
        }
    }
}

fn tag_<'t>(t: u8) -> impl Parser<&'t [u8], u8, (&'t [u8], nom::error::ErrorKind)> {
        tag_return::<'t>(t, t)
}

fn many_m<I, O, E>(m: usize, parser: impl Parser<I, O, E>) -> impl Parser<I, Vec<O>, E>
where
    I: Clone + PartialEq,
    E: ParseError<I>,
{
    many_m_n(m, m, parser)
}
/* WASM VALUES */

fn byte(input: &[u8]) -> IParserResult<u8> {
    take1(input)
}

/// Read an unsigned int.
fn unsigned_int(size: usize, input: &[u8]) -> IParserResult<u64> {
    todo!()
}

/// Read a signed int
fn signed_int(size: usize, input: &[u8]) -> IParserResult<i64> {
    todo!()
}

/// Reads the length of a vector
fn vector_length(input: &[u8]) -> IParserResult<u32> {
    todo!()
}

/// Read a name, this parser expects a length read from a previous
/// parser.
fn name(input: &[u8]) -> IParserResult<&str> {
    let (next, length) = vector_length(input)?;
    let (next, bytes) = take(length)(next)?;
    // TODO: convert the error to a valid parser error
    Ok((next, std::str::from_utf8(bytes).expect("a valid string")))
}

/* WASM types */

fn valtype(input: &[u8]) -> IParserResult<WasmType> {
    alt((
        tag_return(0x7F, WasmType::I32),
        tag_return(0x7E, WasmType::I64),
        tag_return(0x7D, WasmType::F32),
        tag_return(0x7C, WasmType::F64),
    ))(input)
}

fn resulttype(input: &[u8]) -> IParserResult<Vec<WasmType>> {
    let (next, length) = vector_length(input)?;
    many_m(length as usize, valtype)(next)
}

fn functype(input: &[u8]) -> IParserResult<WasmFunctionType> {
    let (next, _) = tag_return(0x60, ())(input)?;
    let (next, (parameter_types, result_types)) = tuple((resulttype, resulttype))(next)?;

    Ok((
        next,
        WasmFunctionType {
            parameter_types,
            result_types,
        },
    ))
}

fn limits_without_max(input: &[u8]) -> IParserResult<WasmLimitType> {
    let (next, _) = tag_return(0x00, ())(input)?;
    let (next, n) = unsigned_int(32, next)?;
    Ok((
        next,
        WasmLimitType {
            min: n as u32,
            max: None,
        },
    ))
}

fn limits_with_max(input: &[u8]) -> IParserResult<WasmLimitType> {
    let (next, _) = tag_return(0x01, ())(input)?;
    let (next, n) = unsigned_int(32, next)?;
    let (next, m) = unsigned_int(32, next)?;
    Ok((
        next,
        WasmLimitType {
            min: n as u32,
            max: Some(m as u32),
        },
    ))
}

fn limits(input: &[u8]) -> IParserResult<WasmLimitType> {
    alt((limits_without_max, limits_with_max))(input)
}

fn elemtype(input: &[u8]) -> IParserResult<WasmElemType> {
    // note, that there is only one elemtype,
    // however as we follow the spec, we create
    // a production rule for this as well, possibly
    // this is to facilite other types of tables in the future
    tag_return(0x70, WasmElemType::FuncRef)(input)
}

fn tabletype(input: &[u8]) -> IParserResult<WasmTableType> {
    tuple((elemtype, limits))(input).map_output(|(et, lim)| WasmTableType {
        elemtype: et,
        limits: lim,
    })
}

fn globaltype(input: &[u8]) -> IParserResult<WasmGlobalType> {
    let (next, t) = valtype(input)?;
    let (next, m) = alt((tag_(0x00), tag_(0x01)))(next)?;
    Ok((next, match m {
        0x00 => WasmGlobalType::Const(t),
        0x01 => WasmGlobalType::Var(t),
        // this should never happen because of the alt above
        _ => panic!("not a valid global type")
    }))
}

/* Instructions */


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_byte() {
        assert_eq!(byte(&[0x20]), Ok((vec![].as_ref(), 0x20)));
        assert_eq!(byte(&[0x21, 0x20]), Ok((vec![0x20].as_ref(), 0x21)));
    }

    #[test]
    fn test_valtype() {
        assert_eq!(valtype(&[0x7F]), Ok((vec![].as_ref(), WasmType::I32)));
        assert_eq!(valtype(&[0x7E]), Ok((vec![].as_ref(), WasmType::I64)));
        assert_eq!(valtype(&[0x7D]), Ok((vec![].as_ref(), WasmType::F32)));
        assert_eq!(valtype(&[0x7C]), Ok((vec![].as_ref(), WasmType::F64)));
    }

    #[test]
    fn test_resulttype() {}
}
