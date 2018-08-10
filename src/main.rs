extern crate hound;
extern crate rgsl;
extern crate gnuplot;
extern crate png;

#[macro_use] extern crate log;
extern crate simple_logger;

mod dsp;
mod wav;

// use wav;
// use dsp;
use dsp::Signal;
use std::f32::consts::PI;

fn main() -> hound::Result<()> {

    simple_logger::init().unwrap();

    debug!("Leyendo WAV:");
    let (input_signal, input_spec) = wav::load_wav("./20800.wav");
    debug!("Cargado WAV en Vec");
    debug!("reader_spec: {:?}", input_spec);

    let atten = 50.;
    let delta_w = 1./20.;
    let demodulated = dsp::demodulate(&input_signal, atten, delta_w);


    let max: &f32 = dsp::get_max(&input_signal);
    debug!("Maximo: {}", max);



    // sync frame to find: seven impulses and some black pixels (some lines
    // have something like 8 black pixels and then white ones)
    let mut syncA: Signal = Vec::with_capacity(20*7 + 35);
    for _i in 0..7 {
        syncA.extend_from_slice(&[-1., -1., -1., -1., -1., -1., -1., -1., -1., -1.,
                                 1., 1., 1., 1., 1., 1., 1., 1., 1., 1.]);
    }
    for _i in 0..35 {
        syncA.push(-1.);
    }

    // list of maximum correlations found: (index, value)
    let mut peaks: Vec<(usize, f32)> = Vec::new();
    peaks.push((0, 0.));

    // minimum distance between peaks
    let mindistance: usize = 2000*5;

    // need to shift the values down to get meaningful correlation values
    // signalshifted = [x-128 for x in signal]
    // syncA = [x-128 for x in syncA]
    for i in 0 .. demodulated.len() - syncA.len() {
        let mut corr: f32 = 0.;
        for j in 0..syncA.len() {
            corr += syncA[j] * (demodulated[i+j] - *max/2.);
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


    println!("peaks: {:?}", peaks);

    let mut salida: Signal = Vec::new();

    for i in 0..peaks.len()-1 {
        salida.extend_from_slice(&demodulated[peaks[i].0 .. peaks[i].0+2080*5]);
    }

    let demodulated = salida;




    let r = 7/3; // resampling factor
    let l = 1; // interpolation
    let m = 5;
    let resampled = dsp::resample(&demodulated, l, m, atten, delta_w);

    let writer_spec = hound::WavSpec {
        channels: 1,
        sample_rate: input_spec.sample_rate * l/m as u32,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    println!("Resampleado");
    let max = dsp::get_max(&resampled);
    println!("Maximo: {}", max);
    let normalized = resampled.iter().map(|x| x/max).collect();

    println!("Escribiendo WAV:");
    wav::write_wav("./salida.wav", &normalized, writer_spec);

    let acomodado: Vec<u8> = normalized.iter().map(|x| (x*255.) as u8).collect();

    // For reading and opening files
    use std::path::Path;
    use std::fs::File;
    use std::io::BufWriter;
    // To use encoder.set()
    use png::HasParameters;

    let path = Path::new("./salida.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, 2080, 700);
    encoder.set(png::ColorType::Grayscale).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&acomodado[0..(2080*700) as usize]).unwrap(); // Save

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

    Ok(())
}
