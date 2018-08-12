use wav;
use dsp;
use dsp::Signal;

use hound;
use png;

/// Resample wav file
///
/// The filter parameters are the default ones.
pub fn resample_wav(input_filename: &str, output_filename: &str,
                    output_rate: u32) {

    info!("Reading WAV file");
    let (input_signal, input_spec) = wav::load_wav(input_filename);

    info!("Resampling");
    let resampled = dsp::resample_to(&input_signal, input_spec.sample_rate,
                                     output_rate);

    let writer_spec = hound::WavSpec {
        channels: 1,
        sample_rate: output_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    info!("Writing WAV to '{}'", output_filename);

    wav::write_wav(output_filename, &resampled, writer_spec);

}

/// Decode APT image from WAV file.
pub fn decode(input_filename: &str, output_filename: &str) {

    info!("Reading WAV file");

    let (signal, input_spec) = wav::load_wav(input_filename);

    info!("Resampling");

    let signal = dsp::resample_to(&signal, input_spec.sample_rate, 20800);

    info!("Demodulating");

    let atten = 40.;
    let delta_w = 1./20.;
    let signal = dsp::demodulate(&signal, atten, delta_w);

    info!("Syncing");

    let max: &f32 = dsp::get_max(&signal);

    // sync frame to find: seven impulses and some black pixels (some lines
    // have something like 8 black pixels and then white ones)
    let mut guard: Signal = Vec::with_capacity(20*7 + 35);
    for _i in 0..7 {
        guard.extend_from_slice(&[-1., -1., -1., -1., -1., -1., -1., -1., -1., -1.,
                                 1., 1., 1., 1., 1., 1., 1., 1., 1., 1.]);
    }
    for _i in 0..35 {
        guard.push(-1.);
    }

    // list of maximum correlations found: (index, value)
    let mut peaks: Vec<(usize, f32)> = Vec::new();
    peaks.push((0, 0.));

    // minimum distance between peaks
    let mindistance: usize = 2000*5;

    // need to shift the values down to get meaningful correlation values
    for i in 0 .. signal.len() - guard.len() {
        let mut corr: f32 = 0.;
        for j in 0..guard.len() {
            corr += guard[j] * (signal[i+j] - *max/2.);
        }

        // if previous peak is too far, keep it and add this value to the
        // list as a new peak
        if i - peaks.last().unwrap().0 > mindistance {
            peaks.push((i, corr));
        }

        // else if this value is bigger than the previous maximum, set this
        // one
        else if corr > peaks.last().unwrap().1 {
            peaks.pop();
            peaks.push((i, corr));
        }
    }

    let mut aligned: Signal = Vec::new();

    for i in 0..peaks.len()-1 {
        aligned.extend_from_slice(&signal[peaks[i].0 .. peaks[i].0+2080*5]);
    }

    let l = 1; // interpolation
    let m = 5;
    let aligned = dsp::resample(&aligned, l, m, atten, delta_w);

    println!("Resampleado");
    let max = dsp::get_max(&aligned);
    println!("Maximo: {}", max);
    let aligned: Signal = aligned.iter().map(|x| x/max).collect();


    let aligned: Vec<u8> = aligned.iter().map(|x| (x*255.) as u8).collect();

    // For reading and opening files
    use std::path::Path;
    use std::fs::File;
    use std::io::BufWriter;
    // To use encoder.set()
    use png::HasParameters;

    info!("Writing PNG to '{}'", output_filename);

    let path = Path::new(output_filename);
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    println!("{}", aligned.len());
    let height = aligned.len() as u32 / 2080;

    let mut encoder = png::Encoder::new(w, 2080, height);
    encoder.set(png::ColorType::Grayscale).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&aligned[..]).unwrap(); // Save

    /*
    // let window = dsp::kaiser(40., 1./10.);
    // let mut lowpass = dsp::lowpass(window.len() as u32, 1./4.);

    let lowpass = dsp::lowpass(1./4., 40., 1./10.);

    // println!("window: {:?}", window);
    // lowpass = dsp::product(window, &lowpass);
    println!("filter: {:?}", lowpass);

    let x: Vec<usize> = (0 .. lowpass.len()).collect();
    let mut fg = gnuplot::Figure::new();
    fg.axes2d().lines(&x, lowpass, &[gnuplot::Caption("A line"), gnuplot::Color("black")]);
    fg.show();
    */

}
