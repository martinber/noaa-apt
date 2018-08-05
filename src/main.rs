extern crate hound;
extern crate rgsl;
extern crate gnuplot;

mod dsp;
mod wav;

// use wav;
// use dsp;
use dsp::Signal;
use std::f32::consts::PI;

fn main() -> hound::Result<()> {
    println!("Hello, world!");

    // let mut input_signal: Signal;
    // let input_spec: hound::WavSpec;

    println!("Leyendo WAV:");
    let (input_signal, input_spec) = wav::load_wav("./11025.wav");
    println!("Cargado WAV en Vec");
    println!("reader_spec: {:?}", input_spec);

    let max: &f32 = dsp::get_max(&input_signal);
    println!("Maximo: {}", max);

    /*

    let r = 7/3; // resampling factor
    let l = 7; // interpolation
    let m = 3; // decimation
    let resampled = dsp::resample(&input_signal, l, m);

    let writer_spec = hound::WavSpec {
        channels: 1,
        sample_rate: input_spec.sample_rate * l as u32,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    println!("Resampleado");
    let max = dsp::get_max(&resampled);
    println!("Maximo: {}", max);
    let normalized = resampled.iter().map(|x| x/max).collect();

    println!("Escribiendo WAV:");
    wav::write_wav("./salida.wav", &normalized, writer_spec);

    */

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

    Ok(())
}
