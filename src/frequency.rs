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
    pi_rad: f32
}

impl Freq {
    /// Create frequency struct from radians per second.
    fn rad(f: f32) -> Freq {
        Freq { pi_rad: f/PI }
    }

    /// Create frequency struct from fractions of pi radians per second.
    fn pi_rad(f: f32) -> Freq {
        Freq { pi_rad: f }
    }

    /// Create frequency struct from Hertz and the sample rate used.
    fn hz(f: f32, rate: Rate) -> Freq {
        Freq { pi_rad: 2.*f/rate.get_hz() }
    }

    /// Get radians per second.
    fn get_rad(&self) -> f32 {
        self.pi_rad*PI
    }

    /// Get fractions of pi radians per second.
    fn get_pi_rad(&self) -> f32 {
        self.pi_rad
    }

    /// Get frequency on Hertz given some sample rate.
    fn get_hz(&self, rate: Rate) -> f32 {
        self.pi_rad*rate.get_hz()/2.
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rate {
    hz: f32
}

impl Rate {
    /// Create rate from Hertz.
    fn hz(r: f32) -> Rate {
        Rate { hz: r }
    }
    /// Get rate on Hertz.
    fn get_hz(&self) -> f32 {
        self.hz
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
    Freq { pi_rad: self.pi_rad + other.pi_rad }
});
overload!(trait Sub, self: Freq, other: Freq, fn sub {
    Freq { pi_rad: self.pi_rad - other.pi_rad }
});
overload!(trait Mul, self: Freq, other: Freq, fn mul {
    Freq { pi_rad: self.pi_rad * other.pi_rad }
});
overload!(trait Div, self: Freq, other: Freq, fn div {
    Freq { pi_rad: self.pi_rad / other.pi_rad }
});

// Freq against f32

overload!(trait Mul, self: Freq, other: f32, fn mul {
    Freq { pi_rad: self.pi_rad * other }
});
overload!(trait Div, self: Freq, other: f32, fn div {
    Freq { pi_rad: self.pi_rad / other }
});

// Freq assign against Freq

overload_assign!(trait AddAssign, self: Freq, other: Freq, fn add_assign {
    self.pi_rad += other.pi_rad;
});

overload_assign!(trait SubAssign, self: Freq, other: Freq, fn sub_assign {
    self.pi_rad -= other.pi_rad;
});

overload_assign!(trait MulAssign, self: Freq, other: Freq, fn mul_assign {
    self.pi_rad *= other.pi_rad;
});

overload_assign!(trait DivAssign, self: Freq, other: Freq, fn div_assign {
    self.pi_rad /= other.pi_rad;
});

// Freq assign against f32

overload_assign!(trait MulAssign, self: Freq, other: f32, fn mul_assign {
    self.pi_rad *= other;
});

overload_assign!(trait DivAssign, self: Freq, other: f32, fn div_assign {
    self.pi_rad /= other;
});

// Rate against Rate

overload!(trait Add, self: Rate, other: Rate, fn add {
    Rate { hz: self.hz + other.hz }
});
overload!(trait Sub, self: Rate, other: Rate, fn sub {
    Rate { hz: self.hz - other.hz }
});
overload!(trait Mul, self: Rate, other: Rate, fn mul {
    Rate { hz: self.hz * other.hz }
});
overload!(trait Div, self: Rate, other: Rate, fn div {
    Rate { hz: self.hz / other.hz }
});

// Rate against f32

overload!(trait Mul, self: Rate, other: f32, fn mul {
    Rate { hz: self.hz * other }
});
overload!(trait Div, self: Rate, other: f32, fn div {
    Rate { hz: self.hz / other }
});

// Rate assign against Rate

overload_assign!(trait AddAssign, self: Rate, other: Rate, fn add_assign {
    self.hz += other.hz;
});

overload_assign!(trait SubAssign, self: Rate, other: Rate, fn sub_assign {
    self.hz -= other.hz;
});

overload_assign!(trait MulAssign, self: Rate, other: Rate, fn mul_assign {
    self.hz *= other.hz;
});

overload_assign!(trait DivAssign, self: Rate, other: Rate, fn div_assign {
    self.hz /= other.hz;
});

// Rate assign against f32

overload_assign!(trait MulAssign, self: Rate, other: f32, fn mul_assign {
    self.hz *= other;
});

overload_assign!(trait DivAssign, self: Rate, other: f32, fn div_assign {
    self.hz /= other;
});

#[cfg(test)]
mod tests {

    use super::*;

    /// Has equivalences between values on different units
    #[derive(Debug)]
    struct Equivalence {
        pi_rad: f32,
        rad: f32,
        hz: f32,
        rate: f32
    }

    const TEST_VALUES: [Equivalence; 11] = [
        Equivalence {
            pi_rad: 0.435374149659864,
            rad: 1.367768230134332,
            hz: 2400.,
            rate: 11025.
        },
        Equivalence {
            pi_rad: -0.435374149659864,
            rad: -1.367768230134332,
            hz: -2400.,
            rate: 11025.
        },
        Equivalence {
            pi_rad: 0.1,
            rad: 0.3141592653589793,
            hz: 100.,
            rate: 2000.
        },
        Equivalence {
            pi_rad: -0.1,
            rad: -0.3141592653589793,
            hz: -100.,
            rate: 2000.
        },
        Equivalence { pi_rad: 0., rad: 0., hz: 0., rate: 11025. },
        Equivalence { pi_rad: 1., rad: PI, hz: 5512.5, rate: 11025. },
        Equivalence { pi_rad: -1., rad: -PI, hz: -5512.5, rate: 11025. },
        Equivalence { pi_rad: 2., rad: 2.*PI, hz: 11025., rate: 11025. },
        Equivalence { pi_rad: -2., rad: -2.*PI, hz: -11025., rate: 11025. },
        Equivalence { pi_rad: 300., rad: 300.*PI, hz: 150., rate: 1. },
        Equivalence { pi_rad: -300., rad: -300.*PI, hz: -150., rate: 1. },
    ];


    /// Check if two floats are equal given some margin of precision
    fn assert_roughly_equal(a: f32, b: f32) {
        assert_ulps_eq!(a, b, max_ulps = 10)
    }

    #[test]
    fn test_frequency_conversion() {
        for e in TEST_VALUES.iter() {
            let rate = Rate::hz(e.rate);
            let f = Freq::pi_rad(e.pi_rad);
            assert_roughly_equal(f.get_pi_rad(), e.pi_rad);
            assert_roughly_equal(f.get_rad(), e.rad);
            assert_roughly_equal(f.get_hz(rate), e.hz);
            let f = Freq::rad(e.rad);
            assert_roughly_equal(f.get_pi_rad(), e.pi_rad);
            assert_roughly_equal(f.get_rad(), e.rad);
            assert_roughly_equal(f.get_hz(rate), e.hz);
            let f = Freq::hz(e.hz, rate);
            assert_roughly_equal(f.get_pi_rad(), e.pi_rad);
            assert_roughly_equal(f.get_rad(), e.rad);
            assert_roughly_equal(f.get_hz(rate), e.hz);
        }
    }

    #[test]
    fn test_freq_operations() {
        let a: f32 = 12345.;
        let b: f32 = -23456.;

        let fa = Freq::pi_rad(a);
        let fb = Freq::pi_rad(b);

        // Operators against Frequencies

        assert_roughly_equal(
            fa.get_pi_rad() + fb.get_pi_rad(),
            (fa + fb).get_pi_rad());
        assert_roughly_equal(
            fa.get_pi_rad() - fb.get_pi_rad(),
            (fa - fb).get_pi_rad());
        assert_roughly_equal(
            fa.get_pi_rad() * fb.get_pi_rad(),
            (fa * fb).get_pi_rad());
        assert_roughly_equal(
            fa.get_pi_rad() / fb.get_pi_rad(),
            (fa / fb).get_pi_rad());

        // Operators against f32

        assert_roughly_equal(
            fa.get_pi_rad() * b,
            (fa * b).get_pi_rad());
        assert_roughly_equal(
            fa.get_pi_rad() / b,
            (fa / b).get_pi_rad());

        // Assign operators against frequencies

        let fb = Freq::pi_rad(b);

        let mut fa = Freq::pi_rad(a);
        fa += fb;
        assert_roughly_equal(fa.get_pi_rad(), a + b);

        let mut fa = Freq::pi_rad(a);
        fa -= fb;
        assert_roughly_equal(fa.get_pi_rad(), a - b);

        let mut fa = Freq::pi_rad(a);
        fa *= fb;
        assert_roughly_equal(fa.get_pi_rad(), a * b);

        let mut fa = Freq::pi_rad(a);
        fa /= fb;
        assert_roughly_equal(fa.get_pi_rad(), a / b);

        // Assign operators against f32

        let mut fa = Freq::pi_rad(a);
        fa *= b;
        assert_roughly_equal(fa.get_pi_rad(), a * b);

        let mut fa = Freq::pi_rad(a);
        fa /= b;
        assert_roughly_equal(fa.get_pi_rad(), a / b);
    }

    #[test]
    #[allow(unused_assignments, unused_mut)]
    fn test_freq_copying() {
        let mut a = Freq::pi_rad(123.);
        let mut b = Freq::pi_rad(456.);

        b = a;

        assert_roughly_equal(b.get_pi_rad(), 123.);
        assert_roughly_equal(b.get_pi_rad(), 123.);
    }

    #[test]
    fn test_rate_operations() {
        let a: f32 = 12345.;
        let b: f32 = -23456.;

        let fa = Rate::hz(a);
        let fb = Rate::hz(b);

        // Operators against Rates

        assert_roughly_equal(
            fa.get_hz() + fb.get_hz(),
            (fa + fb).get_hz());
        assert_roughly_equal(
            fa.get_hz() - fb.get_hz(),
            (fa - fb).get_hz());
        assert_roughly_equal(
            fa.get_hz() * fb.get_hz(),
            (fa * fb).get_hz());
        assert_roughly_equal(
            fa.get_hz() / fb.get_hz(),
            (fa / fb).get_hz());

        // Operators against f32

        assert_roughly_equal(
            fa.get_hz() * b,
            (fa * b).get_hz());
        assert_roughly_equal(
            fa.get_hz() / b,
            (fa / b).get_hz());

        // Assign operators against rates

        let fb = Rate::hz(b);

        let mut fa = Rate::hz(a);
        fa += fb;
        assert_roughly_equal(fa.get_hz(), a + b);

        let mut fa = Rate::hz(a);
        fa -= fb;
        assert_roughly_equal(fa.get_hz(), a - b);

        let mut fa = Rate::hz(a);
        fa *= fb;
        assert_roughly_equal(fa.get_hz(), a * b);

        let mut fa = Rate::hz(a);
        fa /= fb;
        assert_roughly_equal(fa.get_hz(), a / b);

        // Assign operators against f32

        let mut fa = Rate::hz(a);
        fa *= b;
        assert_roughly_equal(fa.get_hz(), a * b);

        let mut fa = Rate::hz(a);
        fa /= b;
        assert_roughly_equal(fa.get_hz(), a / b);
    }

    #[test]
    #[allow(unused_assignments, unused_mut)]
    fn test_rate_copying() {
        let mut a = Rate::hz(123.);
        let mut b = Rate::hz(456.);

        b = a;

        assert_roughly_equal(b.get_hz(), 123.);
        assert_roughly_equal(b.get_hz(), 123.);
    }
}
