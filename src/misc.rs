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
        use rgsl::bessel::I0 as gsl_bessel;
        // Compare my implementation of the Bessel function with the one in GSL
        let tolerance = 0.001; // 0.1%

        // Iterate from 0 to 7 with steps of 0.01
        for i in 0..700 {
            let i = i as f32 / 100.;

            println!("my_bessel({}) = {}", i, bessel_i0(i as f32));
            println!("gsl_bessel({}) = {}", i, gsl_bessel(i as f64));
            assert!(((bessel_i0(i as f32) - gsl_bessel(i as f64) as f32) /
                    bessel_i0(i as f32)).abs() < tolerance);
        }
    }
}

