use ndarray::Array1;
use rustfft::Fft;
use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
// We need PI for the Hamming calculation
use std::f32::consts::PI;

const SAMPLE_RATE: f32 = 16_000.0; // 16 kHz
const WINDOW_S: f32 = 0.025; // 25ms
const WINDOW_SIZE: usize = (SAMPLE_RATE * WINDOW_S) as usize;
const FRAME_HOP_S: f32 = 0.010; // 10ms
const FRAME_HOP: usize = (SAMPLE_RATE * FRAME_HOP_S) as usize;
pub const N_FILTERS: usize = 20; // Number of Mel filter banks

pub struct MfccIter {
    input: Receiver<Vec<u8>>,
    input_pos: usize,
    input_bytes: Vec<u8>,
    ring_buffer: [f32; WINDOW_SIZE],
    index: usize,
    fft: Arc<dyn Fft<f32>>,
    hamming_window: Vec<f32>,
}
impl MfccIter {
    pub fn new(input: Receiver<Vec<u8>>) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(WINDOW_SIZE);
        let hamming_window = create_hamming_window(WINDOW_SIZE);
        Self {
            input,
            input_pos: 0,
            input_bytes: Vec::new(),
            ring_buffer: [0f32; WINDOW_SIZE],
            index: 0,
            fft,
            hamming_window,
        }
    }
}
impl Iterator for MfccIter {
    type Item = Array1<f32>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Ok(bytes) = self.input.recv() {
            self.input_bytes.extend_from_slice(&bytes);

            // println!("current input_bytes: {:?}", self.input_bytes);

            while self.input_bytes.len() - self.input_pos >= 2 {
                let sample_bytes = &self.input_bytes[..2];

                let sample_i16 = i16::from_le_bytes([sample_bytes[0], sample_bytes[1]]);
                // println!("read bytes: {:?}: {:?}", sample_bytes, sample_i16);
                self.input_bytes.drain(0..2);

                let normalized_sample = sample_i16 as f32 / 32768.0;
                self.ring_buffer[self.index] = normalized_sample;
                self.index = (self.index + 1) % WINDOW_SIZE;
                if self.index % FRAME_HOP != 0 {
                    continue;
                }
                let ordered_buffer = reorder_ring_buffer(&self.ring_buffer, self.index);
                let mfcc_vec = process_window(&self.fft, &self.hamming_window, &ordered_buffer);
                return Some(mfcc_vec);
            }
        }
        None
    }
}

/// Generate a Hamming window of length `size`.
/// Hamming window: w[n] = 0.54 - 0.46 * cos(2*pi*n/(size-1))
fn create_hamming_window(size: usize) -> Vec<f32> {
    let mut window = Vec::with_capacity(size);
    for n in 0..size {
        let value = 0.54 - 0.46 * (2.0 * PI * n as f32 / (size as f32 - 1.0)).cos();
        window.push(value);
    }
    window
}

fn reorder_ring_buffer(ring_buffer: &[f32; WINDOW_SIZE], write_index: usize) -> [f32; WINDOW_SIZE] {
    let mut temp = [0f32; WINDOW_SIZE];

    // The oldest sample is at `write_index` (because that's where new data
    // will overwrite next), so we copy ring_buffer in a wrapped fashion.
    //
    // For each i in 0..WINDOW_SIZE:
    //   temp[i] = ring_buffer[(write_index + i) % WINDOW_SIZE]
    // This way, temp[0] is the oldest, and temp[WINDOW_SIZE-1] is the newest.
    for i in 0..WINDOW_SIZE {
        temp[i] = ring_buffer[(write_index + i) % WINDOW_SIZE];
    }
    temp
}

fn hz_to_mel(hz: f32) -> f32 {
    // Common formula:
    2595.0 * (1.0 + hz / 700.0).log10()
}

fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10_f32.powf(mel / 2595.0) - 1.0)
}

