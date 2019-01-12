use hound;

use dsp;
use dsp::Signal;
use err;


/// Load wav file and return signal and specs.
pub fn load_wav(filename: &str) -> err::Result<(Signal, hound::WavSpec)> {

    debug!("Loading WAV: {}", filename);

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
pub fn write_wav(filename: &str, signal: &Signal, spec: hound::WavSpec) -> err::Result<()> {

    debug!("Normalizing samples and writing WAV to '{}'", filename);

    let max = dsp::get_max(&signal)?;
    debug!("Max: {}", max);

    debug!("WAV specifications: {:?}", spec);

    let mut writer = hound::WavWriter::create(filename, spec)?;

    for sample in signal.iter() {
        writer.write_sample(*sample / max)?;
    }
    writer.finalize()?;

    debug!("Finished writing WAV");

    Ok(())
}
