mod block;
mod compaction;
mod index;
mod memtable;
mod sstable;
mod wal;

use crate::Responder;
use bytes::Bytes;
use tokio::sync::mpsc;

pub enum Command {
    Get {
        key: Bytes,
        responder: Responder<Option<Bytes>>,
    },
    Set {
        key: Bytes,
        value: Bytes,
        responder: Responder<()>,
    },
}

#[derive(Debug)]
pub struct Engine {
    input_rx: mpsc::Receiver<Command>,
    // TODO: Channel to shutdown + tokio::select! inside run loop.
    // shutdown_rx: mpsc::Receiver<Command>,
    memtable: memtable::MemTable,
    shadow_table: memtable::MemTable,
    index: index::Index,
    wal: wal::Wal,
    shadow_table_written: bool, // TODO: May be move it to table level somehow.
}

impl Engine {
    pub fn new(rx: mpsc::Receiver<Command>) -> Engine {
        Engine {
            input_rx: rx,
            memtable: memtable::MemTable::new(),
            shadow_table: memtable::MemTable::new(),
            shadow_table_written: true,
            index: index::Index::new(),
            wal: wal::Wal {},
        }
    }

    pub async fn run(mut self) {
        // TODO: Change it to select! here to handle shutdown.
        while let Some(cmd) = self.input_rx.recv().await {
            match cmd {
                Command::Get { key, responder } => {
                    let res = self.get(key).await;
                    responder.send(res).unwrap();
                }
                Command::Set {
                    key,
                    value,
                    responder,
                } => {
                    let res = self.insert(key, value);
                    responder.send(res).unwrap();
                }
            }
        }
    }

    fn insert(&mut self, key: Bytes, value: Bytes) -> crate::Result<()> {
        match self.memtable.insert(key, value) {
            memtable::InsertResult::Full => {
                self.swap_tables();
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn get(&self, key: Bytes) -> crate::Result<Option<Bytes>> {
        if let Some(value) = self.get_from_mem(&key) {
            return Ok(Some(value));
        }

        Ok(None)

        // match self.get_from_index(key).await {
        //     Ok(res) => match res {
        //         Some(value) => Ok(Some(value)),
        //         None => Ok(None),
        //     },
        //     Err(e) => Err(e),
        // }
    }

    // It only checks hot spots: cache, memtable, shadow table.
    fn get_from_mem(&self, key: &Bytes) -> Option<Bytes> {
        // TODO: First search cache.

        if let Some(value) = self.memtable.get(key) {
            return Some(value);
        }

        if let Some(value) = self.shadow_table.get(key) {
            return Some(value);
        }

        None
    }

    async fn get_from_index(&self, key: Bytes) -> crate::Result<Option<Bytes>> {
        // unimplemented!("TODO: Make it go to disk search sstables.");
        Ok(None)
    }

    // Swap memtable and shadow table to data while sstable is building.
    fn swap_tables(&mut self) {
        // TODO: When SSTable is written WAL should be rotated.
        if self.shadow_table_written {
            self.shadow_table.clear(); // Ensure its empty.
            std::mem::swap(&mut self.memtable, &mut self.shadow_table);
        }

        panic!("handle edge case somehow");
    }
}
