use wav;
use dsp::{self, Signal, Rate, Freq};
use err;
use filters;
use context::{Context, Step};

use std;
use hound;
use png;
use reqwest;

// Working sample rate, used during demodulation and syncing, better if multiple
// of the final sample rate. That way, the second resampling it's just a
// decimation
const WORK_RATE: u32 = 20800;

// Final signal (with has one sample per pixel) sample rate
const FINAL_RATE: u32 = 4160;

// Pixels per row
const PX_PER_ROW: u32 = 2080;

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

    // Need to center signal on zero (remove DC) to get meaningful correlation
    // values
    let average: f32 = *dsp::get_max(&signal)? / 2.; // Not true but close enough.
    let signal: Signal = signal.iter().map(|x| x - average).collect();

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

    let signal = dsp::resample_with_filter(
        &mut context, &signal, work_rate, final_rate, filters::NoFilter)?;
    let max = dsp::get_max(&signal)?;
    let min = dsp::get_min(&signal)?;
    let range = max - min;

    debug!("Mapping samples from {}-{} to 0-255", min, max);

    let signal: Vec<u8> = signal.iter()
        .map(|x| ((x - min) / range * 255.) as u8).collect();

    context.step(Step::signal(
            "mapped",
            &signal.iter().map(|x| *x as f32).collect(),
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
