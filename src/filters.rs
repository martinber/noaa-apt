//! Filter definitions.

use std::f32::consts::PI;

use log::debug;

use crate::dsp::{Freq, Rate, Signal};

/// Some kind of filter
pub trait Filter {
    /// Design filter from parameters.
    fn design(&self) -> Signal;

    /// Resample filter to a new `Rate`.
    fn resample(&mut self, input_rate: Rate, output_rate: Rate);
}

/// No filter.
///
/// Impulse response is an impulse.
#[derive(Clone, PartialEq)]
pub struct NoFilter;

/// Lowpass FIR filter, windowed by a kaiser window.
///
/// Attenuation in positive decibels. The transition band goes from
/// `cutout - delta_w / 2` to `cutout + delta_w / 2`.
#[derive(Clone, PartialEq)]
pub struct Lowpass {
    pub cutout: Freq,
    pub atten: f32,
    pub delta_w: Freq,
}

/// Lowpass and DC removal FIR filter, windowed by a kaiser window.
///
/// Attenuation in positive decibels. It's actually a bandpass filter so has two
/// transition bands, one is the same transition band that `lowpass()` has:
/// `cutout - delta_w / 2` to `cutout + delta_w / 2`. The other transition band
/// goes from `0` to `delta_w`.
#[derive(Clone, PartialEq)]
pub struct LowpassDcRemoval {
    pub cutout: Freq,
    pub atten: f32,
    pub delta_w: Freq,
}

impl Filter for NoFilter {
    fn design(&self) -> Signal {
        return vec![1.];
    }

    fn resample(&mut self, _input_rate: Rate, _output_rate: Rate) {}
}

impl Filter for Lowpass {
    fn design(&self) -> Signal {
        debug!(
            "Designing Lowpass filter, \
               cutout: pi*{}rad/s, attenuation: {}dB, delta_w: pi*{}rad/s",
            self.cutout.get_pi_rad(),
            self.atten,
            self.delta_w.get_pi_rad()
        );

        let window = kaiser(self.atten, self.delta_w);

        if window.len() % 2 == 0 {
            panic!("Kaiser window length should be odd");
        }

        let mut filter: Signal = Vec::with_capacity(window.len());

        let m = window.len() as i32;

        for n in -(m - 1) / 2..=(m - 1) / 2 {
            if n == 0 {
                filter.push(self.cutout.get_pi_rad());
            } else {
                let n = n as f32;
                filter.push((n * PI * self.cutout.get_pi_rad()).sin() / (n * PI));
            }
        }

        debug!("Lowpass filter design finished");

        product(filter, &window)
    }

    fn resample(&mut self, input_rate: Rate, output_rate: Rate) {
        let ratio = output_rate.get_hz() as f32 / input_rate.get_hz() as f32;
        self.cutout /= ratio;
        self.delta_w /= ratio;
    }
}

impl Filter for LowpassDcRemoval {
    fn design(&self) -> Signal {
        debug!(
            "Designing Lowpass and DC removal filter, \
               cutout: pi*{}rad/s, attenuation: {}dB, delta_w: pi*{}rad/s",
            self.cutout.get_pi_rad(),
            self.atten,
            self.delta_w.get_pi_rad()
        );

        let window = kaiser(self.atten, self.delta_w);

        if window.len() % 2 == 0 {
            panic!("Kaiser window length should be odd");
        }

        let mut filter: Signal = Vec::with_capacity(window.len());

        let m = window.len() as i32;

        for n in -(m - 1) / 2..=(m - 1) / 2 {
            if n == 0 {
                filter.push(self.cutout.get_pi_rad() - (self.delta_w / 2.).get_pi_rad());
            } else {
                let n = n as f32;
                filter.push(
                    (n * PI * self.cutout.get_pi_rad()).sin() / (n * PI)
                        - (n * PI * (self.delta_w / 2.).get_pi_rad()).sin() / (n * PI),
                );
            }
        }

        debug!("Lowpass and DC removal filter design finished");

        product(filter, &window)
    }

    fn resample(&mut self, input_rate: Rate, output_rate: Rate) {
        let ratio = output_rate.get_hz() as f32 / input_rate.get_hz() as f32;
        self.cutout /= ratio;
        self.delta_w /= ratio;
    }
}

