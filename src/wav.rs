use dsp;
use dsp::Signal;
use err;

use hound;

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

    let input_samples: Signal = match spec.sample_format {
        hound::SampleFormat::Int => {
            reader.samples::<i32>()
                .collect::<Result<Vec<i32>, hound::Error>>()?
                .iter().enumerate().filter_map(|(i, x)|
                        match i % spec.channels as usize {
                            0 => Some(*x as f32),
                            _ => None,
                        })
                .collect()
        }
        hound::SampleFormat::Float => {
            reader.samples::<f32>()
                .collect::<Result<Vec<f32>, hound::Error>>()?
                .iter().enumerate().filter_map(|(i, x)|
                        match i % spec.channels as usize {
                            0 => Some(*x),
                            _ => None,
                        })
                .collect()
        }
    };

    debug!("Finished reading WAV");

    Ok((input_samples, spec))
}

/// Write signal to file.
pub fn write_wav(filename: &str, signal: &Signal, spec: hound::WavSpec) -> err::Result<()> {

    debug!("Normalizing samples");

    let max = dsp::get_max(&signal)?;
    debug!("Max: {}", max);
    let normalized: Signal = signal.iter().map(|x| x/max).collect();

    debug!("Writing WAV to '{}'", filename);

    debug!("WAV specifications: {:?}", spec);

    let mut writer = hound::WavWriter::create(filename, spec)?;

    for sample in normalized.iter() {
        writer.write_sample(*sample)?;
    }
    writer.finalize()?;

    debug!("Finished writing WAV");

    Ok(())
}
