//! High-level function for decoding APT.

use log::info;

use crate::config;
use crate::context::{Context, Step};
use crate::dsp::{self, Signal, Rate, Freq};
use crate::err;
use crate::filters;


/// Final signal sample rate.
///
/// This signal has one sample per pixel.
pub const FINAL_RATE: u32 = 4160;

/// Channel sync frame, in pixels.
pub const PX_SYNC_FRAME: u32 = 39;

/// Deep space data and minute markers.
pub const PX_SPACE_DATA: u32 = 47;

/// Channel image data.
pub const PX_CHANNEL_IMAGE_DATA: u32 = 909;

/// Telemetry data.
pub const PX_TELEMETRY_DATA: u32 = 45;

// Source: https://www.sigidwiki.com/wiki/Automatic_Picture_Transmission_(APT)#Structure
/// Pixels per channel.
pub const PX_PER_CHANNEL: u32 = 
    PX_SYNC_FRAME + 
    PX_SPACE_DATA + 
    PX_CHANNEL_IMAGE_DATA +
    PX_TELEMETRY_DATA;

/// Pixels per image row, 1040 * 2 = 2080.
pub const PX_PER_ROW: u32 = PX_PER_CHANNEL * 2;

/// AM carrier frequency in Hz.
pub const CARRIER_FREQ: u32 = 2400;


/// Decode APT image.
///
/// Returns raw image data, line by line.
pub fn decode(
    context: &mut Context,
    settings: &config::Settings,
    signal: &Signal,
    input_rate: Rate,
    sync: bool,
) -> err::Result<Signal>{

    // --------------------

    let final_rate = Rate::hz(FINAL_RATE);

    // Samples on each image row when at `WORK_RATE`.
    let samples_per_work_row: u32 = PX_PER_ROW * settings.work_rate / FINAL_RATE;

    let work_rate = Rate::hz(settings.work_rate);

    context.step(Step::signal("input", &signal, Some(input_rate)))?;

    // --------------------

    context.status(0.1, format!("Resampling to {}", work_rate.get_hz()));

    let filter = filters::LowpassDcRemoval {
        // Cutout frequency of the resampling filter, only the AM spectrum should go
        // through to avoid noise, 2 times the carrier frequency is enough
        cutout: Freq::hz(settings.resample_cutout, input_rate),

        atten: settings.resample_atten,

        // Width of transition band, we are using a DC removal filter that has a
        // transition band from zero to delta_w. I think that APT signals have
        // nothing below 500Hz.
        delta_w: Freq::hz(settings.resample_delta_freq, input_rate),
    };
    let signal = dsp::resample_with_filter(
        context, &signal, input_rate, work_rate, filter)?;

    if signal.len() < 10 * samples_per_work_row as usize {
        return Err(err::Error::Internal(
            "Got less than 10 rows of samples, audio file is too short".to_string()));
    }

    // --------------------

    context.status(0.4, "Demodulating".to_string());

    let signal = dsp::demodulate(
        context, &signal, Freq::hz(CARRIER_FREQ as f32, work_rate))?;

    // --------------------

    context.status(0.42, "Filtering".to_string());

    let cutout = Freq::pi_rad(FINAL_RATE as f32 / work_rate.get_hz() as f32);
    let filter = filters::Lowpass {
        cutout,
        atten: settings.demodulation_atten,
        delta_w: cutout / 5.
    };
    // mut because on sync the signal is going to be modified
    let mut signal = dsp::filter(context, &signal, filter)?;

    // --------------------

    if sync {
        context.status(0.5, "Syncing".to_string());

        // Get list of sync frames positions
        let sync_pos = find_sync(context, &signal, work_rate)?;

        if sync_pos.len() < 5 {
            return Err(err::Error::Internal(
                "Found less than 5 sync frames, audio file is too short or too \
                noisy".to_string())
            );
        }

        // Create new "aligned" vector to samples_per_work_row. Each row starts on
        // a found sync frame position
        let mut aligned: Signal = Vec::new();

        // For each sync position
        for i in 0..sync_pos.len()-1 {
            // Check if there are enough samples left to fill an image row
            if (sync_pos[i] + samples_per_work_row as usize) < signal.len() {

                aligned.extend_from_slice(
                    &signal[sync_pos[i] .. sync_pos[i] + samples_per_work_row as usize]
                );
            }
        }

        signal = aligned;

    } else {
        context.status(0.5, "Skipping Syncing".to_string());

        // If we are not syncing send a dummy correlation step
        context.step(Step::signal("sync_correlation", &vec![], Some(work_rate)))?;

        // Crop signal to multiple of samples_per_work_row
        let length = signal.len();
        signal.truncate(length
            / samples_per_work_row as usize // Integer division
            * samples_per_work_row as usize
        );
    }

    context.step(Step::signal("sync_result", &signal, Some(work_rate)))?;

    // --------------------

    context.status(0.90, "Resampling to 4160".to_string());

    // Resample without filter because we already filtered the signal before
    // syncing
    let signal = dsp::resample_with_filter(
        context, &signal, work_rate, final_rate, filters::NoFilter)?;

    Ok(signal)
}

