use dsp;
use dsp::Signal;

use hound;

/// Load wav file and return signal and specs.
pub fn load_wav(filename: &str) -> (Signal, hound::WavSpec) {

    debug!("Loading WAV: {}", filename);

    let mut reader = hound::WavReader::open(filename)
            .expect("Failed to open WAV file");
    let spec = reader.spec();

    if spec.channels != 1 {
        panic!("Failed to open WAV file: audio should have only one channel");
    }

    debug!("WAV specifications: {:?}", spec);

    // TODO: Read WAV files that aren't 16 bit integer encoded
    let input_samples = reader.samples::<i16>().map(|x| x.unwrap() as f32)
                .collect();

    debug!("Finished reading WAV");

    (input_samples, spec)
}

/// Write signal to file, for testing purposes.
///
/// So you can see the samples in Audacity for example.
pub fn write_wav(filename: &str, signal: &Signal, spec: hound::WavSpec) {

    println!("Normalizing samples");

    let max = dsp::get_max(&signal);
    println!("Max: {}", max);
    let normalized: Signal = signal.iter().map(|x| x/max).collect();

    debug!("Writing WAV: {}", filename);

    debug!("WAV specifications: {:?}", spec);

    let mut writer = hound::WavWriter::create(filename, spec)
            .expect("Failed to create output WAV file");

    for sample in normalized.iter() {
        writer.write_sample(*sample).expect("Failed to write sample");
    }
    writer.finalize().expect("Failed to finalize writing WAV file");

    debug!("Finished writing WAV");
}