fn compute_mel_filter_banks(power_spectrum: &[f32], n_filters: usize) -> Vec<f32> {
    let fft_size = power_spectrum.len();
    let nyquist = SAMPLE_RATE / 2.0;

    // 1) Mel points (n_filters + 2 for edges at 0 and nyquist)
    let mel_min = hz_to_mel(0.0);
    let mel_max = hz_to_mel(nyquist);
    let mel_step = (mel_max - mel_min) / (n_filters + 1) as f32;

    let mel_points: Vec<f32> = (0..n_filters + 2)
        .map(|i| mel_min + (i as f32 * mel_step))
        .collect();

    // 2) Convert Mel points back to Hz, then to FFT bin indices
    //    (integer indices in [0..fft_size/2])
    let bin_points: Vec<usize> = mel_points
        .iter()
        .map(|&m| {
            let freq = mel_to_hz(m);
            // Map frequency to FFT bin index: bin = freq * (fft_size / sample_rate)
            let bin = (freq / (SAMPLE_RATE) * fft_size as f32).floor() as usize;
            bin.min(fft_size - 1) // clamp to valid range
        })
        .collect();

    // 3) Build each triangular filter and accumulate energy
    let mut mel_energies = vec![0.0_f32; n_filters];

    for m in 0..n_filters {
        let start = bin_points[m];
        let peak = bin_points[m + 1];
        let end = bin_points[m + 2];

        // For bins [start..peak]
        for k in start..peak {
            let weight = (k as f32 - start as f32) / (peak as f32 - start as f32);
            mel_energies[m] += power_spectrum[k] * weight;
        }
        // For bins [peak..end]
        for k in peak..=end {
            let weight = (end as f32 - k as f32) / (end as f32 - peak as f32);
            mel_energies[m] += power_spectrum[k] * weight;
        }
    }

    mel_energies
}

fn to_log_scale(mel_energies: &mut [f32]) {
    for val in mel_energies.iter_mut() {
        *val = (*val + 1e-10).ln();
        // *val = 20.0 * (*val + 1e-10).log10()
        // or use 20.0 * (val + 1e-10).log10() for dB
    }
}

fn dct_type2(input: &[f32], output: &mut Array1<f32>) {
    // DCT-II formula:
    // X[k] = sum_{n=0..N-1} x[n] * cos( pi/N * (n + 0.5) * k ), for k in [0..N-1]
    // We'll ignore scale factors for simplicity.
    let n = input.len() as f32;
    let n_int = input.len();

    for k in 0..n_int {
        let mut sum = 0.0;
        for (n_idx, &x_n) in input.iter().enumerate() {
            let angle = std::f32::consts::PI / n * (n_idx as f32 + 0.5) * (k as f32);
            sum += x_n * angle.cos();
        }
        output[k] = sum;
    }
}

fn process_window(
    fft: &Arc<dyn Fft<f32>>,
    hamming_window: &[f32],
    window: &[f32; WINDOW_SIZE],
) -> Array1<f32> {
    // 1) Convert our buffer to complex floats (with hamming window applied)
    let mut buffer: Vec<Complex<f32>> = window
        .iter()
        .zip(hamming_window)
        .map(|(&sample, &ham)| Complex::new(sample * ham, 0.0))
        .collect();

    // 2) Perform the FFT
    fft.process(&mut buffer);

    // 3) Convert to power spectrum
    let power_spectrum: Vec<f32> = buffer.iter().map(|val| val.norm_sqr()).collect();

    // 4) Compute Mel filter banks
    let mut mel_energies = compute_mel_filter_banks(&power_spectrum, N_FILTERS);

    // 5) Convert energies to log scale
    to_log_scale(&mut mel_energies);

    // 6) Perform DCT to get MFCC-like coefficients
    let mut mfcc_out = Array1::zeros(N_FILTERS); // array![0.0f32; N_FILTERS]; // typically you might want fewer than all 20
    dct_type2(&mel_energies, &mut mfcc_out);

    // 7) Print the MFCC coefficients
    // println!("{:?}", mfcc_out);

    // let top_freqs = top_n_frequencies(&buffer, 16000.0, 10);
    // println!("--- FFT Magnitudes (first 10 bins) ---");
    // for (freq, mag) in top_freqs {
    //     println!("freq: {:.2} Hz, magnitude: {:.2}", freq, mag);
    // }
    // println!("--------------------------------------");
    mfcc_out
}
