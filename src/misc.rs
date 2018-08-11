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
