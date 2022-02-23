use nom::{bytes::complete::take, IResult};

use crate::header::{BlockIdentifier, BlockIdentifiers};

#[derive(Debug, PartialEq)]
pub struct Block {
    pub id: BlockIdentifier,
    pub data: Vec<u8>,
}

impl std::ops::Deref for Block {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

fn block<'a>(identifier: BlockIdentifier, input: &'a [u8]) -> IResult<&'a [u8], Block> {
    let (rest, data) = take(identifier.length)(input)?;
    let block = Block {
        id: identifier,
        data: data.to_vec(),
    };
    Ok((rest, block))
}

#[derive(Debug, PartialEq)]
pub struct Blocks(pub Vec<Block>);

impl std::ops::Deref for Blocks {
    type Target = Vec<Block>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) fn blocks(block_identifiers: BlockIdentifiers, input: &[u8]) -> IResult<&[u8], Blocks> {
    let mut blocks = Vec::with_capacity(block_identifiers.len());
    for id in block_identifiers.iter().copied() {
        let (rest, block) = block(id, input)?;
        blocks.push(block);
    }
    Ok((input, Blocks(blocks)))
}

#[cfg(test)]
mod test_block {
    use crate::parser::header;

    use super::*;

    #[test]
    fn test_() {
        let input = include_bytes!("..\\dayz_inventory.edds");
        eprintln!("Crating header");
        let (data, header) = match header(input) {
            Ok(e) => e,
            Err(err) => match err {
                nom::Err::Incomplete(e) => panic!("{:?}", e),
                nom::Err::Error(e) => panic!("error: {:?}, {:?}", e.code, &e.input[0..8]),
                nom::Err::Failure(e) => panic!("failure: {:?}, {:?}", e.code, &e.input[0..8]),
            },
        };
        eprintln!("Crating blocks");
        let (_, blocks) = match blocks(header.block_identifiers, data) {
            Ok(e) => e,
            Err(err) => match err {
                nom::Err::Incomplete(e) => panic!("{:?}", e),
                nom::Err::Error(e) => panic!("error: {:?}, {:?}", e.code, &e.input[0..8]),
                nom::Err::Failure(e) => panic!("failure: {:?}, {:?}", e.code, &e.input[0..8]),
            },
        };
        blocks.iter().for_each(|block| println!("{:?}", block.id));
    }
}
