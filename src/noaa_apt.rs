//! Core of the program.
//!
//! This module has the high-level functions for decoding APT.

use hound;
use png;
use reqwest;

use wav;
use dsp::{self, Signal, Rate, Freq};
use err;
use filters;
use context::{Context, Step};


// Working sample rate, used during demodulation and syncing, better if multiple
// of the final sample rate. That way, the second resampling it's just a
// decimation
const WORK_RATE: u32 = 20800;

// Final signal (with has one sample per pixel) sample rate
const FINAL_RATE: u32 = 4160;

// Pixels per row
pub const PX_PER_ROW: u32 = 2080;

// AM carrier frequency
const CARRIER_FREQ: u32 = 2400;

// Samples on each image row when at WORK_RATE
const SAMPLES_PER_WORK_ROW: u32 = PX_PER_ROW * WORK_RATE / FINAL_RATE;

/// Resample wav file
pub fn resample_wav(
    input_filename: &str,
    output_filename: &str,
    output_rate: Rate,
    export_wav: bool,
    export_resample_filtered: bool,
) -> err::Result<()> {

    info!("Reading WAV file");
    let (input_signal, input_spec) = wav::load_wav(input_filename)?;
    let input_rate = Rate::hz(input_spec.sample_rate);
    let mut context = Context::resample(export_wav, export_resample_filtered);

    context.step(Step::signal("input", &input_signal, Some(input_rate)))?;

    info!("Resampling");
    let resampled = dsp::resample(
        &mut context, &input_signal, input_rate, output_rate, Freq::pi_rad(0.1))?;

    if resampled.is_empty() {
        return Err(err::Error::Internal(
            "Got zero samples after resampling, audio file too short or \
            output sampling frequency too low".to_string())
        );
    }

    let writer_spec = hound::WavSpec {
        channels: 1,
        sample_rate: output_rate.get_hz(),
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    info!("Writing WAV to '{}'", output_filename);

    wav::write_wav(output_filename, &resampled, writer_spec)?;

    Ok(())
}

/// Find sync frame positions.
///
/// Returns list of found sync frames positions.
fn find_sync(context: &mut Context, signal: &Signal) -> err::Result<Vec<usize>> {

    info!("Searching for sync frames");

    // TODO define and resample to WORK_RATE
    // Sync frame to find: seven impulses and some black pixels (some lines
    // have something like 8 black pixels and then white ones)
    let mut guard: Vec<i32> = Vec::with_capacity(20*7 + 35);
    for _i in 0..7 {
        guard.extend_from_slice(&[-1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
                                 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    }
    for _i in 0..35 {
        guard.push(-1);
    }

    // list of maximum correlations found: (index, value)
    let mut peaks: Vec<(usize, f32)> = Vec::new();
    peaks.push((0, 0.));

    // Minimum distance between peaks, some arbitrary number smaller but close
    // to the number of samples by line
    let min_distance: usize = SAMPLES_PER_WORK_ROW as usize * 8/10;

    // Save cross-correlation if wav-steps is enabled
    let mut correlation = if context.export {
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

        if context.export {
            correlation.push(corr);
        }

        // If previous peak is too far, keep it and add this value to the
        // list as a new peak
        if i - peaks.last().unwrap().0 > min_distance {
            peaks.push((i, corr));
        }

        // Else if this value is bigger than the previous maximum, set this
        // one
        else if corr > peaks.last().unwrap().1 {
            peaks.pop();
            peaks.push((i, corr));
        }
    }

    if context.export {
        context.step(Step::signal("sync_correlation", &correlation, None))?;
    }

    info!("Found {} sync frames", peaks.len());

    Ok(peaks.iter().map(|(index, _value)| *index).collect())
}

/// Get wedge value from one telemetry band.
///
/// Useful for wedges 15 and 16. They have different values on each band.
///
/// Use the given index as the position of the first wedge.
// fn get_wedge_single(wedge: u32, telemetry: &Signal, index: usize) -> f32 {
//
    // // Telemetry signals have 8 values per wedge
    // // Substract one because wedge numbers start from 1 instead of 0
    // let wedge_pos = index + (wedge - 1) as usize * 8;
//
    // // If this is a contrast wedge, better to use the next contrast wedge too
    // if wedge <= 9 {
        // let wedge_pos_2 = wedge_pos + 16*8; // 16 wedges of length 8
//
        // (telemetry[wedge_pos..(wedge_pos+8)].iter().sum::<f32>() +
         // telemetry[wedge_pos_2..(wedge_pos_2+8)].iter().sum::<f32>()) / 16.
//
    // } else {
//
        // telemetry[wedge_pos..(wedge_pos+8)].iter().sum::<f32>() / 8.
//
    // }
// }
//
// /// Get wedge value from two telemetry bands
// ///
// /// Useful for wedges 1 to 14. They have different values on each band.
// ///
// /// Use the given index as the position of the first wedge
// fn get_wedge(wedge: u32, telemetry_a: &Signal, telemetry_b: &Signal, index: usize) -> f32 {
//
    // (get_wedge_single(wedge, telemetry_a, index) +
     // get_wedge_single(wedge, telemetry_b, index)) / 2.
// }

enum Channel {
    A,
    B,
}

/// Contains the telemetry information
struct Telemetry {
    // One value for each wedge on each band
    values_a: Vec<f32>,
    values_b: Vec<f32>,
}

impl Telemetry {

    /// Read telemetry from received bands.
    ///
    /// Takes two signals containing the horizontal averages of each band, also
    /// takes the row where to start reading the frame
    pub fn from_bands(means_a: &Signal, means_b: &Signal, row: usize) -> Telemetry{

        // Calculate the mean of contiguous 8 values, starting from the given
        // until wedge 9 of the next frame
        let means_a: Signal = means_a[row..]
            .chunks_exact(8).map(|x| x.iter().sum::<f32>() / 8.).take(16 + 9).collect();
        let means_b: Signal = means_b[row..]
            .chunks_exact(8).map(|x| x.iter().sum::<f32>() / 8.).take(16 + 9).collect();

        let telemetry = Telemetry {
            values_a: (1..=16).map(|wedge|
                // Contrast wedges are averaged to the ones on the next frame
                if wedge <= 9 {
                    (means_a[wedge - 1] + means_a[wedge + 16 - 1]) / 2.
                } else {
                    means_a[wedge - 1]
                }
            ).collect(),

            values_b: (1..=16).map(|wedge|
                // Contrast wedges are averaged to the ones on the next frame
                if wedge <= 9 {
                    (means_b[wedge - 1] + means_a[wedge + 16 - 1]) / 2.
                } else {
                    means_b[wedge - 1]
                }
            ).collect(),
        };

        debug!("Telemetry wedges_a: {:?}, wedges_b: {:?}",
            telemetry.values_a, telemetry.values_b);

        telemetry
    }

    /// Get value of wedge.
    ///
    /// Does not mean anything by itself, you should use it for comparison
    /// against another wedge.
    pub fn get_wedge_value(&self, wedge: u32, channel: Option<Channel>) -> f32 {

        let wedge = wedge as usize;
        match channel {
            Some(Channel::A) => self.values_a[wedge - 1],
            Some(Channel::B) => self.values_b[wedge - 1],
            None => (self.values_a[wedge - 1] + self.values_b[wedge - 1]) / 2.,
        }
    }

    /// Get channel name.
    pub fn get_channel_name(&self, channel: Channel) -> &str {

        // Take wedge 16 and compare to wedges 1 to 9 to determine the channel

        let value = self.get_wedge_value(16, Some(channel));

        let contrast_wedges = (1..=9).map(|i| self.get_wedge_value(i, None));

        // let differences = (1..9)
            // .map(|i| (self.get_wedge_value(i, None) - value).abs());

        let channel_names = ["1", "2", "3a", "4", "5", "3b", "Unknown", "Unknown", "Unknown"];

        // for name, diff in channel_names.iter().zip(differences).min_by(|t| t.1).0

        let (name, _difference) = channel_names.iter().zip(contrast_wedges)
            .min_by(|a, b|
                (a.1 - value).abs().partial_cmp(&(b.1 - value).abs()).expect("Can't compare values")
            ).expect("Empty zip");

        name
    }
}

/// Maps signal values to range 0-255
fn map(context: &mut Context, signal: &Signal, sync: bool) -> err::Result<Vec<u8>> {

    // Sample of telemetry band used for correlation. Only contrast wedges
    // (1 to 9) are given. Each value is repeated 8 times because the height of
    // the wedges is 8 pixels
    let telemetry_sample: Signal = [
        31., 63., 95., 127., 159., 191., 224., 255., 0., // For contrast
        0., 0., 0., 0., 0., 0., 0.,                      // Variable
        31., 63., 95., 127., 159., 191., 224., 255., 0.  // For contrast
    ].iter().flat_map(|x| std::iter::repeat(*x).take(8)).collect();

    // Reserve vectors, length is the height of the image

    // Horizontal average of both bands
    let mut mean_a: Signal = Vec::with_capacity(signal.len() / PX_PER_ROW as usize);
    let mut mean_b: Signal = Vec::with_capacity(signal.len() / PX_PER_ROW as usize);
    // Horizontal variance of both telemetry bands, indicates if there is noise
    let mut variance: Signal = Vec::with_capacity(signal.len() / PX_PER_ROW as usize);

    // Iterate a row at a time (each row is one pixel high)
    for line in signal.chunks_exact(PX_PER_ROW as usize) {

        // Values on each bamd
        let a_values = &line[994..(994+44)];
        let b_values = &line[2034..(2034+44)];

        // Horizontal average
        let curr_mean_a: f32 = a_values.iter().sum::<f32>() / 44.;
        let curr_mean_b: f32 = b_values.iter().sum::<f32>() / 44.;
        mean_a.push(curr_mean_a);
        mean_b.push(curr_mean_b);

        // Horizontal variance
        variance.push(
            (a_values.iter().map(|x| (x - curr_mean_a).powi(2)).sum::<f32>() +
             b_values.iter().map(|x| (x - curr_mean_b).powi(2)).sum::<f32>()) / 88.
        );
    }

    // Cross correlation between telemetry band averages and the telemetry
    // sample, has peaks where telemetry frames start
    let mut corr: Signal = Vec::new();

    // Cross correlation divided by the standard deviation, has peaks where
    // telemetry frames with low standard deviation start
    let mut quality: Signal = Vec::new();

    // These will be used only if the steps are exported
    if context.export {
        corr.reserve(signal.len() / PX_PER_ROW as usize);
        quality.reserve(signal.len() / PX_PER_ROW as usize)
    };

    // Row with the best quality
    let mut best: (usize, f32) = (0, 0.); // (row, quality)

    // Cross correlation of both telemetry bands with a sample
    for i in 0 .. mean_a.len() - telemetry_sample.len() {
        let mut sum: f32 = 0.;
        for j in 0..telemetry_sample.len() {
            sum += telemetry_sample[j] * mean_a[i + j];
            sum += telemetry_sample[j] * mean_b[i + j];
        }

        // sqrt() for standard deviation instead of variance, otherwise variance
        // is too big and noise affects the quality estimation too much compared
        // to the correlation
        // Check standard deviation on the same places where we cross correlate,
        // that's why I use telemetry_sample.len()
        let q = sum / variance[i..(i + telemetry_sample.len())].iter()
            .map(|x| x.sqrt()).sum::<f32>();

        if q > best.1 {
            best = (i, q);
        }

        if context.export {
            corr.push(sum);
            quality.push(q);
        }
    }

    let telemetry = Telemetry::from_bands(&mean_a, &mean_b, best.0);
    info!("Channel A: {}, Channel B: {}",
        telemetry.get_channel_name(Channel::A), telemetry.get_channel_name(Channel::B));

    context.step(Step::signal("telemetry_a", &mean_a, None))?;
    context.step(Step::signal("telemetry_b", &mean_b, None))?;
    context.step(Step::signal("telemetry_correlation", &corr, None))?;
    context.step(Step::signal("telemetry_variance", &variance, None))?;
    context.step(Step::signal("telemetry_quality", &quality, None))?;

    let max = telemetry.get_wedge_value(8, None);
    let min = telemetry.get_wedge_value(9, None);
    let range = max - min;
    let signal: Vec<u8> = signal.iter()
        .map(|x|
             // Map and clamp between 0 and 255 using min() and max()
             ((x - min) / range * 255.).max(0.).min(255.) as u8
        ).collect();

    Ok(signal)
}

/// Decode APT image from WAV file.
pub fn decode(
    input_filename: &str,
    output_filename: &str,
    export_wav: bool,
    export_resample_filtered: bool,
    sync: bool,
) -> err::Result<()>{

    info!("Reading WAV file");

    let (signal, input_spec) = wav::load_wav(input_filename)?;
    let input_rate = Rate::hz(input_spec.sample_rate);
    let work_rate = Rate::hz(WORK_RATE);
    let final_rate = Rate::hz(FINAL_RATE);
    let mut context = Context::decode(
        work_rate, final_rate, export_wav, export_resample_filtered);

    context.step(Step::signal("input", &signal, Some(input_rate)))?;

    info!("Resampling to {}", WORK_RATE);

    let filter = filters::LowpassDcRemoval {
        // Cutout frequency of the resampling filter, only the AM spectrum should go
        // through to avoid noise, 2 times the carrier frequency is enough
        cutout: Freq::hz(CARRIER_FREQ as f32, input_rate) * 2.,

        atten: 40.,

        // Width of transition band, we are using a DC removal filter that has a
        // transition band from zero to delta_w. I think that APT signals have
        // nothing below 500Hz.
        delta_w: Freq::hz(500., input_rate)
    };
    let signal = dsp::resample_with_filter(
        &mut context, &signal, input_rate, work_rate, filter)?;

    if signal.len() < 10 * SAMPLES_PER_WORK_ROW as usize {
        return Err(err::Error::Internal(
            "Got less than 10 rows of samples, audio file is too short".to_string()));
    }

    info!("Demodulating");

    let signal = dsp::demodulate(
        &mut context, &signal, Freq::hz(CARRIER_FREQ as f32, work_rate))?;

    info!("Filtering");

    let cutout = Freq::pi_rad(FINAL_RATE as f32 / WORK_RATE as f32);
    let filter = filters::Lowpass {
        cutout,
        atten: 20.,
        delta_w: cutout / 5.
    };
    // mut because on sync the signal is going to be modified
    let mut signal = dsp::filter(&mut context, &signal, filter)?;

    if sync {
        info!("Syncing");

        // Get list of sync frames positions
        let sync_pos = find_sync(&mut context, &signal)?;

        if sync_pos.len() < 5 {
            return Err(err::Error::Internal(
                "Found less than 5 sync frames, audio file is too short or too \
                noisy".to_string())
            );
        }

        // Create new "aligned" vector to SAMPLES_PER_WORK_ROW. Each row starts on
        // a found sync frame position
        let mut aligned: Signal = Vec::new();

        // For each sync position
        for i in 0..sync_pos.len()-1 {
            // Check if there are enough samples left to fill an image row
            if (sync_pos[i] + SAMPLES_PER_WORK_ROW as usize) < signal.len() {

                aligned.extend_from_slice(
                    &signal[sync_pos[i] .. sync_pos[i] + SAMPLES_PER_WORK_ROW as usize]
                );
            }
        }

        signal = aligned;

    } else {
        info!("Not syncing");

        // If we are not syncing send a dummy correlation step
        context.step(Step::signal("sync_correlation", &vec![], Some(work_rate)))?;

        // Crop signal to multiple of SAMPLES_PER_WORK_ROW
        let length = signal.len();
        signal.truncate(length
            / SAMPLES_PER_WORK_ROW as usize // Integer division
            * SAMPLES_PER_WORK_ROW as usize
        );
    }

    context.step(Step::signal("sync_result", &signal, Some(work_rate)))?;

    info!("Resampling to 4160");

    // Resample without filter because we already filtered the signal before
    // syncing
    let signal = dsp::resample_with_filter(
        &mut context, &signal, work_rate, final_rate, filters::NoFilter)?;

    info!("Reading telemetry and mapping colors");

    let signal = map(&mut context, &signal, sync)?;
    context.step(Step::signal(
            "mapped",
            &signal.iter().map(|x| f32::from(*x)).collect(),
            Some(final_rate)
    ))?;

    info!("Writing PNG to '{}'", output_filename);

    // To use encoder.set()
    use png::HasParameters;

    let path = std::path::Path::new(output_filename);
    let file = std::fs::File::create(path)?;
    let buffer = &mut std::io::BufWriter::new(file);

    let height = signal.len() as u32 / PX_PER_ROW;

    let mut encoder = png::Encoder::new(buffer, PX_PER_ROW, height);
    encoder.set(png::ColorType::Grayscale).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;

    writer.write_image_data(&signal[..])?;

    Ok(())
}

/// Check if there is an update to this program.
///
/// Takes a String with the current version being used.
///
/// Returns a tuple of a bool idicating if there are new updates and a String
/// with the latest version. Wrapped in Option, returns None if there were
/// problems retrieving new versions and logs the error.
pub fn check_updates(current: &str) -> Option<(bool, String)> {
    let addr = format!("https://noaa-apt.mbernardi.com.ar/version_check?{}", current);

    let latest: Option<String> = match reqwest::get(addr.as_str()) {
        Ok(mut response) => {
            match response.text() {
                Ok(text) => {
                    Some(text.trim().to_string())
                }
                Err(e) => {
                    warn!("Error checking for updates: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            warn!("Error checking for updates: {}", e);
            None
        }
    };

    match latest {
        Some(latest) => {
            if latest.len() > 10 {
                warn!("Error checking for updates: Response too long");
                None
            } else {
                // Return true if there are updates
                Some((latest != current, latest))
            }
        }
        None => None,
    }
}
