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
    let mut next = input;
    let mut result = 0u64;
    let mut shift = 0u64;
    loop {
        let (n, byte) = take1(next)?;
        result |= ((byte & 0x7f) as u64) << shift;
        shift += 7;
        next = n;
        if (0x80 & byte) == 0 {
            return Ok((n, result));
        }
    }
}

/// Read a signed int
fn signed_int(size: usize, input: &[u8]) -> IParserResult<i64> {
    let mut next = input;
    let mut result = 0i64;
    let mut shift = 0u64;

    loop {
        let (n, byte) = take1(next)?;
        result |= ((byte & 0x7f) as i64) << shift;
        shift += 7;
        next = n;
        if (0x80 & byte) == 0 {
            if (shift < size as u64) && (byte & 0x40) != 0 {
                result |= !0 << shift
            }
            break;
        }
    }

    Ok((next, result))
}

/// Reads the length of a vector
fn vector_length(input: &[u8]) -> IParserResult<u32> {
    unsigned_int(32, input).map_output(|n| n as u32)
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
    Ok((
        next,
        match m {
            0x00 => WasmGlobalType::Const(t),
            0x01 => WasmGlobalType::Var(t),
            // this should never happen because of the alt above
            _ => panic!("not a valid global type"),
        },
    ))
}

/* Instructions */

fn blocktype_empty(input: &[u8]) -> IParserResult<WasmBlockType> {
    tag_return(0x40, WasmBlockType::Empty)(input)
}

fn blocktype_valtype(input: &[u8]) -> IParserResult<WasmBlockType> {
    valtype(input).map_output(|t| WasmBlockType::Valtype(t))
}

fn blocktype_typeindex(input: &[u8]) -> IParserResult<WasmBlockType> {
    signed_int(33, input).map_output(|index| WasmBlockType::TypeIndex(index as i32))
}

fn blocktype(input: &[u8]) -> IParserResult<WasmBlockType> {
    alt((blocktype_empty, blocktype_valtype, blocktype_typeindex))(input)
}

/* Modules */

fn typeidx(input: &[u8]) -> IParserResult<u32> {
    unsigned_int(32, input).map_output(|idx| idx as u32)
}
fn funcidx(input: &[u8]) -> IParserResult<u32> {
    unsigned_int(32, input).map_output(|idx| idx as u32)
}
fn tableidx(input: &[u8]) -> IParserResult<u32> {
    unsigned_int(32, input).map_output(|idx| idx as u32)
}
fn memidx(input: &[u8]) -> IParserResult<u32> {
    unsigned_int(32, input).map_output(|idx| idx as u32)
}
fn globalidx(input: &[u8]) -> IParserResult<u32> {
    unsigned_int(32, input).map_output(|idx| idx as u32)
}
fn localidx(input: &[u8]) -> IParserResult<u32> {
    unsigned_int(32, input).map_output(|idx| idx as u32)
}
fn labelidx(input: &[u8]) -> IParserResult<u32> {
    unsigned_int(32, input).map_output(|idx| idx as u32)
}

fn section_id_size(id: u8, input: &[u8]) -> IParserResult<u32> {
    tuple((tag_(id), vector_length))(input).map_output(|(_, size)| size)
}

fn custom_section<'t>(input: &'t [u8]) -> IParserResult<WasmSectionContent<'t>> {
    let (next_begin, size) = section_id_size(0x00, input)?;
    let (next, name) = name(next_begin)?;
    // subtract 4 for the length of the name, then subtract the length
    let (next, bytes) = take(size - ((next_begin.len() - next.len()) as u32))(next)?;
    Ok((
        next,
        WasmSectionContent::CustomSection {
            name: name.to_string(),
            bytes,
        },
    ))
}

fn type_section<'t>(input: &'t [u8]) -> IParserResult<WasmSectionContent<'t>> {
    let (next, _) = section_id_size(0x01, input)?;
    let (next, length) = vector_length(next)?;
    let (next, functypes) = many_m(length as usize, functype)(next)?;
    Ok((next, WasmSectionContent::TypeSection { types: functypes }))
}

fn sections<'t>(input: &'t [u8]) -> IParserResult<WasmSection<'t>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_byte() {
        assert_eq!(byte(&[0x20]), Ok((vec![].as_ref(), 0x20)));
        assert_eq!(byte(&[0x21, 0x20]), Ok((vec![0x20].as_ref(), 0x21)));
    }

    #[test]
    fn test_unsigned_int() {
        assert_eq!(
            unsigned_int(32, &[0xE5, 0x8E, 0x26]),
            Ok((vec![].as_ref(), 624485))
        );
    }

    #[test]
    fn test_signed_int() {
        assert_eq!(
            signed_int(32, &[0xC0, 0xBB, 0x78]),
            Ok((vec![].as_ref(), -123456))
        );
    }

    #[test]
    fn test_valtype() {
        assert_eq!(valtype(&[0x7F]), Ok((vec![].as_ref(), WasmType::I32)));
        assert_eq!(valtype(&[0x7E]), Ok((vec![].as_ref(), WasmType::I64)));
        assert_eq!(valtype(&[0x7D]), Ok((vec![].as_ref(), WasmType::F32)));
        assert_eq!(valtype(&[0x7C]), Ok((vec![].as_ref(), WasmType::F64)));
    }

    #[test]
    fn test_resulttype() {
        use WasmType::*;
        assert_eq!(
            resulttype(&[0x03, 0x7F, 0x7E, 0x7E]),
            Ok((vec![].as_ref(), vec![I32, I64, I64]))
        );
    }

    #[test]
    fn test_custom_section() {
        let contents = vec![0x00, 0x8, 0x5, 104, 101, 108, 108, 111, 0xFF, 0xFE];
        assert_eq!(
            custom_section(contents.as_ref()),
            Ok((
                vec![].as_ref(),
                WasmSectionContent::CustomSection {
                    name: "hello".to_string(),
                    bytes: vec![0xFF, 0xFE].as_ref(),
                }
            ))
        );
    }
}
