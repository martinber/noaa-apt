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

#[cfg(test)]
mod tests {

    use super::*;

    pub fn test_gcd() {
        assert_eq!(gcd(346, 1), 1);
        assert_eq!(gcd(123, 234), 3);
        assert_eq!(gcd(123, 23), 1);
        assert_eq!(gcd(10012, 50060), 10012);
    }
}
