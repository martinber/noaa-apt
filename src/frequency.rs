use std::f32::consts::PI;

use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;
use std::ops::Div;
use std::ops::AddAssign;
use std::ops::SubAssign;
use std::ops::MulAssign;
use std::ops::DivAssign;

pub type Rate = f32;

#[derive(Copy, Clone, Debug)]
pub struct Freq {
    pi_radians: f32
}

impl Freq {
    /// Create frequency struct from radians per second.
    fn radians(f: f32) -> Freq {
        Freq { pi_radians: f/PI }
    }

    /// Create frequency struct from fractions of pi radians per second.
    fn pi_radians(f: f32) -> Freq {
        Freq { pi_radians: f }
    }

    /// Create frequency struct from Hertz and the sample rate used.
    fn hertz(f: f32, rate: f32) -> Freq {
        Freq { pi_radians: 2.*f/rate }
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

macro_rules! overload {
    (trait $trait:ident, $self:ident : $self_type:ident, $other:ident : $other_type:ident, fn $method:ident $expr:block ) => {
        impl $trait<$other_type> for $self_type {
            type Output = $self_type;

            fn $method($self, $other: $other_type) -> $self_type {
                $expr
            }
        }
    }
}
macro_rules! overload_assign {
    (trait $trait:ident, $self:ident : $self_type:ident, $other:ident : $other_type:ident, fn $method:ident $expr:block ) => {
        impl $trait<$other_type> for $self_type {
            fn $method(&mut $self, $other: $other_type) {
                $expr
            }
        }
    }
}

// Operators against Frequencies

overload!(trait Add, self: Freq, other: Freq, fn add {
    Freq { pi_radians: self.pi_radians + other.pi_radians }
});
overload!(trait Sub, self: Freq, other: Freq, fn sub {
    Freq { pi_radians: self.pi_radians - other.pi_radians }
});
overload!(trait Mul, self: Freq, other: Freq, fn mul {
    Freq { pi_radians: self.pi_radians * other.pi_radians }
});
overload!(trait Div, self: Freq, other: Freq, fn div {
    Freq { pi_radians: self.pi_radians / other.pi_radians }
});

// Operators against f32

overload!(trait Mul, self: Freq, other: f32, fn mul {
    Freq { pi_radians: self.pi_radians * other }
});
overload!(trait Div, self: Freq, other: f32, fn div {
    Freq { pi_radians: self.pi_radians / other }
});

// Assign operators against Frequencies

overload_assign!(trait AddAssign, self: Freq, other: Freq, fn add_assign {
    self.pi_radians += other.pi_radians;
});

overload_assign!(trait SubAssign, self: Freq, other: Freq, fn sub_assign {
    self.pi_radians -= other.pi_radians;
});

overload_assign!(trait MulAssign, self: Freq, other: Freq, fn mul_assign {
    self.pi_radians *= other.pi_radians;
});

overload_assign!(trait DivAssign, self: Freq, other: Freq, fn div_assign {
    self.pi_radians /= other.pi_radians;
});

// Assign operators against f32

overload_assign!(trait MulAssign, self: Freq, other: f32, fn mul_assign {
    self.pi_radians *= other;
});

overload_assign!(trait DivAssign, self: Freq, other: f32, fn div_assign {
    self.pi_radians /= other;
});

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
            let f = Freq::pi_radians(e.pi_radians);
            assert_roughly_equal(f.get_pi_radians(), e.pi_radians);
            assert_roughly_equal(f.get_radians(), e.radians);
            assert_roughly_equal(f.get_hertz(e.rate), e.hertz);
            let f = Freq::radians(e.radians);
            assert_roughly_equal(f.get_pi_radians(), e.pi_radians);
            assert_roughly_equal(f.get_radians(), e.radians);
            assert_roughly_equal(f.get_hertz(e.rate), e.hertz);
            let f = Freq::hertz(e.hertz, e.rate);
            assert_roughly_equal(f.get_pi_radians(), e.pi_radians);
            assert_roughly_equal(f.get_radians(), e.radians);
            assert_roughly_equal(f.get_hertz(e.rate), e.hertz);
        }
    }

    #[test]
    fn test_operations() {
        let a: f32 = 12345.;
        let b: f32 = -23456.;

        let fa = Freq::pi_radians(a);
        let fb = Freq::pi_radians(b);

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

        // Operators against Rate

        /*
        assert_roughly_equal(
            fa.get_pi_radians() * Rate(b),
            (fa * b).get_pi_radians());
        assert_roughly_equal(
            fa.get_pi_radians() / Rate(b),
            (fa / b).get_pi_radians());
        */

        // Assign operators against frequencies

        let fb = Freq::pi_radians(b);

        let mut fa = Freq::pi_radians(a);
        fa += fb;
        assert_roughly_equal(fa.get_pi_radians(), a + b);

        let mut fa = Freq::pi_radians(a);
        fa -= fb;
        assert_roughly_equal(fa.get_pi_radians(), a - b);

        let mut fa = Freq::pi_radians(a);
        fa *= fb;
        assert_roughly_equal(fa.get_pi_radians(), a * b);

        let mut fa = Freq::pi_radians(a);
        fa /= fb;
        assert_roughly_equal(fa.get_pi_radians(), a / b);

        // Assign operators against f32

        let mut fa = Freq::pi_radians(a);
        fa *= b;
        assert_roughly_equal(fa.get_pi_radians(), a * b);

        let mut fa = Freq::pi_radians(a);
        fa /= b;
        assert_roughly_equal(fa.get_pi_radians(), a / b);

        // Assign operators against Rate

        /*
        let mut fa = Freq::pi_radians(a);
        fa *= b;
        assert_roughly_equal(fa.get_pi_radians(), a * b);

        let mut fa = Freq::pi_radians(a);
        fa /= b;
        assert_roughly_equal(fa.get_pi_radians(), a / b);
        */
    }

    #[test]
    #[allow(unused_assignments, unused_mut)]
    fn test_copying() {
        let mut a = Freq::pi_radians(123.);
        let mut b = Freq::pi_radians(456.);

        b = a;

        assert_roughly_equal(b.get_pi_radians(), 123.);
        assert_roughly_equal(b.get_pi_radians(), 123.);
    }
}
