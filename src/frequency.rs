use std::f32::consts::PI;

use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;
use std::ops::Div;
use std::ops::AddAssign;
use std::ops::SubAssign;
use std::ops::MulAssign;
use std::ops::DivAssign;

#[derive(Copy, Clone, Debug)]
pub struct Frequency {
    pi_radians: f32
}

impl Frequency {
    /// Create frequency struct from radians per second.
    fn radians(f: f32) -> Frequency {
        Frequency { pi_radians: f/PI }
    }

    /// Create frequency struct from fractions of pi radians per second.
    fn pi_radians(f: f32) -> Frequency {
        Frequency { pi_radians: f }
    }

    /// Create frequency struct from Hertz and the sample rate used.
    fn hertz(f: f32, rate: f32) -> Frequency {
        Frequency { pi_radians: 2.*f/rate }
    }

    /// Get radians per second.
    fn get_radians(&self) -> f32 {
        self.pi_radians*PI
    }

    /// Get fractions of pi radians per second.
    fn get_pi_radians(&self) -> f32 {
        self.pi_radians
    }

    /// Get frequency on Hertz given some sample rate.
    fn get_hertz(&self, rate: f32) -> f32 {
        self.pi_radians*rate/2.
    }
}

// Operators against Frequencies

impl Add for Frequency {
    type Output = Frequency;

    fn add(self, other: Frequency) -> Frequency {
        Frequency { pi_radians: self.pi_radians + other.pi_radians }
    }
}

impl Sub for Frequency {
    type Output = Frequency;

    fn sub(self, other: Frequency) -> Frequency {
        Frequency { pi_radians: self.pi_radians - other.pi_radians }
    }
}

impl Mul for Frequency {
    type Output = Frequency;

    fn mul(self, other: Frequency) -> Frequency {
        Frequency { pi_radians: self.pi_radians * other.pi_radians }
    }
}

impl Div for Frequency {
    type Output = Frequency;

    fn div(self, other: Frequency) -> Frequency {
        Frequency { pi_radians: self.pi_radians / other.pi_radians }
    }
}

// Operators against f32

impl Mul<f32> for Frequency {
    type Output = Frequency;

    fn mul(self, other: f32) -> Frequency {
        Frequency { pi_radians: self.pi_radians * other }
    }
}

impl Div<f32> for Frequency {
    type Output = Frequency;

    fn div(self, other: f32) -> Frequency {
        Frequency { pi_radians: self.pi_radians / other }
    }
}

// Assign operators against Frequencies

impl AddAssign for Frequency {
    fn add_assign(&mut self, other: Frequency) {
        self.pi_radians += other.pi_radians;
    }
}

impl SubAssign for Frequency {
    fn sub_assign(&mut self, other: Frequency) {
        self.pi_radians -= other.pi_radians;
    }
}

impl MulAssign for Frequency {
    fn mul_assign(&mut self, other: Frequency) {
        self.pi_radians *= other.pi_radians;
    }
}

impl DivAssign for Frequency {
    fn div_assign(&mut self, other: Frequency) {
        self.pi_radians /= other.pi_radians;
    }
}

// Assign operators against f32

impl MulAssign<f32> for Frequency {
    fn mul_assign(&mut self, other: f32) {
        self.pi_radians *= other;
    }
}

