//! Functions for loading and saving WAV files.

use std::path::Path;

use hound;

use dsp;
use dsp::Signal;
use err;


/// Load wav file, return `Signal` and specs.
pub fn load_wav(filename: &Path) -> err::Result<(Signal, hound::WavSpec)> {
    debug!("Loading WAV: {}", filename.display());

    let mut reader = hound::WavReader::open(filename)?;
    let spec = reader.spec();

    if spec.channels != 1 {
        warn!("WAV file has {} channels (probably stereo), processing only the \
            first one", spec.channels);
    }

    debug!("WAV specifications: {:?}", spec);

    // Convert samples to float, also if there is more than one channel, the
    // samples are interleaved so we drop samples from extra channels using
    // filter_map()
    let input_samples: Signal = match spec.sample_format {
        hound::SampleFormat::Int => {
            reader.samples::<i32>()
                .collect::<Result<Vec<i32>, hound::Error>>()?
                .iter()
                .enumerate()
                .filter_map(|(i, x)|
                    match i % spec.channels as usize {
                        0 => Some(*x as f32),
                        _ => None,
                    }
                )
                .collect()
        }
        hound::SampleFormat::Float => {
            reader.samples::<f32>()
                .collect::<Result<Vec<f32>, hound::Error>>()?
                .iter()
                .enumerate()
                .filter_map(|(i, x)|
                    match i % spec.channels as usize {
                        0 => Some(*x),
                        _ => None,
                    }
                )
                .collect()
        }
    };

    debug!("Finished reading WAV");

    Ok((input_samples, spec))
}

/// Write signal to file.
///
/// Only works for 32 bit float and 16 bit integer. As an input this function
/// takes always a `Signal` but converts samples according to the WAV specs.
pub fn write_wav(filename: &Path, signal: &Signal, spec: hound::WavSpec) -> err::Result<()> {
    debug!("Normalizing samples and writing WAV to '{}'", filename.display());

    let max = dsp::get_max(&signal)?;
    debug!("Max: {}", max);

    debug!("WAV specifications: {:?}", spec);

    let mut writer = hound::WavWriter::create(filename, spec)?;

    if spec.bits_per_sample == 32
        && spec.sample_format == hound::SampleFormat::Float
    {
        for sample in signal.iter() {
            writer.write_sample(*sample / max)?;
        }
    }
    else if spec.bits_per_sample == 16
        && spec.sample_format == hound::SampleFormat::Int
    {
        for sample in signal.iter() {
            writer.write_sample(
                (*sample / max * (i16::max_value() as f32)) as i16
            )?;
        }
    }
    else
    {
        return Err(err::Error::Internal(
            format!("Can't write WAV with spec {:?}", spec)
        ));
    }

    writer.finalize()?;

    debug!("Finished writing WAV");

    Ok(())
}
