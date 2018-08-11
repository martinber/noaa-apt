extern crate hound;
extern crate rgsl;
extern crate gnuplot;
extern crate png;
#[macro_use] extern crate log;
extern crate simple_logger;
extern crate argparse;

mod noaa_apt;
mod dsp;
mod wav;
mod misc;

fn main() {

    simple_logger::init().unwrap();

    noaa_apt::resample_wav("./11025.wav", "./salida.wav", 20800);
}
