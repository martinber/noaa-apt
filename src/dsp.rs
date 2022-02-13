//! Functions for digital signal processing.

use std::convert::TryFrom;

use gcd::Gcd;
use log::{debug, error};

pub use crate::frequency::Freq;
pub use crate::frequency::Rate;

use crate::context::{Context, Step};
use crate::err;
use crate::filters;

/// Represents a signal, it's just a `Vec<f32>`.
pub type Signal = Vec<f32>;

/// Get biggest sample in signal.
#[allow(dead_code)]
pub fn get_max(vector: &Signal) -> err::Result<&f32> {
    if vector.is_empty() {
        return Err(err::Error::Internal(
            "Can't get maximum of a zero length vector".to_string(),
        ));
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
#[allow(dead_code)]
pub fn get_min(vector: &Signal) -> err::Result<&f32> {
    if vector.is_empty() {
        return Err(err::Error::Internal(
            "Can't get minimum of a zero length vector".to_string(),
        ));
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
/// Does both things at the same time, so it's faster than calling `filter()`
/// and then resampling. Make sure that the filter prevents aliasing.
///
/// The filter should have the frequencies referenced to the `input_rate`.
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

    let gcd = input_rate.get_hz().gcd(output_rate.get_hz());
    let l = output_rate.get_hz() / gcd; // interpolation factor
    let m = input_rate.get_hz() / gcd; // decimation factor

    let result;

    if l > 1 {
        // If we need interpolation
        // Reference the frequencies to the rate we have after interpolation
        let interpolated_rate = input_rate.checked_mul(l).ok_or_else(|| {
            err::Error::RateOverflow(format!(
                "Can't resample, looks like the sample rates do not have a big
                divisor in common. input_rate: {}, output_rate: {}, l: {}, m: {}",
                input_rate.get_hz(),
                output_rate.get_hz(),
                l,
                m
            ))
        })?;

        filt.resample(input_rate, interpolated_rate);
        let coeff = filt.design();

        context.step(Step::filter("resample_filter", &coeff))?;

        result = fast_resampling(context, signal, l, m, &coeff, input_rate)?;

        context.step(Step::signal(
            "resample_decimated",
            &result,
            Some(output_rate),
        ))?;
    } else {
        context.step(Step::filter("resample_filter", &filt.design()))?;

        let filtered = &filter(context, signal, filt)?;

        context.step(Step::signal(
            "resample_filtered",
            filtered,
            Some(input_rate),
        ))?;

        result = decimate(filtered, m);

        context.step(Step::signal(
            "resample_decimated",
            &result,
            Some(output_rate),
        ))?;
    }

    Ok(result)
}

/// Resample signal.
///
/// `delta_w` is the transition band of the lowpass filter to use. `atten` is
/// attenuation on positive decibels.
pub fn resample(
    context: &mut Context,
    signal: &Signal,
    input_rate: Rate,
    output_rate: Rate,
    atten: f32,
    delta_w: Freq,
) -> err::Result<Signal> {
    let cutout = if output_rate > input_rate {
        // Filter everything outside the original spectrum, so everything higher
        // than input_rate/2.
        // It's the same as Freq::pi_rad(1.).
        Freq::hz(input_rate.get_hz() as f32 / 2., input_rate)
    } else {
        // We have to filter everything that is not going to fit on the new
        // sample rate. So filter everything higher than output_rate/2.
        Freq::hz(output_rate.get_hz() as f32 / 2., input_rate)
    };

    resample_with_filter(
        context,
        signal,
        input_rate,
        output_rate,
        filters::Lowpass {
            cutout,
            atten,
            delta_w,
        },
    )
}

/// Resample a signal using a given filter.
///
/// Low-level function used by `resample_with_filter`.
///
/// I expect you to read this while looking at the diagram on the documentation
/// to figure out what the letters mean.
///
/// Resamples by expansion by `l`, filtering and then decimation by `m`. The
/// expansion is equivalent to the insertion of `l-1` zeros between samples.
///
/// The given filter coefficients should be designed for the signal after
/// expansion by `l`, so you might want to divide every frequency by `l` when
/// designing the filter.
///
/// I've tried to make it faster several times, that's why it's so ugly. It's
/// much more efficient than expanding, filtering and decimating, because skips
/// computed values that otherwise would be dropped on decimation.
///
/// Should be careful because it's easy to overflow usize when on 32 bits
/// systems. Specifically the variables that can overflow are:
/// `interpolated_len`, `n`, `t`.
#[allow(clippy::many_single_char_names)]
fn fast_resampling(
    context: &mut Context,
    signal: &Signal,
    l: u32,
    m: u32,
    coeff: &Signal,
    input_rate: Rate,
) -> err::Result<Signal> {
    let l = l as u64;
    let m = m as u64;

    // Check the diagram on the documentation to see what the letters mean

    debug!("Resampling by L/M: {}/{}", l, m);

    // Length that the interpolated signal should have, as u64 because this can
    // easily overflow if usize is 32 bits long
    let interpolated_len: u64 = signal.len() as u64 * l;

    // Length of the output signal, this should fit in 32 bits anyway.
    let output_len: u64 = interpolated_len / m;

    let mut output: Signal = Vec::with_capacity(output_len as usize);

    // Save expanded and filtered signal if we need to export that step
    let mut expanded_filtered = if context.export_resample_filtered {
        // Very likely to overflow usize on 32 bits systems
        match usize::try_from(interpolated_len) {
            Ok(l) => Vec::with_capacity(l),
            Err(_) => {
                error!("Expanded filtered signal can't fit in memory, skipping step");
                context.export_resample_filtered = false;
                Vec::new() // Not going to be used
            }
        }
    } else {
        Vec::new() // Not going to be used
    };

    // Filter delay in the n axis, half of filter width
    let offset: u64 = (coeff.len() as u64 - 1) / 2;

    let mut n: u64; // Current working n

    let mut t: u64 = offset; // Like n but fixed to the current output
                             // sample to calculate

    // Iterate over each output sample
    while t < interpolated_len {
        // Find first n inside the window that has a input sample that I
        // should multiply with a filter coefficient
        if t > offset {
            n = t - offset; // Go to n at start of filter
            match n % l {
                // Jump to first sample in window
                0 => (),
                rem => n += l - rem, // I checked this on pen and paper once and
                                     // forgot how it works
            }
        } else {
            // In this case the first sample in window is located at 0
            n = 0;
        }

        // Loop over all n inside the window with input samples and
        // calculate products
        let mut sum = 0.;
        let mut x = n / l; // Current input sample
        while n <= t + offset {
            // Check if there is a sample in that index, in case that we
            // use an index bigger that signal.len()
            if let Some(sample) = signal.get(x as usize) {
                // n+offset-t is equal to j
                sum += coeff[(n + offset - t) as usize] * sample;
            }
            x += 1;
            n += l;
        }

        if context.export_resample_filtered {
            // Iterate over every sample on the n axis, inefficient because we
            // only need to push to `output` the samples that would survive
            // a decimation.
            expanded_filtered.push(sum);
            t += 1;
            if t % m == 0 {
                output.push(sum);
            }
        } else {
            // Iterate over the samples that would survive a decimation.
            output.push(sum);
            t += m; // Jump to next output sample
        }
    }

    context.step(Step::signal(
        "resample_filtered",
        &expanded_filtered,
        Some(input_rate * l as u32),
    ))?;

    debug!("Resampling finished");
    Ok(output)
}

/// Decimate without filtering.
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
///
/// Demodulate from two consecutive samples, by the calculation of:
///
/// ```
/// y[i] = sqrt(x[i-1]^2 + x[i]^2 - x[i-1]*x[i]*2*cos(phi)) / sin(phi)
/// ```
///
/// Where:
///
/// ```
/// phi = 2 * pi * (carrier_freq / sampling_freq)
/// ```
///
/// Take a look at the documentation.
///
/// Expression taken from
/// [apt137](https://github.com/pietern/apt137/blob/master/decoder.c).
///
/// MIT License
///
/// Copyright (c) 2015 Pieter Noordhuis, Martin Bernardi
///
/// Permission is hereby granted, free of charge, to any person obtaining a copy
/// of this software and associated documentation files (the "Software"), to deal
/// in the Software without restriction, including without limitation the rights
/// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
/// copies of the Software, and to permit persons to whom the Software is
/// furnished to do so, subject to the following conditions:
///
/// The above copyright notice and this permission notice shall be included in all
/// copies or substantial portions of the Software.
///
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
/// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
/// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
/// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
/// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
/// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
/// SOFTWARE.

pub fn demodulate(
    context: &mut Context,
    signal: &Signal,
    carrier_freq: Freq,
) -> err::Result<Signal> {
    debug!("Demodulating signal");

    let mut output: Signal = vec![0_f32; signal.len()];

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

/// Filter a signal.
pub fn filter(
    context: &mut Context,
    signal: &Signal,
    filter: impl filters::Filter,
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

#[cfg(test)]
mod tests {

    use super::*;

    /// Check that when we use strange resampling rates, the greatest common
    /// divisor between them can be too small and the calculated interpolated
    /// rate can overflow.
    #[test]
    fn test_rate_overflow() {
        let result = resample_with_filter(
            &mut Context::resample(|_, _| {}, false, false), // Dummy context, not important
            &vec![0.0; 1000],
            Rate::hz(99371), // Two primes as sample rates
            Rate::hz(93911),
            filters::NoFilter,
        );

        if let Err(err::Error::RateOverflow(_)) = result {
        } else {
            panic!();
        }
    }

    /// Check a simple resample using `fast_resampling()`.
    ///
    /// I'm checking only for overflows, not checking if the resample is
    /// actually good.
    #[test]
    fn test_fast_resampling() {
        let result = fast_resampling(
            &mut Context::resample(|_, _| {}, false, false), // Dummy context, not important
            &vec![0.0; 1000],                                // signal
            3,                                               // l
            2,                                               // m
            &vec![0.0; 100],                                 // coeff
            Rate::hz(1000),                                  // input_rate
        );
        assert!(result.is_ok());
    }

    /// Check `fast_resampling()` when the coeffs are longer than the signal.
    ///
    /// I'm checking only for overflows, not checking if the resample is
    /// actually good.
    #[test]
    fn test_fast_resampling_short() {
        let result = fast_resampling(
            &mut Context::resample(|_, _| {}, false, false), // Dummy context, not important
            &vec![0.0; 100],                                 // signal
            3,                                               // l
            2,                                               // m
            &vec![0.0; 1000],                                // coeff
            Rate::hz(1000),                                  // input_rate
        );
        assert!(result.is_ok());
    }
}
