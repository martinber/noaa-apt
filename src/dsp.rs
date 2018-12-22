pub use frequency::Freq;
pub use frequency::Rate;
use err;

use num::Integer; // For u32.gcd(u32)
use std::f32::consts::PI;

pub type Signal = Vec<f32>;

/// Get biggest sample in signal.
pub fn get_max(vector: &Signal) -> err::Result<&f32> {
    if vector.len() == 0 {
        return Err(err::Error::Internal(
                "Can't get maximum of a zero length vector".to_string()));
    }

    let mut max: &f32 = &vector[0];
    for sample in vector.iter() {
        if sample > max {
            max = sample;
        }
    }

    Ok(max)
}

/// Get smallest sample in signal.
pub fn get_min(vector: &Signal) -> err::Result<&f32> {
    if vector.len() == 0 {
        return Err(err::Error::Internal(
                "Can't get minimum of a zero length vector".to_string()));
    }

    let mut min: &f32 = &vector[0];
    for sample in vector.iter() {
        if sample < min {
            min = sample;
        }
    }

    Ok(min)
}

/// Resample signal to given rate.
///
/// `cutout` is the cutout frequency of the lowpass filter in fractions of pi
/// radians per second, has type Option<f32>, when None uses `1/l' to prevent
/// aliasing on decimation.
///
/// The filter has a transition band equal to the 20% of the spectrum width of
/// the input signal. Starts at 90% of the input signal spectrum, so it lets a
/// little of spectrum to go through.
///
/// The filter attenuation is 40dB.
pub fn resample_to(signal: &Signal, input_rate: Rate, output_rate: Rate,
                   cutout: Option<Freq>) -> err::Result<Signal> {

    if output_rate.get_hz() == 0 {
        return Err(err::Error::Internal("Can't resample to 0Hz".to_string()));
    }

    let gcd = input_rate.get_hz().gcd(&output_rate.get_hz());
    let l = output_rate.get_hz() / gcd; // interpolation factor
    let m = input_rate.get_hz() / gcd; // decimation factor

    let atten = 40.;
    let delta_w = Freq::pi_rad(0.2) / l;

    Ok(resample(&signal, l, m, cutout, atten, delta_w))
}


/// Resample signal by L/M following specific parameters.
///
/// `l` is the interpolation factor and `m` is the decimation one. The filter
/// is designed by a Kaiser window method depending in the attenuation `atten`
/// and the transition band width `delta_w`.
///
/// `cutout` is the cutout frequency of the lowpass filter in fractions of pi
/// radians per second, has type Option<f32>, when None uses `1/l' to prevent
/// aliasing on decimation.
///
/// `atten` should be positive and specified in decibels. `delta_w` has units of
/// fractions of pi radians per second, considering the signal after `l - 1`
/// insertions of zeros.
pub fn resample(signal: &Signal, l: u32, m: u32, cutout: Option<Freq>,
                atten: f32, delta_w: Freq) -> Signal {

    let l = l as usize;
    let m = m as usize;

    let cutout = match cutout {
        Some(x) => x,
        None => Freq::pi_rad(1.) / l,
    };

    if l > 1 { // If we need interpolation

        // This is the resampling algorithm, i've tried to make it faster
        // several times, that's why it's so ugly

        // Check the image I made to see what the letters mean

        debug!("Resampling by L/M: {}/{}", l, m);

        let mut output: Signal = Vec::with_capacity(signal.len() * l / m);

        let f = lowpass(cutout, atten, delta_w); // filter coefficients

        let offset = (f.len()-1)/2; // Filter delay in the n axis, half of
                                    // filter width

        let mut n: usize; // Current working n

        let mut t: usize = offset; // Like n but fixed to the current output
                                   // sample to calculate

        // Iterate over each output sample
        while t < signal.len()*l {

            // Find first n inside the window that has a input sample that I
            // should multiply with a filter coefficient
            if t > offset {
                n = t - offset; // Go to n at start of filter
                match n % l { // Jump to first sample in window
                    0 => (),
                    x => n += l - x, // I checked this on paper once and forgot
                                     // how it works
                }
            } else { // In this case the first sample in window is located at 0
                n = 0;
            }

            // Loop over all n inside the window with input samples and
            // calculate products
            let mut sum = 0.;
            let mut x = n/l; // Current input sample
            while n <= t + offset {
                // Check if there is a sample in that index, in case that we
                // use an index bigger that signal.len()
                match signal.get(x) {
                    // n+offset-t is equal to j
                    Some(sample) => sum += f[n+offset-t] * sample,
                    None => (),
                }
                x += 1;
                n += l;
            }
            output.push(sum); // Store output sample

            t += m; // Jump to next output sample
        }

        debug!("Resampling finished");
        output

    } else {

        // TODO: Check if wee need a filter

        debug!("Resampling by decimation, L/M: {}/{}", l, m);

        let mut decimated: Signal = Vec::with_capacity(signal.len() / m);

        for i in 0..signal.len()/m {
            decimated.push(signal[i*m]);
        }

        debug!("Resampling finished");
        decimated

    }

}

