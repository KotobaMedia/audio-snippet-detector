use std::{
    cell::{Cell, RefCell},
    io::{BufReader, Cursor, Seek, Write},
    rc::Rc,
    sync::{
        mpsc::{channel, Receiver, RecvError, SendError, Sender},
        Arc, Mutex, RwLock,
    },
    thread,
};

use neon::types::Finalize;

use crate::{db, mfcc::MfccIter, util};

#[derive(Clone)]
pub struct AudioSnippetDetector {
    pub db: Arc<Mutex<db::Database>>,

    input_tx: Arc<RwLock<Option<Sender<Vec<u8>>>>>,
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
            let mfcc_stream = MfccIter::new(input_rx);
            let mut overlapping_stream = util::OverlappingMfccStream::new(mfcc_stream, 100);
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
        });

        Self {
            db,
            input_tx: Arc::new(RwLock::new(Some(input_tx))),
            output_rx: Arc::new(Mutex::new(output_rx)),
        }
    }

    pub fn next(&self) -> Result<String, RecvError> {
        let rx = self.output_rx.lock().unwrap();
        let value = rx.recv()?;
        Ok(value)
    }

    pub fn write(&self, buffer: Vec<u8>) -> Result<(), SendError<Vec<u8>>> {
        let tx = self.input_tx.read().unwrap();
        match tx.as_ref() {
            None => Err(SendError(buffer)),
            Some(tx) => tx.send(buffer),
        }
    }

    pub fn close(&self) {
        let mut tx = self.input_tx.write().unwrap();
        *tx = None;
    }
}
impl Finalize for AudioSnippetDetector {}
