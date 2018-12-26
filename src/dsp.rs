pub use frequency::Freq;
pub use frequency::Rate;
use err;
use filters;

use num::Integer; // For u32.gcd(u32)

pub type Signal = Vec<f32>;

/// Get biggest sample in signal.
pub fn get_max(vector: &Signal) -> err::Result<&f32> {
    if vector.len() == 0 {
        return Err(err::Error::Internal(
            "Can't get maximum of a zero length vector".to_string()));
    }

    let mut max: &f32 = &vector[0];
    for sample in vector {
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
    for sample in vector {
        if sample < min {
            min = sample;
        }
    }

    Ok(min)
}

/// Filter and then resample.
///
/// Does both things at the same time, so it's faster than calling filter() and
/// then resampling. Make sure that the filter prevents aliasing.
///
/// Takes a &Signal, the input rate, the output rate and the filter to use.
pub fn resample_with_filter(
    signal: &Signal,
    input_rate: Rate,
    output_rate: Rate,
    mut filt: impl filters::Filter,
) -> err::Result<Signal> {

    if output_rate.get_hz() == 0 {
        return Err(err::Error::Internal("Can't resample to 0Hz".to_string()));
    }

    let gcd = input_rate.get_hz().gcd(&output_rate.get_hz());
    let l = output_rate.get_hz() / gcd; // interpolation factor
    let m = input_rate.get_hz() / gcd; // decimation factor


    if l > 1 { // If we need interpolation
        // Reference the frequencies to the rate we have after interpolation
        filt.resample(input_rate, input_rate * l);
        let coeff = filt.design();

        Ok(fast_resampling(&signal, l, m, &coeff))
    } else {
        Ok(decimate(&filter(&signal, filt), m))
    }
}

/// Resample signal.
///
/// Takes a &Signal, the input rate, the output rate and the transition band of
/// the lowpass filter to use.
pub fn resample(
    signal: &Signal,
    input_rate: Rate,
    output_rate: Rate,
    delta_w: Freq,
) -> err::Result<Signal> {

    resample_with_filter(&signal, input_rate, output_rate,
        filters::Lowpass {
            cutout: Freq::pi_rad(1.),
            atten: 40.,
            delta_w: delta_w
        }
    )
}

/// Resample a signal using a given filter.
///
/// Resamples by expansion by l, filtering and then decimation by m. The
/// expansion is equivalent to the insertion of l-1 zeros between samples.
///
/// The given filter coefficients should be designed for the signal after
/// expansion by l, so you might want to divide every frequency by l when
/// designing the filter.
fn fast_resampling(signal: &Signal, l: u32, m: u32, coeff: &Signal) -> Signal {

    let l = l as usize;
    let m = m as usize;

    // This is the resampling algorithm, i've tried to make it faster
    // several times, that's why it's so ugly

    // Check the diagram on the documentation to see what the letters mean

    debug!("Resampling by L/M: {}/{}", l, m);

    let mut output: Signal = Vec::with_capacity(signal.len() * l / m);

    let offset = (coeff.len() - 1) / 2; // Filter delay in the n axis, half
                                         // of filter width

    let mut n: usize; // Current working n

    let mut t: usize = offset; // Like n but fixed to the current output
                               // sample to calculate

    // Iterate over each output sample
    while t < signal.len() * l {

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
        let mut x = n / l; // Current input sample
        while n <= t + offset {
            // Check if there is a sample in that index, in case that we
            // use an index bigger that signal.len()
            match signal.get(x) {
                // n+offset-t is equal to j
                Some(sample) => sum += coeff[n + offset - t] * sample,
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
}

/// Decimate without filtering
///
/// The signal should be accordingly bandlimited previously to avoid aliasing.
fn decimate(signal: &Signal, m: u32) -> Signal {

    let m = m as usize;

    debug!("Resampling by decimation, M: {}", m);

    let mut decimated: Signal = Vec::with_capacity(signal.len() / m);

    for i in 0..signal.len() / m {
        decimated.push(signal[i * m]);
    }

    debug!("Resampling finished");
    decimated

}

/*
/// Resample signal to given rate.
///
/// `cutout` is the cutout frequency of the lowpass filter, when None uses 1
/// radians per second to prevent aliasing on decimation.
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
    let delta_w = Freq::pi_rad(0.2);
    // TODO: check

    Ok(resample(&signal, l, m, cutout, atten, delta_w))
}
*/

/*
/// Resample signal by L/M following specific parameters.
///
/// `l` is the interpolation factor and `m` is the decimation one. The filter
/// is designed by a Kaiser window method depending in the attenuation `atten`
/// and the transition band width `delta_w`.
///
/// `cutout` is the cutout frequency of the lowpass filter, when None uses 1
/// radians per second to prevent aliasing on decimation.
///
/// `atten` should be positive and specified in decibels. `delta_w` is the
/// transition band.
pub fn resample(signal: &Signal, l: u32, m: u32, cutout: Option<Freq>,
                atten: f32, delta_w: Freq) -> Signal {

    let l = l as usize;
    let m = m as usize;

    // Divide by l to reference the frequencies to the rate we have after interpolation
    let cutout = match cutout {
        Some(x) => x / l,
        None => Freq::pi_rad(1.) / l,
    };
    let delta_w = delta_w / l;

    if l > 1 { // If we need interpolation

        // This is the resampling algorithm, i've tried to make it faster
        // several times, that's why it's so ugly

        // Check the image I made to see what the letters mean

        debug!("Resampling by L/M: {}/{}", l, m);

        let mut output: Signal = Vec::with_capacity(signal.len() * l / m);

        let f = lowpass_dc_removal(cutout, atten, delta_w); // filter coefficients

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
*/

/// Demodulate AM signal.
pub fn demodulate(signal: &Signal, carrier_freq: Freq) -> Signal {

    debug!("Demodulating signal");

    let mut output: Signal = vec![0_f32; signal.len()];

    // Demodulate from two consecutive samples, by the calculation of:
    // y[i] = sqrt(x[i-1]^2 + x[i]^2 - x[i-1]*x[i]*2*cos(phi)) / sin(phi)
    // Where:
    // phi = 2 * pi * (carrier_freq / sampling_freq)
    //
    // Take a look at the documentation
    //
    // Taken from:
    // https://github.com/pietern/apt137/blob/master/decoder.c

    // Shortcut to 2 * pi * (carrier_freq.get_hz() / sample_rate.get_hz())
    let phi = 2. * carrier_freq.get_rad();

    let cosphi2 = phi.cos() * 2.;
    let sinphi = phi.sin();

    let mut curr;
    let mut curr_sq;
    let mut prev = signal[0];
    let mut prev_sq = signal[0].powi(2);
    for i in 1..signal.len() {
        curr = signal[i];
        curr_sq = signal[i].powi(2);

        output[i] = (prev_sq + curr_sq - (prev * curr * cosphi2)).sqrt() / sinphi;

        prev = curr;
        prev_sq = curr_sq;
    }

    debug!("Demodulation finished");

    output
}

/// Filter a signal,
pub fn filter(signal: &Signal, filter: impl filters::Filter) -> Signal {

    debug!("Filtering signal");

    let coeff = filter.design();
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