/// Demodulate AM signal.
pub fn demodulate(signal: &Signal, sample_rate: Rate, carrier_freq: Freq) -> Signal {

    debug!("Demodulating signal");

    let mut output: Signal = vec![0_f32; signal.len()];

    // Demodulate from two consecutive samples, by the calculation of:
    // y[i] = sqrt(x[i-1]^2 + x[i]^2 - x[i-1]*x[i]*2*cos(phi)) / sin(phi)
    // Where:
    // phi = 2 * pi * (carrier_freq / sampling_freq)
    //
    // Take a look at the README
    //
    // Taken from:
    // https://github.com/pietern/apt137/blob/master/decoder.c

    let phi = 2. * (carrier_freq / sample_rate.get_hz()).get_rad();
    let cosphi2 = phi.cos() * 2.;
    let sinphi = phi.sin();

    let mut curr;
    let mut curr_sq;
    let mut prev = signal[0];
    let mut prev_sq = signal[0].powi(2);
    for i in 1..signal.len() {
        curr = signal[i];
        curr_sq = signal[i].powi(2);

        output[i] = (prev_sq + curr_sq - prev*curr*cosphi2).sqrt() / sinphi;

        prev = curr;
        prev_sq = curr_sq;
    }

    debug!("Demodulation finished");

    output
}



