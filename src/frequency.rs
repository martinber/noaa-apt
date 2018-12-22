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
    fn hertz(f: f32, rate: Rate) -> Freq {
        Freq { pi_radians: 2.*f/rate.get_hertz() }
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
    fn get_hertz(&self, rate: Rate) -> f32 {
        self.pi_radians*rate.get_hertz()/2.
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rate {
    hertz: f32
}

impl Rate {
    /// Create rate from Hertz.
    fn hertz(r: f32) -> Rate {
        Rate { hertz: r }
    }
    /// Get rate on Hertz.
    fn get_hertz(&self) -> f32 {
        self.hertz
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

// Freq against Freq

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

// Freq against f32

overload!(trait Mul, self: Freq, other: f32, fn mul {
    Freq { pi_radians: self.pi_radians * other }
});
overload!(trait Div, self: Freq, other: f32, fn div {
    Freq { pi_radians: self.pi_radians / other }
});

// Freq assign against Freq

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

// Freq assign against f32

overload_assign!(trait MulAssign, self: Freq, other: f32, fn mul_assign {
    self.pi_radians *= other;
});

overload_assign!(trait DivAssign, self: Freq, other: f32, fn div_assign {
    self.pi_radians /= other;
});

// Rate against Rate

overload!(trait Add, self: Rate, other: Rate, fn add {
    Rate { hertz: self.hertz + other.hertz }
});
overload!(trait Sub, self: Rate, other: Rate, fn sub {
    Rate { hertz: self.hertz - other.hertz }
});
overload!(trait Mul, self: Rate, other: Rate, fn mul {
    Rate { hertz: self.hertz * other.hertz }
});
overload!(trait Div, self: Rate, other: Rate, fn div {
    Rate { hertz: self.hertz / other.hertz }
});

// Rate against f32

overload!(trait Mul, self: Rate, other: f32, fn mul {
    Rate { hertz: self.hertz * other }
});
overload!(trait Div, self: Rate, other: f32, fn div {
    Rate { hertz: self.hertz / other }
});

// Rate assign against Rate

overload_assign!(trait AddAssign, self: Rate, other: Rate, fn add_assign {
    self.hertz += other.hertz;
});

overload_assign!(trait SubAssign, self: Rate, other: Rate, fn sub_assign {
    self.hertz -= other.hertz;
});

overload_assign!(trait MulAssign, self: Rate, other: Rate, fn mul_assign {
    self.hertz *= other.hertz;
});

overload_assign!(trait DivAssign, self: Rate, other: Rate, fn div_assign {
    self.hertz /= other.hertz;
});

// Rate assign against f32

overload_assign!(trait MulAssign, self: Rate, other: f32, fn mul_assign {
    self.hertz *= other;
});

overload_assign!(trait DivAssign, self: Rate, other: f32, fn div_assign {
    self.hertz /= other;
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
            let rate = Rate::hertz(e.rate);
            let f = Freq::pi_radians(e.pi_radians);
            assert_roughly_equal(f.get_pi_radians(), e.pi_radians);
            assert_roughly_equal(f.get_radians(), e.radians);
            assert_roughly_equal(f.get_hertz(rate), e.hertz);
            let f = Freq::radians(e.radians);
            assert_roughly_equal(f.get_pi_radians(), e.pi_radians);
            assert_roughly_equal(f.get_radians(), e.radians);
            assert_roughly_equal(f.get_hertz(rate), e.hertz);
            let f = Freq::hertz(e.hertz, rate);
            assert_roughly_equal(f.get_pi_radians(), e.pi_radians);
            assert_roughly_equal(f.get_radians(), e.radians);
            assert_roughly_equal(f.get_hertz(rate), e.hertz);
        }
    }

    #[test]
    fn test_freq_operations() {
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
    }

    #[test]
    #[allow(unused_assignments, unused_mut)]
    fn test_freq_copying() {
        let mut a = Freq::pi_radians(123.);
        let mut b = Freq::pi_radians(456.);

        b = a;

        assert_roughly_equal(b.get_pi_radians(), 123.);
        assert_roughly_equal(b.get_pi_radians(), 123.);
    }

    #[test]
    fn test_rate_operations() {
        let a: f32 = 12345.;
        let b: f32 = -23456.;

        let fa = Rate::hertz(a);
        let fb = Rate::hertz(b);

        // Operators against Rates

        assert_roughly_equal(
            fa.get_hertz() + fb.get_hertz(),
            (fa + fb).get_hertz());
        assert_roughly_equal(
            fa.get_hertz() - fb.get_hertz(),
            (fa - fb).get_hertz());
        assert_roughly_equal(
            fa.get_hertz() * fb.get_hertz(),
            (fa * fb).get_hertz());
        assert_roughly_equal(
            fa.get_hertz() / fb.get_hertz(),
            (fa / fb).get_hertz());

        // Operators against f32

        assert_roughly_equal(
            fa.get_hertz() * b,
            (fa * b).get_hertz());
        assert_roughly_equal(
            fa.get_hertz() / b,
            (fa / b).get_hertz());

        // Assign operators against rates

        let fb = Rate::hertz(b);

        let mut fa = Rate::hertz(a);
        fa += fb;
        assert_roughly_equal(fa.get_hertz(), a + b);

        let mut fa = Rate::hertz(a);
        fa -= fb;
        assert_roughly_equal(fa.get_hertz(), a - b);

        let mut fa = Rate::hertz(a);
        fa *= fb;
        assert_roughly_equal(fa.get_hertz(), a * b);

        let mut fa = Rate::hertz(a);
        fa /= fb;
        assert_roughly_equal(fa.get_hertz(), a / b);

        // Assign operators against f32

        let mut fa = Rate::hertz(a);
        fa *= b;
        assert_roughly_equal(fa.get_hertz(), a * b);

        let mut fa = Rate::hertz(a);
        fa /= b;
        assert_roughly_equal(fa.get_hertz(), a / b);
    }

    #[test]
    #[allow(unused_assignments, unused_mut)]
    fn test_rate_copying() {
        let mut a = Rate::hertz(123.);
        let mut b = Rate::hertz(456.);

        b = a;

        assert_roughly_equal(b.get_hertz(), 123.);
        assert_roughly_equal(b.get_hertz(), 123.);
    }
}
