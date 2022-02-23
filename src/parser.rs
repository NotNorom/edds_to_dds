use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    combinator::{cond, consumed, map, map_res, peek, recognize, success, value},
    multi::many1,
    number::complete::le_i32,
    sequence::pair,
    IResult,
};

#[derive(Debug, PartialEq)]
pub struct Header {
    pub flags: HeaderFlags,
    pub block_identifiers: BlockIdentifiers,
}

impl std::ops::Deref for Header {
    type Target = HeaderFlags;

    fn deref(&self) -> &Self::Target {
        &self.flags
    }
}

fn header(input: &[u8]) -> IResult<&[u8], Header> {
    let header_fields = pair(header_flags, block_identifiers);
    map(header_fields, |(flags, block_identifiers)| Header { flags, block_identifiers })(input)
}

#[derive(Debug, PartialEq)]
pub struct HeaderFlags(pub Vec<u8>);

impl std::ops::Deref for HeaderFlags {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn header_flags(input: &[u8]) -> IResult<&[u8], HeaderFlags> {
    let (_, has_dx10) = peek(take(84_u8))(input)?;
    let (_, flag_length) = alt((value(148_usize, tag(b"DX10")), success(128_usize)))(has_dx10)?;

    map(take(flag_length), |header_flags: &[u8]| HeaderFlags(header_flags.to_vec()))(input)
}

#[derive(Debug, PartialEq)]
pub struct BlockIdentifiers(pub Vec<BlockIdentifier>);

impl std::ops::Deref for BlockIdentifiers {
    type Target = Vec<BlockIdentifier>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn block_identifiers(input: &[u8]) -> IResult<&[u8], BlockIdentifiers> {
    map(many1(block_identifier), |bi| BlockIdentifiers(bi))(input)
}

#[derive(Debug, PartialEq)]
pub enum BlockKind {
    Copy,
    Lz4,
}

impl TryFrom<&[u8]> for BlockKind {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"COPY" => Ok(Self::Copy),
            b"LZ4 " => Ok(Self::Lz4),
            _ => Err("Invalid byte sequence for BlockKind"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BlockIdentifier {
    pub kind: BlockKind,
    pub length: usize,
}

fn block_identifier(input: &[u8]) -> IResult<&[u8], BlockIdentifier> {
    let (rest, kind) = map_res(alt((tag(b"COPY"), tag(b"LZ4 "))), BlockKind::try_from)(input)?;
    let (rest, length) = map_res(le_i32, |length| length.try_into())(rest)?;

    Ok((rest, BlockIdentifier { kind, length }))
}

#[cfg(test)]
mod test_parser {
    use super::*;

    #[test]
    fn test_parse_block_identifier() {
        let input = include_bytes!("..\\dayz_inventory.edds");
        let res = block_identifier(&input[128..input.len()]);

        assert!(res.is_ok());

        if let Ok((_, block_id)) = res {
            assert_eq!(block_id.kind, BlockKind::Copy);
            assert_eq!(block_id.length, 16);
        }
    }

    #[test]
    fn test_parse_block_identifiers() {
        let input = include_bytes!("..\\dayz_inventory.edds");
        let res = block_identifiers(&input[128..input.len()]);

        assert!(res.is_ok());

        if let Ok((_, block_ids)) = res {
            eprintln!("{:?}", block_ids);
            assert_eq!(block_ids[0].kind, BlockKind::Copy);
            assert_eq!(block_ids[0].length, 16);
        }
    }

    #[test]
    fn test_parse_header_flags() {
        let input = include_bytes!("..\\dayz_inventory.edds");
        let res = header_flags(input);

        assert!(res.is_ok());

        if let Ok((_, header_flags)) = res {
            assert_eq!(&header_flags[0..4], b"DDS ");
            assert_eq!(header_flags.len(), 128);
        }
    }

    #[test]
    fn test_parse_header() {
        let input = include_bytes!("..\\dayz_inventory.edds");
        let res = header(input);

        assert!(res.is_ok());

        if let Ok((_, header)) = res {
            assert_eq!(&header.flags[0..4], b"DDS ");
            assert_eq!(header.len(), 128);
        }
    }
}
