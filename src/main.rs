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
    let (input_signal, input_spec) = wav::load_wav("./11025.wav");
    debug!("Cargado WAV en Vec");
    debug!("reader_spec: {:?}", input_spec);

    let max: &f32 = dsp::get_max(&input_signal);
    debug!("Maximo: {}", max);

    let r = 7/3; // resampling factor
    let l = 1; // interpolation
    let m = 1; // decimation
    let atten = 50.;
    let delta_w = 1./20.;
    let resampled = dsp::resample(&input_signal, l, m, atten, delta_w);

    let writer_spec = hound::WavSpec {
        channels: 1,
        sample_rate: input_spec.sample_rate * l/m as u32,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let demodulated = dsp::demodulate(&resampled, atten, delta_w);

    println!("Resampleado");
    let max = dsp::get_max(&demodulated);
    println!("Maximo: {}", max);
    let normalized = demodulated.iter().map(|x| x/max).collect();

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

    let mut encoder = png::Encoder::new(w, input_spec.sample_rate/2, 700);
    encoder.set(png::ColorType::Grayscale).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&acomodado[0..(input_spec.sample_rate/2*700) as usize]).unwrap(); // Save

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
