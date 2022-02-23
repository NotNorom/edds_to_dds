use std::marker::PhantomData;

use crate::{
    block::{blocks, Blocks},
    header::Header,
    parser::header,
};
use anyhow::Result;

#[derive(Debug, PartialEq)]
pub struct EddsFile {
    header: Header,
    blocks: Blocks,
}

impl EddsFile {
    pub fn new(input: &[u8]) -> Result<Self> {
        let (data, header) = header(input).map_err(|err| err.to_owned())?;

        let (data, blocks) = blocks(header.block_identifiers.clone(), data).map_err(|err| err.to_owned())?;

        Ok(Self { header, blocks })
    }
}

#[cfg(test)]
mod test_converter {
    use super::*;

    #[test]
    fn test_file_creation() {
        let input = include_bytes!("..\\dayz_inventory.edds");
        let edds_file = match EddsFile::new(input) {
            Ok(file) => file,
            Err(err) => panic!("Error {:?}", err),
        };
        eprint!("{:x?}", edds_file.header);
    }
}
