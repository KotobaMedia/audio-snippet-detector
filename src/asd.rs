use std::{
    io::{Cursor, Seek, Write},
    sync::{
        mpsc::{channel, Receiver, RecvError, SendError, Sender},
        Arc, Mutex,
    },
    thread,
};

use neon::types::Finalize;

use crate::{db, mfcc::MfccIter, util};

#[derive(Clone)]
pub struct AudioSnippetDetector {
    pub db: Arc<Mutex<db::Database>>,

    input_tx: Option<Sender<Vec<u8>>>,
    // input_rx: Arc<Mutex<Receiver<Vec<u8>>>>,
    // output_tx: Sender<String>,
    output_rx: Arc<Mutex<Receiver<String>>>,
}
impl AudioSnippetDetector {
    pub fn new() -> Self {
        let db = Arc::new(Mutex::new(db::Database::new()));
        let (input_tx, input_rx) = channel::<Vec<u8>>();
        let (output_tx, output_rx) = channel();

        // create the processing thread
        let thread_db = db.clone();
        thread::spawn(move || {
            let cursor = Arc::new(Mutex::new(Cursor::new(Vec::<u8>::new())));
            let mfcc_stream = MfccIter::new(cursor.clone());
            let mut overlapping_stream = util::OverlappingMfccStream::new(mfcc_stream, 100);

            while let Ok(bytes) = input_rx.recv() {
                if bytes.len() == 0 {
                    break;
                }

                // Write the bytes to the cursor
                // we are using a closure here so we can release the lock on the cursor
                {
                    let mut buffer = cursor.lock().unwrap();
                    let written_bytes = buffer.write(&bytes).unwrap();
                    buffer.seek_relative(-(written_bytes as i64)).unwrap();
                }

                while let Some(mfcc_vec) = overlapping_stream.next() {
                    let db = thread_db.lock().unwrap();
                    let query_result = db.query(mfcc_vec.view());
                    if let Some(result) = query_result {
                        println!("Best match: {} with score {}", result.label, result.score);
                        output_tx.send(result.label).unwrap();
                    } else {
                        println!("No match found");
                        output_tx.send("No match found".to_string()).unwrap();
                    }
                }
            }
        });

        Self {
            db,
            input_tx: Some(input_tx),
            output_rx: Arc::new(Mutex::new(output_rx)),
        }
    }

    pub fn next(&self) -> Result<String, RecvError> {
        let rx = self.output_rx.lock().unwrap();
        let value = rx.recv()?;
        Ok(value)
    }

    pub fn write(&mut self, buffer: Vec<u8>) -> Result<(), SendError<Vec<u8>>> {
        if buffer.len() == 0 {
            // We're done, so we drop the input_tx to signal the processing thread to stop.
            self.input_tx = None;
            return Ok(());
        }
        let tx = self.input_tx.clone().unwrap();
        tx.send(buffer)
    }
}
impl Finalize for AudioSnippetDetector {}