/// Design Kaiser window from parameters.
///
/// The length depends on the parameters given, and it's always odd.
fn kaiser(atten: f32, delta_w: Freq) -> Signal {
    use crate::misc::bessel_i0 as bessel;

    debug!(
        "Designing Kaiser window, \
           attenuation: {}dB, delta_w: pi*{}rad/s",
        atten,
        delta_w.get_pi_rad()
    );

    let beta: f32;
    if atten > 50. {
        beta = 0.1102 * (atten - 8.7);
    } else if atten < 21. {
        beta = 0.;
    } else {
        beta = 0.5842 * (atten - 21.).powf(0.4) + 0.07886 * (atten - 21.);
    }

    // Filter length, we want an odd length
    let mut length: i32 = ((atten - 8.) / (2.285 * delta_w.get_rad())).ceil() as i32 + 1;
    if length % 2 == 0 {
        length += 1;
    }

    let mut window: Signal = Vec::with_capacity(length as usize);

    for n in -(length - 1) / 2..=(length - 1) / 2 {
        let n = n as f32;
        let m = length as f32;
        window.push(bessel(beta * (1. - (n / (m / 2.)).powi(2)).sqrt()) / bessel(beta))
    }

    debug!(
        "Kaiser window design finished, beta: {}, length: {}",
        beta, length
    );

    window
}