/// Generate sample sync frame.
///
/// Generates sync A frame, a square wave which has a pulse width of 2 pixels,
/// and period of 4 pixels. Only has values 0 and 1.
///
/// Used for cross correlation against the received signal to find the sync
/// frames positions.
fn generate_sync_frame(work_rate: Rate) -> err::Result<Vec<i8>> {

    if work_rate.get_hz() % FINAL_RATE != 0 {
        return Err(err::Error::Internal(
            "work_rate is not multiple of FINAL_RATE".to_string()));
    }

    // Width of pixels at the work_rate.
    let pixel_width = (work_rate.get_hz() / FINAL_RATE) as usize;

    // Width of pulses at work_rate
    let sync_pulse_width = pixel_width * 2;

    // Tried to use iterators, it's horrible

    use std::iter::repeat;

    Ok(
        (
            repeat(-1).take(sync_pulse_width).chain(
            repeat(1).take(sync_pulse_width))
            .cycle().take(7 * 2 * sync_pulse_width)
        ).chain(
        repeat(-1).take(8 * pixel_width)).collect()
    )
}

/// Find sync frame positions.
///
/// Returns list of found sync frames positions.
fn find_sync(
    context: &mut Context,
    signal: &Signal,
    work_rate: Rate
) -> err::Result<Vec<usize>> {

    let guard = generate_sync_frame(work_rate)?;

    // list of maximum correlations found: (index, value)
    let mut peaks: Vec<(usize, f32)> = Vec::new();
    peaks.push((0, 0.));

    // Samples on each image row when at `WORK_RATE`.
    let samples_per_work_row: u32 = PX_PER_ROW * work_rate.get_hz() / FINAL_RATE;

    // Minimum distance between peaks, some arbitrary number smaller but close
    // to the number of samples by line
    let min_distance: usize = samples_per_work_row as usize * 8/10;

    // Save cross-correlation if exporting steps
    let mut correlation = if context.export_steps {
        Vec::with_capacity(signal.len() - guard.len())
    } else {
        Vec::with_capacity(0) // Not going to be used
    };

    for i in 0 .. signal.len() - guard.len() {
        let mut corr: f32 = 0.;
        for j in 0..guard.len() {
            match guard[j] {
                1 => corr += signal[i + j],
                -1 => corr -= signal[i + j],
                _ => unreachable!(),
            }
        }

        if context.export_steps {
            correlation.push(corr);
        }

        // If previous peak is too far, keep it and add this value to the
        // list as a new peak
        if i - peaks.last().unwrap().0 > min_distance {
            // If it looks that we have too few sync frames considering the
            // length of the signal so far
            while i / samples_per_work_row as usize > peaks.len() {
                peaks.push((i, corr));
            }
        }

        // Else if this value is bigger than the previous maximum, set this
        // one
        else if corr > peaks.last().unwrap().1 {
            peaks.pop();
            peaks.push((i, corr));
        }
    }

    if context.export_steps {
        context.step(Step::signal("sync_correlation", &correlation, None))?;
    }

    info!("Found {} sync frames", peaks.len());

    Ok(peaks.iter().map(|(index, _value)| *index).collect())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_sample_sync_frame() {

        assert_eq!(
            vec![-1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,
                 -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,
                 -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,
                 -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,
                 -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,
                 -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,
                 -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,
                 -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                 -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                 -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                 -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,],
            generate_sync_frame(Rate::hz(FINAL_RATE * 5)).unwrap()
        );

        assert_eq!(
            vec![-1, -1, -1, -1,
                  1,  1,  1,  1,
                 -1, -1, -1, -1,
                  1,  1,  1,  1,
                 -1, -1, -1, -1,
                  1,  1,  1,  1,
                 -1, -1, -1, -1,
                  1,  1,  1,  1,
                 -1, -1, -1, -1,
                  1,  1,  1,  1,
                 -1, -1, -1, -1,
                  1,  1,  1,  1,
                 -1, -1, -1, -1,
                  1,  1,  1,  1,
                 -1, -1, -1, -1,
                 -1, -1, -1, -1,
                 -1, -1, -1, -1,
                 -1, -1, -1, -1,],
            generate_sync_frame(Rate::hz(FINAL_RATE * 2)).unwrap()
        );
    }
}