/// Filter a signal,
pub fn filter(signal: &Signal, coeff: &Signal) -> Signal {

    debug!("Filtering signal");

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
    debug!("Filtering finished");
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

/// Get lowpass FIR filter, windowed by a kaiser window.
///
/// Frequency in fractions of pi radians per second.
/// Attenuation in positive decibels.
pub fn lowpass(cutout: Freq, atten: f32, delta_w: Freq) -> Signal {

    debug!("Designing Lowpass filter, \
           cutout: pi*{}rad/s, attenuation: {}dB, delta_w: pi*{}rad/s",
           cutout.get_pi_rad(), atten, delta_w.get_pi_rad());

    let window = kaiser(atten, delta_w);

    if window.len() % 2 == 0 {
        panic!("Kaiser window length should be odd");
    }

    let mut filter: Signal = Vec::with_capacity(window.len());

    let m = window.len() as i32;

    for n in -(m-1)/2 ..= (m-1)/2 {
        if n == 0 {
            filter.push(cutout.get_pi_rad());
        } else {
            let n = n as f32;
            filter.push((n*PI*cutout.get_pi_rad()).sin()/(n*PI));
        }
    }

    debug!("Lowpass filter design finished");

    product(filter, &window)
}

/// Design Kaiser window from parameters.
///
/// The length depends on the parameters given, and it's always odd.
/// Frequency in fractions of pi radians per second.
fn kaiser(atten: f32, delta_w: Freq) -> Signal {

    debug!("Designing Kaiser window,\
           attenuation: {}dB, delta_w: pi*{}rad/s",
           atten, delta_w.get_pi_rad());

    let beta: f32;
    if atten > 50. {
        beta = 0.1102 * (atten - 8.7);
    } else if atten < 21. {
        beta = 0.;
    } else {
        beta = 0.5842 * (atten - 21.).powf(0.4) + 0.07886 * (atten - 21.);
    }

    // Filter length, we want an odd length
    let mut length: i32 = ((atten - 8.) / (2.285 * delta_w.get_rad())).ceil() as i32 + 1;
    if length % 2 == 0 {
        length += 1;
    }

    let mut window: Signal = Vec::with_capacity(length as usize);

    use misc::bessel_i0 as bessel;
    for n in -(length-1)/2 ..= (length-1)/2 {
        let n = n as f32;
        let m = length as f32;
        window.push(bessel(beta * (1. - (n / (m/2.)).powi(2)).sqrt()) /
                    bessel(beta))
    }

    debug!("Kaiser window design finished, beta: {}, length: {}", beta, length);

    window
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Calculate absolute value of fft
    fn abs_fft(signal: &Signal) -> Signal {
        use rustfft::FFTplanner;
        use rustfft::num_complex::Complex;
        use rustfft::num_traits::Zero;

        let mut input: Vec<Complex<f32>> = signal.iter()
                .map(|x| Complex::new(*x, 0.)).collect();

        let mut output: Vec<Complex<f32>> = vec![Complex::zero(); input.len()];

        let mut planner = FFTplanner::new(false); // inverse=false
        let fft = planner.plan_fft(input.len());
        fft.process(&mut input, &mut output);

        output.iter().map(|x| x.norm()).collect()
    }

    /// Check if two vectors of float are equal given some margin of precision
    fn vector_roughly_equal(a: &Vec<f32>, b: &Vec<f32>) -> bool {
        // Iterator with tuples of values to compare.
        // [(a1, b1), (a2, b2), (a3, b3), ...]
        let mut values = a.iter().zip(b.iter());
        // Check if every pair have similar values
        values.all(|(&a, &b)| ulps_eq!(a, b))
    }

    #[test]
    fn test_abs_fft() {
        // Checked with GNU Octave
        assert!(vector_roughly_equal(&abs_fft(&vec![1., 2., 3., 4.]),
                &vec![10., 2.828427124746190, 2., 2.828427124746190]));
        assert!(vector_roughly_equal(&abs_fft(&vec![1., 1., 1., 1., 1., 1., 1.]),
                &vec![7., 0., 0., 0., 0., 0., 0.]));
        assert!(vector_roughly_equal(&abs_fft(&vec![1., -1., 1., -1.]),
                &vec![0., 0., 4., 0.]));
    }

    #[test]
    fn test_lowpass() {
        // cutout, atten and delta_w values
        let test_parameters: Vec<(Freq, f32, Freq)> = vec![
                (Freq::pi_rad(1./4.), 20., Freq::pi_rad(1./10.)),
                (Freq::pi_rad(1./3.), 35., Freq::pi_rad(1./30.)),
                (Freq::pi_rad(2./5.), 60., Freq::pi_rad(1./20.))];

        for parameters in test_parameters.iter() {
            let (cutout, atten, delta_w) = *parameters;

            let ripple = 10_f32.powf(-atten/20.); // 10^(-atten/20)

            let filter = lowpass(cutout, atten, delta_w);
            let mut fft = abs_fft(&filter);

            println!("cutout: {}, atten: {}, delta_w: {}",
                     cutout.get_pi_rad(), atten, delta_w.get_pi_rad());
            println!("filter: {:?}", filter);

            for (i, v) in fft.iter().enumerate() {
                let w = Freq::pi_rad(2. * (i as f32) / (fft.len() as f32));

                if w < cutout - delta_w/2. {
                    println!("Passband, ripple: {}, v: {}, i: {}, w: {}", ripple, v, i, w.get_pi_rad());
                    assert!(*v < 1. + ripple && *v > 1. - ripple);
                }
                else if w > cutout + delta_w/2. && w < Freq::pi_rad(1.) {
                    println!("Stopband, ripple: {}, v: {}, i: {}, w: {}", ripple, v, i, w.get_pi_rad());
                    assert!(*v < ripple);
                }
            }
        }
    }
}