impl DivAssign<f32> for Frequency {
    fn div_assign(&mut self, other: f32) {
        self.pi_radians /= other;
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Has equivalences between values on different units
    #[derive(Debug)]
    struct Equivalence {
        pi_radians: f32,
        radians: f32,
        hertz: f32,
        rate: f32
    }

    const TEST_VALUES: [Equivalence; 11] = [
        Equivalence {
            pi_radians: 0.435374149659864,
            radians: 1.367768230134332,
            hertz: 2400.,
            rate: 11025.
        },
        Equivalence {
            pi_radians: -0.435374149659864,
            radians: -1.367768230134332,
            hertz: -2400.,
            rate: 11025.
        },
        Equivalence {
            pi_radians: 0.1,
            radians: 0.3141592653589793,
            hertz: 100.,
            rate: 2000.
        },
        Equivalence {
            pi_radians: -0.1,
            radians: -0.3141592653589793,
            hertz: -100.,
            rate: 2000.
        },
        Equivalence { pi_radians: 0., radians: 0., hertz: 0., rate: 11025. },
        Equivalence { pi_radians: 1., radians: PI, hertz: 5512.5, rate: 11025. },
        Equivalence { pi_radians: -1., radians: -PI, hertz: -5512.5, rate: 11025. },
        Equivalence { pi_radians: 2., radians: 2.*PI, hertz: 11025., rate: 11025. },
        Equivalence { pi_radians: -2., radians: -2.*PI, hertz: -11025., rate: 11025. },
        Equivalence { pi_radians: 300., radians: 300.*PI, hertz: 150., rate: 1. },
        Equivalence { pi_radians: -300., radians: -300.*PI, hertz: -150., rate: 1. },
    ];


    /// Check if two floats are equal given some margin of precision
    fn assert_roughly_equal(a: f32, b: f32) {
        assert_ulps_eq!(a, b, max_ulps = 10)
    }

    #[test]
    fn test_frequency_conversion() {
        for e in TEST_VALUES.iter() {
            let f = Frequency::pi_radians(e.pi_radians);
            assert_roughly_equal(f.get_pi_radians(), e.pi_radians);
            assert_roughly_equal(f.get_radians(), e.radians);
            assert_roughly_equal(f.get_hertz(e.rate), e.hertz);
            let f = Frequency::radians(e.radians);
            assert_roughly_equal(f.get_pi_radians(), e.pi_radians);
            assert_roughly_equal(f.get_radians(), e.radians);
            assert_roughly_equal(f.get_hertz(e.rate), e.hertz);
            let f = Frequency::hertz(e.hertz, e.rate);
            assert_roughly_equal(f.get_pi_radians(), e.pi_radians);
            assert_roughly_equal(f.get_radians(), e.radians);
            assert_roughly_equal(f.get_hertz(e.rate), e.hertz);
        }
    }

    #[test]
    fn test_operations() {
        let a: f32 = 12345.;
        let b: f32 = -23456.;

        let fa = Frequency::pi_radians(a);
        let fb = Frequency::pi_radians(b);

        // Operators against Frequencies

        assert_roughly_equal(
            fa.get_pi_radians() + fb.get_pi_radians(),
            (fa + fb).get_pi_radians());
        assert_roughly_equal(
            fa.get_pi_radians() - fb.get_pi_radians(),
            (fa - fb).get_pi_radians());
        assert_roughly_equal(
            fa.get_pi_radians() * fb.get_pi_radians(),
            (fa * fb).get_pi_radians());
        assert_roughly_equal(
            fa.get_pi_radians() / fb.get_pi_radians(),
            (fa / fb).get_pi_radians());

        // Operators against f32

        assert_roughly_equal(
            fa.get_pi_radians() * b,
            (fa * b).get_pi_radians());
        assert_roughly_equal(
            fa.get_pi_radians() / b,
            (fa / b).get_pi_radians());

        // Assign operators against frequencies

        let fb = Frequency::pi_radians(b);

        let mut fa = Frequency::pi_radians(a);
        fa += fb;
        assert_roughly_equal(fa.get_pi_radians(), a + b);

        let mut fa = Frequency::pi_radians(a);
        fa -= fb;
        assert_roughly_equal(fa.get_pi_radians(), a - b);

        let mut fa = Frequency::pi_radians(a);
        fa *= fb;
        assert_roughly_equal(fa.get_pi_radians(), a * b);

        let mut fa = Frequency::pi_radians(a);
        fa /= fb;
        assert_roughly_equal(fa.get_pi_radians(), a / b);

        // Assign operators against f32

        let mut fa = Frequency::pi_radians(a);
        fa *= b;
        assert_roughly_equal(fa.get_pi_radians(), a * b);

        let mut fa = Frequency::pi_radians(a);
        fa /= b;
        assert_roughly_equal(fa.get_pi_radians(), a / b);
    }

    #[test]
    #[allow(unused_assignments, unused_mut)]
    fn test_copying() {
        let mut a = Frequency::pi_radians(123.);
        let mut b = Frequency::pi_radians(456.);

        b = a;

        assert_roughly_equal(b.get_pi_radians(), 123.);
        assert_roughly_equal(b.get_pi_radians(), 123.);
    }
}