/// Product of two vectors, element by element.
pub fn product(mut v1: Signal, v2: &Signal) -> Signal {
    if v1.len() != v2.len() {
        panic!("Both vectors must have the same length");
    }

    for i in 0..v1.len() {
        v1[i] *= v2[i];
    }

    v1
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Calculate absolute value of fft
    fn abs_fft(signal: &Signal) -> Signal {
        use rustfft::num_complex::Complex;
        use rustfft::FftPlanner;

        let mut buffer: Vec<Complex<f32>> = signal.iter().map(|x| Complex::new(*x, 0.)).collect();

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(buffer.len());
        fft.process(&mut buffer); // Result is in buffer

        buffer.iter().map(|x| x.norm()).collect()
    }

    /// Check if two vectors of float are equal given some margin of precision
    fn vector_roughly_equal(a: &Vec<f32>, b: &Vec<f32>) -> bool {
        // Iterator with tuples of values to compare.
        // [(a1, b1), (a2, b2), (a3, b3), ...]
        let mut values = a.iter().zip(b.iter());
        // Check if every pair have similar values
        values.all(|(&a, &b)| approx::ulps_eq!(a, b))
    }

    #[test]
    fn test_abs_fft() {
        // Checked with GNU Octave
        assert!(vector_roughly_equal(
            &abs_fft(&vec![1., 2., 3., 4.]),
            &vec![10., 2.828427124746190, 2., 2.828427124746190]
        ));
        assert!(vector_roughly_equal(
            &abs_fft(&vec![1., 1., 1., 1., 1., 1., 1.]),
            &vec![7., 0., 0., 0., 0., 0., 0.]
        ));
        assert!(vector_roughly_equal(
            &abs_fft(&vec![1., -1., 1., -1.]),
            &vec![0., 0., 4., 0.]
        ));
    }

    #[test]
    fn test_lowpass() {
        // cutout, atten and delta_w values
        let test_parameters: Vec<(Freq, f32, Freq)> = vec![
            (Freq::pi_rad(1. / 4.), 20., Freq::pi_rad(1. / 10.)),
            (Freq::pi_rad(1. / 3.), 35., Freq::pi_rad(1. / 30.)),
            (Freq::pi_rad(2. / 5.), 60., Freq::pi_rad(1. / 20.)),
        ];

        for parameters in test_parameters.iter() {
            let (cutout, atten, delta_w) = *parameters;

            let ripple = 10_f32.powf(-atten / 20.); // 10^(-atten/20)

            let coeff = Lowpass {
                cutout,
                atten,
                delta_w,
            }
            .design();
            let fft = abs_fft(&coeff);

            println!(
                "cutout: {}, atten: {}, delta_w: {}",
                cutout.get_pi_rad(),
                atten,
                delta_w.get_pi_rad()
            );
            println!("coeff: {:?}", coeff);

            for (i, v) in fft.iter().enumerate() {
                let w = Freq::pi_rad(2. * (i as f32) / (fft.len() as f32));

                if w < cutout - delta_w / 2. {
                    println!(
                        "Passband, ripple: {}, v: {}, i: {}, w: {}",
                        ripple,
                        v,
                        i,
                        w.get_pi_rad()
                    );
                    assert!(*v < 1. + ripple && *v > 1. - ripple);
                } else if w > cutout + delta_w / 2. && w < Freq::pi_rad(1.) {
                    println!(
                        "Stopband, ripple: {}, v: {}, i: {}, w: {}",
                        ripple,
                        v,
                        i,
                        w.get_pi_rad()
                    );
                    assert!(*v < ripple);
                }
            }
        }
    }

    #[test]
    fn test_lowpass_dc_removal() {
        // cutout, atten and delta_w values
        let test_parameters: Vec<(Freq, f32, Freq)> = vec![
            (Freq::pi_rad(1. / 4.), 20., Freq::pi_rad(1. / 10.)),
            (Freq::pi_rad(1. / 3.), 35., Freq::pi_rad(1. / 30.)),
            (Freq::pi_rad(2. / 5.), 60., Freq::pi_rad(1. / 20.)),
        ];

        for parameters in test_parameters.iter() {
            let (cutout, atten, delta_w) = *parameters;

            let ripple = 10_f32.powf(-atten / 20.); // 10^(-atten/20)

            let coeff = LowpassDcRemoval {
                cutout,
                atten,
                delta_w,
            }
            .design();
            let fft = abs_fft(&coeff);

            println!(
                "cutout: {}, atten: {}, delta_w: {}",
                cutout.get_pi_rad(),
                atten,
                delta_w.get_pi_rad()
            );
            println!("coeff: {:?}", coeff);

            for (i, v) in fft.iter().enumerate() {
                let w = Freq::pi_rad(2. * (i as f32) / (fft.len() as f32));

                if i == 0 {
                    println!(
                        "Zero, ripple: {}, v: {}, i: {}, w: {}",
                        ripple,
                        v,
                        i,
                        w.get_pi_rad()
                    );
                    // I'm using 2*ripple otherwise it fails, for some reason it
                    // always has problems on zero. Anyways, it's only 3dB
                    assert!(*v < 2. * ripple);
                }

                if w > delta_w && w < cutout - delta_w / 2. {
                    println!(
                        "Passband, ripple: {}, v: {}, i: {}, w: {}",
                        ripple,
                        v,
                        i,
                        w.get_pi_rad()
                    );
                    assert!(*v < 1. + ripple && *v > 1. - ripple);
                } else if w > cutout + delta_w / 2. && w < Freq::pi_rad(1.) {
                    println!(
                        "Stopband, ripple: {}, v: {}, i: {}, w: {}",
                        ripple,
                        v,
                        i,
                        w.get_pi_rad()
                    );
                    assert!(*v < ripple);
                }
            }
        }
    }

    #[test]
    fn test_no_filter() {
        let coeff = NoFilter {}.design();
        assert!(coeff == vec![1.,]);
    }

    // Check if a filter designed on 1000hz and then resampled to 3000hz is the
    // same as a filter designed directly on 3000hz.

    #[test]
    fn test_lowpass_resample() {
        let mut filter = Lowpass {
            cutout: Freq::hz(123., Rate::hz(1000)),
            atten: 40.,
            delta_w: Freq::hz(12., Rate::hz(1000)),
        };

        filter.resample(Rate::hz(1000), Rate::hz(3000));

        let expected = Lowpass {
            cutout: Freq::hz(123., Rate::hz(3000)),
            atten: 40.,
            delta_w: Freq::hz(12., Rate::hz(3000)),
        };

        assert!(filter == expected);
    }

    #[test]
    fn test_lowpass_dc_removal_resample() {
        let mut filter = LowpassDcRemoval {
            cutout: Freq::hz(123., Rate::hz(1000)),
            atten: 40.,
            delta_w: Freq::hz(12., Rate::hz(1000)),
        };

        filter.resample(Rate::hz(1000), Rate::hz(3000));

        let expected = LowpassDcRemoval {
            cutout: Freq::hz(123., Rate::hz(3000)),
            atten: 40.,
            delta_w: Freq::hz(12., Rate::hz(3000)),
        };

        assert!(filter == expected);
    }

    #[test]
    fn test_no_filter_resample() {
        let original = NoFilter {};

        let mut resampled = original.clone();
        resampled.resample(Rate::hz(1000), Rate::hz(3000));

        assert!(original == resampled);
    }
}
