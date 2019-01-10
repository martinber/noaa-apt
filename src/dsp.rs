pub use frequency::Freq;
pub use frequency::Rate;
use err;
use filters;
use context::{Context, Step};

use num::Integer; // For u32.gcd(u32)

pub type Signal = Vec<f32>;

/// Get biggest sample in signal.
pub fn get_max(vector: &Signal) -> err::Result<&f32> {
    if vector.is_empty() {
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
    if vector.is_empty() {
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
/// The filter should have the frequencies referenced to the input_rate.
///
/// Takes a &Signal, the input rate, the output rate and the filter to use.
pub fn resample_with_filter(
    context: &mut Context,
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

    let result;

    if l > 1 { // If we need interpolation
        // Reference the frequencies to the rate we have after interpolation
        filt.resample(input_rate, input_rate * l);
        let coeff = filt.design();

        context.step(Step::filter("resample_filter", &coeff))?;

        result = fast_resampling(context, &signal, l, m, &coeff, input_rate)?;

        context.step(Step::signal("resample_decimated", &result, Some(output_rate)))?;

    } else {

        context.step(Step::filter("resample_filter", &filt.design()))?;

        let filtered = &filter(context, &signal, filt)?;

        context.step(Step::signal("resample_filtered", &filtered, Some(input_rate)))?;

        result = decimate(filtered, m);

        context.step(Step::signal("resample_decimated", &result, Some(output_rate)))?;
    }

    Ok(result)
}

/// Resample signal.
///
/// Takes a &Signal, the input rate, the output rate and the transition band of
/// the lowpass filter to use.
pub fn resample(
    context: &mut Context,
    signal: &Signal,
    input_rate: Rate,
    output_rate: Rate,
    delta_w: Freq,
) -> err::Result<Signal> {

    // Just to prevent aliasing. We need the frequency referenced to the
    // input_rate.
    let cutout = Freq::hz(output_rate.get_hz() as f32 / 2., input_rate);

    resample_with_filter(context, &signal, input_rate, output_rate,
        filters::Lowpass {
            cutout,
            atten: 40.,
            delta_w,
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
fn fast_resampling(
    context: &mut Context,
    signal: &Signal,
    l: u32,
    m: u32,
    coeff: &Signal,
    input_rate: Rate,
) -> err::Result<Signal> {

    let l = l as usize;
    let m = m as usize;

    // This is the resampling algorithm, i've tried to make it faster
    // several times, that's why it's so ugly

    // Check the diagram on the documentation to see what the letters mean

    debug!("Resampling by L/M: {}/{}", l, m);

    let mut output: Signal = Vec::with_capacity(signal.len() * l / m);

    // Save expanded and filtered signal if wav-steps is enabled
    let mut expanded_filtered = if context.export {
        Vec::with_capacity(signal.len() * l)
    } else {
        Vec::with_capacity(0) // Not going to be used
    };

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
            if let Some(sample) = signal.get(x) {
                // n+offset-t is equal to j
                sum += coeff[n + offset - t] * sample;
            }
            x += 1;
            n += l;
        }

        if context.export_resample_filtered {
            expanded_filtered.push(sum);
            t += 1;
            if t % m == 0 {
                output.push(sum);
            }
        } else {
            output.push(sum); // Store output sample
            t += m; // Jump to next output sample
        }

    }

    context.step(Step::signal(
        "resample_filtered",
        &expanded_filtered,
        Some(input_rate * l as u32)
    ))?;

    debug!("Resampling finished");
    Ok(output)
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

/// Demodulate AM signal.
pub fn demodulate(
    context: &mut Context,
    signal: &Signal,
    carrier_freq: Freq
) -> err::Result<Signal> {

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

    context.step(Step::signal("demodulation_result", &output, None))?;
    Ok(output)
}

/// Filter a signal,
pub fn filter(
    context: &mut Context,
    signal: &Signal,
    filter: impl filters::Filter
) -> err::Result<Signal> {

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

    context.step(Step::filter("filter_filter", &coeff))?;
    context.step(Step::signal("filter_result", &output, None))?;
    Ok(output)
}
