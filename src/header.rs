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

#[derive(Debug, PartialEq)]
pub struct HeaderFlags(pub Vec<u8>);

impl std::ops::Deref for HeaderFlags {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlockIdentifiers(pub Vec<BlockIdentifier>);

impl std::ops::Deref for BlockIdentifiers {
    type Target = Vec<BlockIdentifier>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BlockIdentifier {
    pub kind: BlockKind,
    pub length: usize,
}
