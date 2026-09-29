//! An anonymous, delete-on-close audio spool. Ingestion never waits for inference.
//! Only unprocessed audio is queued in memory; long inference backlogs live on disk.
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Condvar, Mutex};

use anyhow::{ensure, Result};

pub struct TimedChunk {
    pub start_ms: u64,
    pub pcm: Vec<u8>,
}

// About 18 hours of 16 kHz mono audio per source. Exhaustion is a visible failure, never
// silent loss or unbounded RAM growth. The file disappears when its last handle closes.
const MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_CHUNK_BYTES: usize = 32_000;

struct Data {
    file: File,
    written: u64,
    read: u64,
    ended: bool,
}

pub struct Spool {
    data: Mutex<Data>,
    changed: Condvar,
    queued_samples: AtomicU64,
    pub overflowed: AtomicBool,
    pub finalizing: AtomicBool,
    pub capture_error: Mutex<Option<String>>,
}

impl Spool {
    pub fn new() -> Result<Self> {
        Ok(Self {
            data: Mutex::new(Data {
                file: tempfile::tempfile()?,
                written: 0,
                read: 0,
                ended: false,
            }),
            changed: Condvar::new(),
            queued_samples: AtomicU64::new(0),
            overflowed: AtomicBool::new(false),
            finalizing: AtomicBool::new(false),
            capture_error: Mutex::new(None),
        })
    }

    pub fn push(&self, chunk: TimedChunk) -> Result<()> {
        ensure!(
            chunk.pcm.len() <= MAX_CHUNK_BYTES && chunk.pcm.len().is_multiple_of(2),
            "Invalid local audio frame"
        );
        let mut data = self.data.lock().unwrap_or_else(|e| e.into_inner());
        let end = data.written + 12 + chunk.pcm.len() as u64;
        ensure!(
            end <= MAX_BYTES,
            "Local audio buffer limit reached; stop and export this session"
        );
        let offset = data.written;
        data.file.seek(SeekFrom::Start(offset))?;
        data.file.write_all(&chunk.start_ms.to_le_bytes())?;
        data.file
            .write_all(&(chunk.pcm.len() as u32).to_le_bytes())?;
        data.file.write_all(&chunk.pcm)?;
        data.written = end;
        self.queued_samples
            .fetch_add(chunk.pcm.len() as u64 / 2, Ordering::Relaxed);
        self.changed.notify_one();
        Ok(())
    }

    pub fn finish(&self) {
        let mut data = self.data.lock().unwrap_or_else(|e| e.into_inner());
        data.ended = true;
        self.finalizing.store(true, Ordering::Relaxed);
        self.changed.notify_all();
    }

    pub fn next(&self) -> Result<Option<TimedChunk>> {
        let mut data = self.data.lock().unwrap_or_else(|e| e.into_inner());
        while data.read == data.written && !data.ended {
            data = self.changed.wait(data).unwrap_or_else(|e| e.into_inner());
        }
        if data.read == data.written {
            return Ok(None);
        }
        let offset = data.read;
        data.file.seek(SeekFrom::Start(offset))?;
        let mut header = [0u8; 12];
        data.file.read_exact(&mut header)?;
        let start_ms = u64::from_le_bytes(header[..8].try_into()?);
        let len = u32::from_le_bytes(header[8..].try_into()?) as usize;
        ensure!(len <= MAX_CHUNK_BYTES, "Invalid local audio buffer");
        let mut pcm = vec![0; len];
        data.file.read_exact(&mut pcm)?;
        data.read += 12 + len as u64;
        // Reuse the file when inference has caught up, keeping a long meeting's disk use
        // proportional to its backlog rather than its entire duration.
        if data.read == data.written {
            data.file.set_len(0)?;
            data.read = 0;
            data.written = 0;
        }
        Ok(Some(TimedChunk { start_ms, pcm }))
    }

    pub fn complete(&self, samples: usize) {
        self.queued_samples
            .fetch_sub(samples as u64, Ordering::Relaxed);
    }

    pub fn pending_ms(&self) -> u64 {
        self.queued_samples.load(Ordering::Relaxed) / 16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn slow_reader_keeps_every_frame_and_timestamp_through_stop() {
        let spool = Arc::new(Spool::new().unwrap());
        for i in 0..2_000u64 {
            spool
                .push(TimedChunk {
                    start_ms: i * 100,
                    pcm: (i as i16).to_le_bytes().repeat(1600),
                })
                .unwrap();
        }
        assert_eq!(spool.pending_ms(), 200_000);
        spool.finish();
        for i in 0..2_000u64 {
            let frame = spool.next().unwrap().unwrap();
            assert_eq!(frame.start_ms, i * 100);
            assert_eq!(frame.pcm, (i as i16).to_le_bytes().repeat(1600));
            spool.complete(1600);
        }
        assert!(spool.next().unwrap().is_none());
        assert_eq!(spool.pending_ms(), 0);
    }

    #[test]
    fn caught_up_spool_can_be_reused_and_wakes_on_stop() {
        let spool = Arc::new(Spool::new().unwrap());
        for _ in 0..3 {
            spool
                .push(TimedChunk {
                    start_ms: 42,
                    pcm: vec![0; 3200],
                })
                .unwrap();
            assert_eq!(spool.next().unwrap().unwrap().start_ms, 42);
            spool.complete(1600);
            assert_eq!(spool.data.lock().unwrap().file.metadata().unwrap().len(), 0);
        }
        let other = spool.clone();
        let reader = std::thread::spawn(move || other.next().unwrap());
        spool.finish();
        assert!(reader.join().unwrap().is_none());
    }
}
