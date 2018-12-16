// Lookup table for numbers used in Bessel function.
// 1 / (n! * 2^n)^2
const BESSEL_TABLE: [f32; 20] = [
    1.0,
    0.25,
    0.015625,
    0.00043402777777777775,
    6.781684027777777e-06,
    6.781684027777778e-08,
    4.709502797067901e-10,
    2.4028075495244395e-12,
    9.385966990329842e-15,
    2.896903392077112e-17,
    7.242258480192779e-20,
    1.4963343967340453e-22,
    2.5978027721077174e-25,
    3.842903509035085e-28,
    4.9016626390753635e-31,
    5.4462918211948485e-34,
    5.318644356635594e-37,
    4.60090342269515e-40,
    3.5500798014623073e-43,
    2.458504017633177e-46
];

/// Greatest common divisor
///
/// Used to choose interpolation (L) and decimation (M) factors for
/// interpolation.
pub fn gcd(a: u32, b: u32) -> u32 {
    let mut a = a;
    let mut b = b;
    while a != 0 {
        let c = a;
        a = b % c;
        b = c;
    }
    b
}

/// First Kind modified Bessel function of order zero.
///
/// From https://dsp.stackexchange.com/questions/37714/kaiser-window-approximation/37715#37715
pub fn bessel_i0(x: f32) -> f32 {
    let mut result: f32 = 0.;
    let limit: usize = 8;

    for k in (1..=limit).rev() {
        result += BESSEL_TABLE[k];
        result *= x.powi(2);
    }

    result + 1.
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    pub fn test_gcd() {
        assert_eq!(gcd(346, 1), 1);
        assert_eq!(gcd(123, 234), 3);
        assert_eq!(gcd(123, 23), 1);
        assert_eq!(gcd(10012, 50060), 10012);
    }

    #[test]
    pub fn test_bessel_i0() {
        let tolerance = 0.001; // 0.1%

        // Compare values with results from GNU Octave
        assert_relative_eq!(bessel_i0(0.),  1.00000000000000, max_relative = tolerance);
        assert_relative_eq!(bessel_i0(0.5), 1.06348337074132, max_relative = tolerance);
        assert_relative_eq!(bessel_i0(1.),  1.26606587775201, max_relative = tolerance);
        assert_relative_eq!(bessel_i0(1.5), 1.64672318977289, max_relative = tolerance);
        assert_relative_eq!(bessel_i0(2.),  2.27958530233607, max_relative = tolerance);
        assert_relative_eq!(bessel_i0(2.5), 3.28983914405012, max_relative = tolerance);
        assert_relative_eq!(bessel_i0(3.),  4.88079258586502, max_relative = tolerance);
        assert_relative_eq!(bessel_i0(3.5), 7.37820343222548, max_relative = tolerance);
        assert_relative_eq!(bessel_i0(4.),  11.3019219521363, max_relative = tolerance);
        assert_relative_eq!(bessel_i0(4.5), 17.4811718556093, max_relative = tolerance);
        assert_relative_eq!(bessel_i0(5.),  27.2398718236044, max_relative = tolerance);
        assert_relative_eq!(bessel_i0(5.5), 42.6946451518478, max_relative = tolerance);
        assert_relative_eq!(bessel_i0(6.),  67.2344069764780, max_relative = tolerance);
        assert_relative_eq!(bessel_i0(6.5), 106.292858243996, max_relative = tolerance);
        assert_relative_eq!(bessel_i0(7.),  168.593908510290, max_relative = tolerance);
    }
}
