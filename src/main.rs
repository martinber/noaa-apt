extern crate hound;

type Sample = i32;
type Signal = Vec<Sample>;

/// Return samples vector and specs from wav file.
fn load_wav(filename: &str) -> (Signal, hound::WavSpec) {

    let mut reader = hound::WavReader::open(filename)
            .expect("Failed to open WAV file");
    let reader_spec = reader.spec();

    // sample size in bits
    const SAMPLE_BITS: u32 = (std::mem::size_of::<Sample>() * 8) as u32;

    let input_samples: Signal;

    match reader_spec.sample_format {
        hound::SampleFormat::Int => {
            input_samples = reader.samples::<Sample>()
                .map(|x| x.unwrap())
                .collect();
        }
        // TODO: Probar
        hound::SampleFormat::Float => {
            input_samples = reader.samples::<Sample>()
                .map(|x| x.unwrap() * Sample::pow(2, SAMPLE_BITS))
                .collect();
        }
    }

    (input_samples, reader_spec)
}

/// Get reference to biggest sample.
fn get_max(vector: &Signal) -> &Sample {
    let mut max: &Sample = &0;
    for sample in vector.iter() {
        if sample > max {
            max = sample;
        }
    }

    max
}

/// Resample signal by upsampling, filtering and downsampling..
///
/// L is the interpolation factor and M the decimation one.
fn resample(signal: &Signal, l: u8, m: u8) -> Signal{
    let l = l as usize;
    let m = m as usize;
    let mut upsampled: Signal = vec![0; signal.len() * l];

    for (i, sample) in signal.iter().enumerate() {
        upsampled[i * l] = *sample;
    }

    upsampled
}

/// Write signal to file, for testing purposes.
///
/// So you can see the samples in Audacity for example.
fn write_wav(filename: &str, signal: &Signal, spec: hound::WavSpec) {

    let mut writer = hound::WavWriter::create(filename, spec)
            .expect("Failed to create output WAV file");

    for sample in signal.iter() {
        writer.write_sample(*sample).expect("Failed to write sample");
    }
    writer.finalize().expect("Failed to finalize writing WAV file");
}

fn main() -> hound::Result<()> {
    println!("Hello, world!");

    // let mut input_signal: Signal;
    // let input_spec: hound::WavSpec;

    let (input_signal, input_spec) = load_wav("./11025.wav");
    println!("Cargado WAV en Vec");
    println!("reader_spec: {:?}", input_spec);

    let max: &Sample = get_max(&input_signal);
    println!("Maximo: {}", max);

    let r = 4/3; // resampling factor
    let l = 4; // interpolation
    let m = 3; // decimation
    let resampled = resample(&input_signal, l, m);

    let writer_spec = hound::WavSpec {
        channels: 1,
        sample_rate: input_spec.sample_rate * l as u32,
        bits_per_sample: input_spec.bits_per_sample,
        sample_format: hound::SampleFormat::Int,
    };

    write_wav("./salida.wav", &resampled, writer_spec);

    Ok(())
}
