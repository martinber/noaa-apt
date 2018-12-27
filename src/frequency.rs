use std::f32::consts::PI;

use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;
use std::ops::Div;
use std::ops::AddAssign;
use std::ops::SubAssign;
use std::ops::MulAssign;
use std::ops::DivAssign;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Freq {
    pi_rad: f32
}

#[allow(dead_code)]
impl Freq {
    /// Create frequency struct from radians per second.
    pub fn rad(f: f32) -> Freq {
        Freq { pi_rad: f/PI }
    }

    /// Create frequency struct from fractions of pi radians per second.
    pub fn pi_rad(f: f32) -> Freq {
        Freq { pi_rad: f }
    }

    /// Create frequency struct from Hertz and the sample rate used.
    pub fn hz(f: f32, rate: Rate) -> Freq {
        Freq { pi_rad: 2.*f / rate.get_hz() as f32 }
    }

    /// Get radians per second.
    pub fn get_rad(&self) -> f32 {
        self.pi_rad*PI
    }

    /// Get fractions of pi radians per second.
    pub fn get_pi_rad(&self) -> f32 {
        self.pi_rad
    }

    /// Get frequency on Hertz given some sample rate.
    pub fn get_hz(&self, rate: Rate) -> f32 {
        self.pi_rad * rate.get_hz() as f32 / 2.
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Rate {
    hz: u32
}

impl Rate {
    /// Create rate from Hertz.
    pub fn hz<T: num::Integer + num::ToPrimitive>(r: T) -> Rate {
        // Should panic only when r < 0
        Rate { hz: num::NumCast::from(r).unwrap() }
    }
    /// Get rate on Hertz.
    pub fn get_hz(&self) -> u32 {
        self.hz
    }
}

macro_rules! overload {
    (trait $trait:ident,
     $self:ident : $self_type:ident,
     $other:ident : $other_type:ident,
     fn $method:ident $expr:block ) => {

        impl $trait<$other_type> for $self_type {
            type Output = $self_type;

            fn $method($self, $other: $other_type) -> $self_type {
                $expr
            }
        }
    }
}
macro_rules! overload_assign {
    (trait $trait:ident,
     $self:ident : $self_type:ident,
     $other:ident : $other_type:ident,
     fn $method:ident $expr:block ) => {

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

// Freq against u32

overload!(trait Mul, self: Freq, other: u32, fn mul {
    Freq { pi_rad: self.pi_rad * other as f32 }
});
overload!(trait Div, self: Freq, other: u32, fn div {
    Freq { pi_rad: self.pi_rad / other as f32 }
});

// Freq against usize

overload!(trait Mul, self: Freq, other: usize, fn mul {
    Freq { pi_rad: self.pi_rad * other as f32 }
});
overload!(trait Div, self: Freq, other: usize, fn div {
    Freq { pi_rad: self.pi_rad / other as f32 }
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

// Freq assign against u32

overload_assign!(trait MulAssign, self: Freq, other: u32, fn mul_assign {
    self.pi_rad *= other as f32;
});

overload_assign!(trait DivAssign, self: Freq, other: u32, fn div_assign {
    self.pi_rad /= other as f32;
});

// Freq assign against usize

overload_assign!(trait MulAssign, self: Freq, other: usize, fn mul_assign {
    self.pi_rad *= other as f32;
});

overload_assign!(trait DivAssign, self: Freq, other: usize, fn div_assign {
    self.pi_rad /= other as f32;
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

// Rate against u32

overload!(trait Mul, self: Rate, other: u32, fn mul {
    Rate { hz: self.hz * other }
});
overload!(trait Div, self: Rate, other: u32, fn div {
    Rate { hz: self.hz / other }
});

// Rate against usize

overload!(trait Mul, self: Rate, other: usize, fn mul {
    Rate { hz: self.hz * other as u32 }
});
overload!(trait Div, self: Rate, other: usize, fn div {
    Rate { hz: self.hz / other as u32 }
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

// Rate assign against u32

overload_assign!(trait MulAssign, self: Rate, other: u32, fn mul_assign {
    self.hz *= other;
});

overload_assign!(trait DivAssign, self: Rate, other: u32, fn div_assign {
    self.hz /= other;
});

// Rate assign against usize

overload_assign!(trait MulAssign, self: Rate, other: usize, fn mul_assign {
    self.hz *= other as u32;
});

overload_assign!(trait DivAssign, self: Rate, other: usize, fn div_assign {
    self.hz /= other as u32;
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
        rate: u32
    }

    const TEST_VALUES: [Equivalence; 11] = [
        Equivalence {
            pi_rad: 0.435374149659864,
            rad: 1.367768230134332,
            hz: 2400.,
            rate: 11025
        },
        Equivalence {
            pi_rad: -0.435374149659864,
            rad: -1.367768230134332,
            hz: -2400.,
            rate: 11025
        },
        Equivalence {
            pi_rad: 0.1,
            rad: 0.3141592653589793,
            hz: 100.,
            rate: 2000
        },
        Equivalence {
            pi_rad: -0.1,
            rad: -0.3141592653589793,
            hz: -100.,
            rate: 2000
        },
        Equivalence { pi_rad: 0., rad: 0., hz: 0., rate: 11025 },
        Equivalence { pi_rad: 1., rad: PI, hz: 5512.5, rate: 11025 },
        Equivalence { pi_rad: -1., rad: -PI, hz: -5512.5, rate: 11025 },
        Equivalence { pi_rad: 2., rad: 2.*PI, hz: 11025., rate: 11025 },
        Equivalence { pi_rad: -2., rad: -2.*PI, hz: -11025., rate: 11025 },
        Equivalence { pi_rad: 300., rad: 300.*PI, hz: 150., rate: 1 },
        Equivalence { pi_rad: -300., rad: -300.*PI, hz: -150., rate: 1 },
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
            (fa + fb).get_pi_rad()
        );
        assert_roughly_equal(
            fa.get_pi_rad() - fb.get_pi_rad(),
            (fa - fb).get_pi_rad()
        );
        assert_roughly_equal(
            fa.get_pi_rad() * fb.get_pi_rad(),
            (fa * fb).get_pi_rad()
        );
        assert_roughly_equal(
            fa.get_pi_rad() / fb.get_pi_rad(),
            (fa / fb).get_pi_rad()
        );

        // Operators against f32

        assert_roughly_equal(
            fa.get_pi_rad() * b,
            (fa * b).get_pi_rad()
        );
        assert_roughly_equal(
            fa.get_pi_rad() / b,
            (fa / b).get_pi_rad()
        );

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
        let a: u32 = 23456;
        let b: u32 = 12345;

        let fa = Rate::hz(a);
        let fb = Rate::hz(b);

        // Operators against Rates

        assert!(fa.get_hz() + fb.get_hz() == (fa + fb).get_hz());
        assert!(fa.get_hz() - fb.get_hz() == (fa - fb).get_hz());
        assert!(fa.get_hz() * fb.get_hz() == (fa * fb).get_hz());
        assert!(fa.get_hz() / fb.get_hz() == (fa / fb).get_hz());

        // Operators against u32

        assert!(fa.get_hz() * b == (fa * b).get_hz());
        assert!(fa.get_hz() / b == (fa / b).get_hz());

        // Assign operators against rates

        let fb = Rate::hz(b);

        let mut fa = Rate::hz(a);
        fa += fb;
        assert!(fa.get_hz() == a + b);

        let mut fa = Rate::hz(a);
        fa -= fb;
        assert!(fa.get_hz() == a - b);

        let mut fa = Rate::hz(a);
        fa *= fb;
        assert!(fa.get_hz() == a * b);

        let mut fa = Rate::hz(a);
        fa /= fb;
        assert!(fa.get_hz() == a / b);

        // Assign operators against f32

        let mut fa = Rate::hz(a);
        fa *= b;
        assert!(fa.get_hz() == a * b);

        let mut fa = Rate::hz(a);
        fa /= b;
        assert!(fa.get_hz() == a / b);
    }

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn test_rate_overflow() {
        Rate::hz(1) - Rate::hz(2);
    }

    #[test]
    #[should_panic]
    fn test_rate_negative() {
        Rate::hz(-1);
    }

    #[test]
    #[allow(unused_assignments, unused_mut)]
    fn test_rate_copying() {
        let mut a = Rate::hz(123_u32);
        let mut b = Rate::hz(456_u32);

        b = a;

        assert!(b.get_hz() == 123);
        assert!(b.get_hz() == 123);
    }
}
