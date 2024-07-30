pub mod block;
mod builder;
mod compaction;
pub mod dispatcher;

type Name = String;

#[derive(Debug)]
struct SSTable {
    // size: u8,
    blocks: Vec<block::Block>,
    footer_offset: u16,
    footer: TableMetadata,
}

#[derive(Debug)]
struct TableMetadata {
    name: Name, // NOTE: timestamp_created+checksum.
    index_offset: u16,
    index: Vec<BlockIndex>,
    checksum: Vec<u16>,
}

#[derive(Debug)]
struct BlockIndex {
    first_key: Vec<u8>,
    offset_in_table: u16,
    // len: u16, // TODO: Either in bytes or number of keys in the block.
}