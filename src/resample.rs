//! High-level function for resampling.

use std::path::Path;

use log::{info, warn, debug};

use crate::config;
use crate::context::{Context, Step};
use crate::dsp::{self, Signal, Rate, Freq};
use crate::err;
use crate::filters;
use crate::misc;
use crate::wav;


/// Load and resample WAV file.
///
/// Copy the modification time timestamp too.
pub fn resample(
    context: &mut Context,
    settings: config::Settings,
    input_filename: &Path,
    output_filename: &Path,
    output_rate: u32,
) -> err::Result<()> {

    info!("Reading WAV file");
    context.status(0.0, "Reading WAV file".to_string());

    let (input_signal, input_spec) = wav::load_wav(input_filename)?;
    let input_rate = Rate::hz(input_spec.sample_rate);
    let timestamp = misc::read_timestamp(input_filename)?;

    context.step(Step::signal("input", &input_signal, Some(input_rate)))?;

    info!("Resampling");
    context.status(0.2, format!("Resampling to {}", output_rate));

    let resampled = dsp::resample(
        context,
        &input_signal,
        input_rate,
        Rate::hz(output_rate),
        settings.wav_resample_atten,
        Freq::pi_rad(settings.wav_resample_delta_freq),
    )?;

    if resampled.is_empty() {
        return Err(err::Error::Internal(
            "Got zero samples after resampling, audio file too short or \
            output sampling frequency too low".to_string())
        );
    }

    let writer_spec = hound::WavSpec {
        channels: 1,
        sample_rate: output_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    info!("Writing WAV to '{}'", output_filename.display());
    context.status(0.8, format!("Writing WAV to '{}'", output_filename.display()));

    wav::write_wav(&output_filename, &resampled, writer_spec)?;
    misc::write_timestamp(timestamp, &output_filename)?;

    context.status(1., "Finished".to_string());
    Ok(())
}
