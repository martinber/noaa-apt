use std::f32::consts::PI;

// TODO: Optimizar cosas simétricas
// TODO: Pedir valores en fracciones de PI

pub type Signal = Vec<f32>;

/// Get biggest sample.
pub fn get_max(vector: &Signal) -> &f32 {
    let mut max: &f32 = &0_f32;
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
pub fn resample(signal: &Signal, l: u32, m: u32, atten: f32, delta_w: f32) -> Signal {

    let upsampled: Signal;
    let l = l as usize;
    let m = m as usize;

    if l > 1 {
        debug!("Upsampling by {}", l);

        let mut padded: Signal = vec![0_f32; signal.len() * l];

        for (i, sample) in signal.iter().enumerate() {
            padded[i * l] = *sample;
        }

        debug!("Filtering");

        let f = lowpass(1./(l as f32), atten, delta_w);

        upsampled = filter(&padded, &f);

    } else {

        upsampled = signal.clone();

    }

    debug!("Decimating");

    let mut decimated: Signal = Vec::with_capacity(upsampled.len() / m);

    for i in 0..upsampled.len()/m {
        decimated.push(upsampled[i*m]);
    }

    decimated
}

/// Demodulate AM signal.
pub fn demodulate(signal: &Signal, atten: f32, delta_w: f32) -> Signal {
    let h_filter = hilbert(atten, delta_w);
    let imag = filter(signal, &h_filter);
    let delay: usize = h_filter.len() / 2;

    let mut output: Signal = vec![0_f32; signal.len()];

    for i in 0..signal.len() {
        if i >= delay {
            output[i] = (imag[i].powi(2) + signal[i-delay].powi(2)).sqrt();
        }
    }

    output
}

/// Filter a signal,
pub fn filter(signal: &Signal, coeff: &Signal) -> Signal {

    let mut output: Signal = vec![0_f32; signal.len()];

    for i in 0..signal.len() {
        let mut sum: f32 = 0_f32;
        for j in 0..coeff.len() {
            if i > j {
                sum += signal[i - j] * coeff[j];
            }
        }
        output[i] = sum;
    }
    output
}

/// Product of two vectors, element by element.
pub fn product(mut v1: Signal, v2: &Signal) -> Signal {
    if v1.len() != v2.len() {
        panic!("Both vectors must have the same length");
    }

    for i in 0 .. v1.len() {
        v1[i] = v1[i] * v2[i];
    }

    v1
}

/// Get hilbert FIR filter, windowed by a kaiser window.
///
/// Frequency in fractions of pi radians per second.
/// Attenuation in positive decibels.
pub fn hilbert(atten: f32, delta_w: f32) -> Signal {

    let window = kaiser(atten, delta_w);

    if window.len() % 2 == 0 {
        panic!("Hilbert filter length should be odd");
    }

    let mut filter: Signal = Vec::with_capacity(window.len());

    let m = window.len() as i32;

    for n in -(m-1)/2 ..= (m-1)/2 {
        if n % 2 != 0 {
            let n = n as f32;
            filter.push(2./(PI*n));
        } else {
            filter.push(0.);
        }
    }

    product(filter, &window)
}

/// Get lowpass FIR filter, windowed by a kaiser window.
///
/// Frequency in fractions of pi radians per second.
/// Attenuation in positive decibels.
pub fn lowpass(cutout: f32, atten: f32, delta_w: f32) -> Signal {

    let window = kaiser(atten, delta_w);

    if window.len() % 2 == 0 {
        panic!("Lowpass filter length should be odd");
    }

    let mut filter: Signal = Vec::with_capacity(window.len());

    let m = window.len() as i32;

    for n in -(m-1)/2 ..= (m-1)/2 {
        if n == 0 {
            filter.push(cutout);
        } else {
            let n = n as f32;
            filter.push((n*PI*cutout).sin()/(n*PI));
        }
    }

    product(filter, &window)
}

/// Design Kaiser window from parameters.
///
/// The length depends on the parameters given, and it's always odd.
/// Frequency in fractions of pi radians per second.
fn kaiser(atten: f32, delta_w: f32) -> Signal {

    let beta: f32;
    if atten > 50. {
        beta = 0.1102 * (atten - 8.7);
    } else if atten < 21. {
        beta = 0.;
    } else {
        beta = 0.5842 * (atten - 21.).powf(0.4) + 0.07886 * (atten - 21.);
    }

    // Filter length, we want an odd length
    let mut length: i32 = ((atten - 8.) / (2.285 * PI*delta_w)).ceil() as i32 + 1;
    if length % 2 == 0 {
        length += 1;
    }

    let mut window: Signal = Vec::with_capacity(length as usize);

    use rgsl::bessel::I0 as bessel;
    for n in -(length-1)/2 ..= (length-1)/2 {
        let n = n as f32;
        let m = length as f32;
        window.push((bessel((beta * (1. - (n / (m/2.)).powi(2)).sqrt()) as f64) /
                    bessel(beta as f64)) as f32)
    }

    debug!("beta: {}, length: {}", beta, length);

    window
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Calculate absolute value of fft and divide each sample by n
    fn abs_fft(signal: &Signal) -> Signal {
        use rgsl;
        use rgsl::types::fast_fourier_transforms::FftComplexWaveTable;
        use rgsl::types::fast_fourier_transforms::FftComplexWorkspace;

        let mut data: Vec<f64> = Vec::with_capacity(signal.len() * 2);

        for s in signal.iter() {
            data.push(*s as f64);
            data.push(0.);
        }

        let wavetable = FftComplexWaveTable::new(signal.len()).unwrap();
        let mut workspace = FftComplexWorkspace::new(signal.len()).unwrap();

        rgsl::fft::mixed_radix::forward(
                &mut data, 1, signal.len(), &wavetable, &mut workspace);

        let mut result: Signal = Vec::with_capacity(signal.len());

        for i in 0 .. signal.len() {
            result.push(f64::sqrt(data[2*i].powi(2) + data[2*i+1].powi(2)) as f32);
        }

        result
    }

    /// Check if the filter meets the required parameters in the positive half
    /// of the spectrum.
    #[test]
    fn test_lowpass() {
        // cutout, atten and delta_w values
        let test_parameters: Vec<(f32, f32, f32)> = vec![
                (1./4., 20., 1./10.), (1./3., 35., 1./30.), (2./5., 60., 1./20.)];

        for parameters in test_parameters.iter() {
            let (cutout, atten, delta_w) = *parameters;

            let ripple = 10_f32.powf(-atten/20.); // 10^(-atten/20)

            let filter = lowpass(cutout, atten, delta_w);
            let mut fft = abs_fft(&filter);

            println!("cutout: {}, atten: {}, delta_w: {}", cutout, atten, delta_w);
            println!("filter: {:?}", filter);

            for (i, v) in fft.iter().enumerate() {
                let w = 2. * (i as f32) / (fft.len() as f32);

                if w < cutout - delta_w/2. {
                    println!("Passband, ripple: {}, v: {}, i: {}, w: {}", ripple, v, i, w);
                    assert!(*v < 1. + ripple && *v > 1. - ripple);
                }
                else if w > cutout + delta_w/2. && w < 1. {
                    println!("Stopband, ripple: {}, v: {}, i: {}, w: {}", ripple, v, i, w);
                    assert!(*v < ripple);
                }
            }
        }
    }

    /*
    /// Check if the filter meets the required parameters in the positive half
    /// of the spectrum.
    #[test]
    fn test_hilbert() {
        // atten and delta_w values
        let test_parameters: Vec<(f32, f32)> = vec![
                /*(20., 1./10.),*/ (35., 1./30.), (60., 1./20.)];

        for parameters in test_parameters.iter() {
            let (atten, delta_w) = *parameters;

            let ripple = 10_f32.powf(-atten/20.); // 10^(-atten/20)

            let filter = hilbert(atten, delta_w);
            let mut fft = abs_fft(&filter);

            println!("atten: {}, delta_w: {}", atten, delta_w);
            println!("filter: {:?}", filter);

            for (i, v) in fft.iter().enumerate() {
                let w = 2. * (i as f32) / (fft.len() as f32);

                if w > delta_w/2. && w < 1. - delta_w/2. {
                    println!("Passband, ripple: {}, v: {}, i: {}, w: {}", ripple, v, i, w);
                    assert!(*v < 1. + ripple && *v > 1. - ripple);
                }
            }
        }
    }
    */
}
