use ndarray::{s, Array1, Array2};

use crate::mfcc::{MfccIter, N_FILTERS};

pub fn collect_to_array2(mfcc_stream1: MfccIter) -> Array2<f32> {
    // Collect the stream of 1D arrays into a vector
    let arrays: Vec<Array1<f32>> = mfcc_stream1.collect();

    // We assume each 1D array has length 20
    let num_rows = arrays.len();
    let num_cols = N_FILTERS;

    // Create a 2D array with (num_rows, num_cols)
    let mut result = Array2::<f32>::zeros((num_rows, num_cols));

    // Copy each 1D array into the corresponding row of the 2D array
    for (i, row) in arrays.into_iter().enumerate() {
        result.slice_mut(s![i, ..]).assign(&row);
    }

    result
}

/// Returns a stream of Array2<f32> windows of size `window_size`
/// with 50% overlap (that is, an overlap of `window_size / 2` items).
///
/// Each `Array1<f32>` in the input is assumed to have the same length.
pub struct OverlappingMfccStream {
    input_stream: MfccIter,
    window_size: usize,
    overlap: usize,
    buffer: Vec<Array1<f32>>,
}
impl OverlappingMfccStream {
    pub fn new(input_stream: MfccIter, window_size: usize) -> Self {
        let overlap = window_size / 2;
        Self {
            input_stream,
            window_size,
            overlap,
            buffer: Vec::new(),
        }
    }
}
impl Iterator for OverlappingMfccStream {
    type Item = Array2<f32>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.input_stream.next() {
            // Keep adding incoming frames into our buffer.
            self.buffer.push(item);

            // Once we have enough frames to form a full window, yield it.
            if self.buffer.len() >= self.window_size {
                // Extract the chunk for the window.
                let chunk = &self.buffer[0..self.window_size];

                // Convert chunk (Vec of Array1<f32>) to Array2<f32>.
                let num_rows = chunk.len();
                let num_cols = chunk[0].len();
                let mut result = Array2::<f32>::zeros((num_rows, num_cols));

                for (i, row) in chunk.iter().enumerate() {
                    result.slice_mut(s![i, ..]).assign(row);
                }

                // Remove the used portion minus the overlapping part.
                // For example, if window_size=20, overlap=10, we move the
                // window forward by 10 frames, leaving the last 10 frames
                // in the buffer for the next window.
                let drain_count = self.window_size - self.overlap;
                self.buffer.drain(0..drain_count);

                return Some(result);
            }
        }
        None
    }
}
