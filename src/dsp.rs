use std::f32::consts::PI;

use rustfft::FFTplanner;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

pub type Sample = f32; // f32 or f64
pub type Signal = Vec<Sample>;
pub type ComplexSignal = Vec<Complex<Sample>>;

/// Get biggest sample.
pub fn get_max(vector: &Signal) -> &Sample {
    let mut max: &Sample = &0_f32;
    for sample in vector.iter() {
        if sample > max {
            max = sample;
        }
    }

    max
}

/// Resample signal by upsampling, filtering and downsampling.
///
/// L is the interpolation factor and M the decimation one.
pub fn resample(signal: &Signal, l: u8, m: u8) -> Signal {
    let l = l as usize;
    let m = m as usize;
    let mut upsampled: Signal = vec![0_f32; signal.len() * l];

    for (i, sample) in signal.iter().enumerate() {
        upsampled[i * l] = *sample;
    }

    // filter(&upsampled, &vec![0.3, 0.6, 0.3])
    signal.clone()
}

/*
pub fn filter(signal: &Signal, coeff: &Signal) -> Signal {

    let mut output: Signal = vec![0_f32; signal.len()];

    for i in 0..signal.len() {
        let mut sum: Sample = 0_f32;
        for j in 0..coeff.len() {
            if i > j {
                sum += signal[i - j] * coeff[j];
            }
        }
        output[i] = sum;
    }
    output
}

pub fn kaiser() {
    // Atenuacion en dB
    let atten: f32 = 20; // 0.1
    // Banda de transicion en rad/s
    let delta_w: f32 = PI/8;
    // Ancho de banda
    let b = PI/4;

    let beta: f32;
    if atten > 50 {
        beta = 0.1102 * (atten - 8.7);
    } else if atten < 21 {
        beta = 0;
    } else {
        beta = 0.5842 * powf(atten - 21, 0.4) + 0.07886 * (atten - 21);
    }

    // Largo del filtro
    let n = (atten - 8) / (2.285 * delta_w) + 1;
}
*/


/// Hacer todo
pub fn process(signal: &Signal, sample_rate: &u32) -> Signal {
    let l = 7;
    let m = 2;
    let mut upsampled: Signal = vec![0_f32; signal.len() * l];

    for (i, sample) in signal.iter().enumerate() {
        upsampled[i * l] = *sample;
    }

    let mut input: ComplexSignal = vec![
            Complex::new(1_f32, 0_f32),
            Complex::new(2_f32, 0_f32),
            Complex::new(3_f32, 0_f32),
            Complex::new(4_f32, 0_f32)];
    let mut output: ComplexSignal = vec![Complex::zero(); 4];
    let mut output2: ComplexSignal = vec![Complex::zero(); 4];

    let mut planner = FFTplanner::new(false);
    let fft = planner.plan_fft(4);
    fft.process(&mut input, &mut output);
    println!("{:?}", output);

    let mut iplanner = FFTplanner::new(true);
    let ifft = iplanner.plan_fft(4);
    ifft.process(&mut output, &mut output2);
    println!("{:?}", output2);

    signal.clone()
}
