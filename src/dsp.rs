use std::f32::consts::PI;

// use rustfft::FFTplanner;
// use rustfft::num_complex::Complex;
// use rustfft::num_traits::Zero;

pub type Sample = f32; // f32 or f64
pub type Signal = Vec<Sample>;
// pub type ComplexSignal = Vec<Complex<Sample>>;

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

*/

/// Get hilbert FIR filter, windowed by a rectangular window.
///
/// Length must be odd.
pub fn get_hilbert(length: &u32, sample_rate: &u32) -> Signal {

    if length % 2 == 0 {
        panic!("Hilbert filter length must be odd");
    }

    let mut filter: Signal = Vec::with_capacity(*length as usize);

    let M = *length as i32;

    for n in -(M-1)/2 ..= (M-1)/2 {
        if n % 2 == 1 {
            let sample_rate = *sample_rate as Sample;
            let n = n as Sample;
            filter.push(sample_rate/(PI*n));
        } else {
            filter.push(0 as Sample);
        }
    }

    filter
}

/// Design Kaiser window from parameters.
///
/// The length depends on the parameters given, and it's always odd.
pub fn kaiser(atten: &Sample, delta_w: &Sample) -> Signal {

    let beta: Sample;
    if atten > 50. {
        beta = 0.1102 * (atten - 8.7);
    } else if atten < 21. {
        beta = 0.;
    } else {
        beta = 0.5842 * (atten - 21.).powf(0.4) + 0.07886 * (atten - 21.);
    }

    // Filter length, we want an odd length
    let mut length: i32 = ((atten - 8.) / (2.285 * delta_w)).ceil() as i32 + 1;
    if length % 2 == 0 {
        length += 1;
    }

    let mut window: Signal = Vec::with_capacity(length as usize);

    use rgsl::bessel::I0 as bessel;
    for n in -(length-1)/2 ..= (length-1)/2 {
        println!("n: {}", n);
        let n = n as Sample;
        let M = length as Sample;
        window.push((bessel((beta * (1. - (n / (M/2.)).powi(2)).sqrt()) as f64) /
                    bessel(beta as f64)) as Sample)
    }

    println!("beta: {}, length: {}", beta, length);

    window
}


/*
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
            Complex::new(4_f32, 0_f32),
            Complex::new(5_f32, 0_f32)];
    let mut output: ComplexSignal = vec![Complex::zero(); 5];
    let mut output2: ComplexSignal = vec![Complex::zero(); 5];

    let mut planner = FFTplanner::new(false);
    let fft = planner.plan_fft(5);
    fft.process(&mut input, &mut output);
    println!("{:?}", output);

    let mut iplanner = FFTplanner::new(true);
    let ifft = iplanner.plan_fft(5);
    ifft.process(&mut output, &mut output2);
    println!("{:?}", output2);

    signal.clone()
}
*/
