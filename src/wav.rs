use std;
use hound;
use dsp::{Sample, Signal};

/// Load wav file and return signal and specs.
pub fn load_wav(filename: &str) -> (Signal, hound::WavSpec) {

    let mut reader = hound::WavReader::open(filename)
            .expect("Failed to open WAV file");
    let reader_spec = reader.spec();

    // sample size in bits
    const SAMPLE_BITS: u32 = (std::mem::size_of::<Sample>() * 8) as u32;

    let input_samples = reader.samples::<i16>().map(|x| x.unwrap() as Sample)
                .collect();
    // let input_samples: Signal = match reader_spec.sample_format {
        // hound::SampleFormat::Int => {
            // let raw = reader.samples::<()
                // .map(|x| x.unwrap())
                // .collect();
//
            // vec![0_f32; 10]
        // }
        // // TODO: Probar
        // hound::SampleFormat::Float => {
            // let raw = reader.samples::<Sample>()
                // .map(|x| x.unwrap() * Sample::powf(2_f32, SAMPLE_BITS as Sample))
                // .collect();
//
            // vec![0_f32; 10]
        // }
    // }

    (input_samples, reader_spec)
}

/// Write signal to file, for testing purposes.
///
/// So you can see the samples in Audacity for example.
pub fn write_wav(filename: &str, signal: &Signal, spec: hound::WavSpec) {

    let mut writer = hound::WavWriter::create(filename, spec)
            .expect("Failed to create output WAV file");

    for sample in signal.iter() {
        writer.write_sample(*sample).expect("Failed to write sample");
    }
    writer.finalize().expect("Failed to finalize writing WAV file");
}
