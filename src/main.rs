mod parser;

use std::{
    env::args,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Cursor, Read, Seek, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};

/// 50 mebibyte buffer
const INITIAL_CAPACITY: usize = 50 * 1024 * 1024;

#[derive(Clone, Debug)]
enum BlockHeader {
    Copy(i32),
    Lz4(i32),
}

impl BlockHeader {
    fn from_bytes(buffer: &[u8; 8]) -> Result<Self> {
        match &buffer[0..4] {
            b"COPY" => Ok(Self::Copy(i32::from_le_bytes(buffer[4..8].try_into()?))),
            b"LZ4 " => Ok(Self::Lz4(i32::from_le_bytes(buffer[4..8].try_into()?))),
            _ => Err(anyhow::anyhow!("Header is over...")),
        }
    }
}

#[derive(Clone, Debug)]
struct Header {
    header: [u8; 128],
    dx_10: Option<[u8; 20]>,
    block_headers: Vec<BlockHeader>,
}

impl Header {
    fn from_bytes(buffer: &[u8]) -> Result<Header> {
        let header = buffer[0..128].try_into()?;

        let mut start_of_block_headers = 128;

        let mut dx_10 = Option::<[u8; 20]>::None;

        if &buffer[84..88] == b"DX10" {
            dx_10 = Some(buffer[128..148].try_into()?);
            start_of_block_headers += 20;
        }

        let block_headers = buffer[start_of_block_headers..]
            .chunks_exact(8)
            .map_while(|buffer| BlockHeader::from_bytes(buffer.try_into().context("Wrong length... should not happen").unwrap()).ok())
            .collect();

        Ok(Header {
            header,
            dx_10,
            block_headers,
        })
    }

    fn len(&self) -> usize {
        let dx_len = match self.dx_10 {
            Some(_) => 20,
            None => 0,
        };
        let block_len = self.block_headers.len() * 8;
        128 + dx_len + block_len
    }
}

#[derive(Clone, Debug)]
struct EddsToDdsConverter {
    path: PathBuf,
    input_buffer: Vec<u8>,
    output_buffer: Vec<u8>,
}

impl EddsToDdsConverter {
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let path = path.into();
        let mut file = File::open(&path).context("Could not open file")?;
        let mut input_buffer = Vec::with_capacity(INITIAL_CAPACITY);
        let output_buffer = Vec::with_capacity(INITIAL_CAPACITY);

        eprintln!("Reserving {INITIAL_CAPACITY} bytes");
        let bytes_read = file.read_to_end(&mut input_buffer).context("Could not read file")?;
        eprintln!("Read {bytes_read} bytes");
        Ok(Self {
            path,
            input_buffer,
            output_buffer,
        })
    }
    fn parse_header(&mut self) -> Result<Header> {
        Header::from_bytes(&self.input_buffer)
    }

    pub fn convert(&mut self) -> Result<()> {
        let header = self.parse_header()?;
        eprintln!("{:?}", header.block_headers);

        let mut block_buffer = Vec::<Vec<u8>>::with_capacity(header.block_headers.len());

        //let mut input_cursor = Cursor::new(self.input_buffer);

        let mut offset = header.len();

        for block_header in header.block_headers {
            eprintln!("offset {:#010x}", offset);

            match block_header {
                BlockHeader::Copy(block_size) => {
                    let data = &self.input_buffer[offset..offset + (block_size as usize)];
                    block_buffer.push(Vec::from(data));
                    offset += block_size as usize;
                }
                BlockHeader::Lz4(block_size) => {
                    let mut decomp_offset = offset;

                    let uncompressed_size = i32::from_le_bytes(self.input_buffer[decomp_offset..decomp_offset + 4].try_into()?);
                    decomp_offset += 4;

                    'frame_loop: loop {
                        let frame_size = i32::from_le_bytes(self.input_buffer[decomp_offset..decomp_offset + 4].try_into()?) & i32::MAX;
                        if frame_size >= block_size {
                            break 'frame_loop;
                        }
                        eprintln!("  frame size: {:>7}", frame_size);
                        decomp_offset += 4;

                        let frame = &self.input_buffer[decomp_offset..decomp_offset + frame_size as usize];
                        decomp_offset += frame_size as usize;

                        let uncomp_size = 65536.min(uncompressed_size as usize);

                        match lz4_flex::decompress(frame, uncomp_size as usize) {
                            Ok(decomp_data) => {}
                            Err(err) => eprintln!("    decomp error: {}", err),
                        }
                    }

                    //let decompressed = lz4::block::decompress(compressed, Some(uncompressed_size))?;
                    // match lz4_flex::decompress(compressed, uncompressed_size.try_into().unwrap()) {
                    //     Ok(decompressed) => {
                    //         block_buffer.push(decompressed);
                    //     }
                    //     Err(err) => eprintln!("decomp error: {}", err),
                    // }
                    offset += block_size as usize;
                }
            };
        }

        block_buffer.reverse();

        for block in block_buffer {
            self.output_buffer.write_all(&block)?;
        }

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let mut new_path = self.path.clone();
        new_path.set_extension("dds");
        let mut new_file = File::create(new_path)?;
        new_file.write_all(&self.output_buffer)?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let path = args().nth(1).context("Missing argument: file path")?;
    let mut converter = EddsToDdsConverter::new(path)?;

    if let Err(err) = converter.convert() {
        eprintln!("{:#?}", err);
    }
    converter.save()?;
    Ok(())
}
