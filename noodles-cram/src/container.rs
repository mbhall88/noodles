pub mod block;
pub mod compression_header;
mod header;
pub mod reference_sequence_id;
pub mod slice;

pub use self::{
    block::Block, compression_header::CompressionHeader, header::Header,
    reference_sequence_id::ReferenceSequenceId, slice::Slice,
};

use std::io;

use super::{num::Itf8, writer::compression_header::write_compression_header, DataContainer};

#[derive(Debug, Default)]
pub struct Container {
    header: Header,
    blocks: Vec<Block>,
}

impl Container {
    /// Creates an EOF container.
    pub fn eof() -> Self {
        Self::new(Header::eof(), vec![Block::eof()])
    }

    pub fn try_from_data_container(data_container: &DataContainer) -> io::Result<Self> {
        let mut buf = Vec::new();
        write_compression_header(&mut buf, data_container.compression_header())?;

        let block = Block::new(
            block::CompressionMethod::None,
            block::ContentType::CompressionHeader,
            0, // FIXME
            buf.len() as Itf8,
            buf,
            0,
        );

        // FIXME: usize => Itf8 cast
        let mut landmarks = vec![block.len() as Itf8];
        let mut blocks = vec![block];

        for slice in data_container.slices() {
            let mut slice_len = 0;

            blocks.push(slice.core_data_block().clone());
            // FIXME: usize => Itf8 cast
            slice_len += slice.core_data_block().len() as Itf8;

            for external_block in slice.external_blocks() {
                blocks.push(external_block.clone());
                // FIXME: usize => Itf8 cast
                slice_len += external_block.len() as Itf8;
            }

            let last_landmark = landmarks.last().unwrap();
            let landmark = last_landmark + slice_len;
            landmarks.push(landmark);
        }

        // TODO
        let header = Header::new(
            0,
            ReferenceSequenceId::None, // FIXME
            0,
            0,
            0,
            0,
            0,
            blocks.len() as Itf8,
            landmarks,
            0,
        );

        Ok(Self::new(header, blocks))
    }

    pub fn new(header: Header, blocks: Vec<Block>) -> Self {
        Self { header, blocks }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn is_eof(&self) -> bool {
        self.header.is_eof()
    }
}
