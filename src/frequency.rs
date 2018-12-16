use std::f32::consts::PI;

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

#[cfg(test)]
mod tests {

    use super::*;

    /// Check if two floats are equal given some margin of precision
    fn roughly_equal(a: &f32, b: &f32) -> bool {
        ulps_eq!(a, b, max_ulps = 10)
    }

    #[test]
    fn test_pi_radians() {
        let f = Frequency::pi_radians(0.1234);
        assert!(roughly_equal(&f.get_pi_radians(), &0.1234));
        let f = Frequency::pi_radians(1.);
        assert!(roughly_equal(&f.get_pi_radians(), &1.));
        let f = Frequency::pi_radians(0.);
        assert!(roughly_equal(&f.get_pi_radians(), &0.));

        // Test values larger than one and smaller than zero too
        let f = Frequency::pi_radians(123.);
        assert!(roughly_equal(&f.get_pi_radians(), &123.));
        let f = Frequency::pi_radians(-1.);
        assert!(roughly_equal(&f.get_pi_radians(), &-1.));
        let f = Frequency::pi_radians(-0.6);
        assert!(roughly_equal(&f.get_pi_radians(), &-0.6));
    }

    #[test]
    fn test_radians() {
        let f = Frequency::radians(1.1234);
        assert!(roughly_equal(&f.get_radians(), &1.1234));
        let f = Frequency::radians(PI);
        assert!(roughly_equal(&f.get_radians(), &PI));
        assert!(roughly_equal(&f.get_pi_radians(), &1.));
        let f = Frequency::radians(0.);
        assert!(roughly_equal(&f.get_radians(), &0.));
        assert!(roughly_equal(&f.get_pi_radians(), &0.));

        // Test values larger than pi and smaller than zero too
        let f = Frequency::radians(123.);
        assert!(roughly_equal(&f.get_radians(), &123.));
        let f = Frequency::radians(-PI);
        assert!(roughly_equal(&f.get_radians(), &-PI));
        assert!(roughly_equal(&f.get_pi_radians(), &-1.));
        let f = Frequency::radians(-2.6);
        assert!(roughly_equal(&f.get_radians(), &-2.6));
    }

    #[test]
    fn test_hertz() {
        let f = Frequency::hertz(2400., 11025.);
        assert!(roughly_equal(&f.get_hertz(11025.), &2400.));
        assert!(roughly_equal(&f.get_pi_radians(), &0.435374149659864));
        let f = Frequency::hertz(5512.5, 11025.);
        assert!(roughly_equal(&f.get_hertz(11025.), &5512.5));
        assert!(roughly_equal(&f.get_pi_radians(), &1.));
        let f = Frequency::hertz(0., 11025.);
        assert!(roughly_equal(&f.get_hertz(11025.), &0.));
        assert!(roughly_equal(&f.get_pi_radians(), &0.));

        // Test values larger than rate and smaller than zero too
        let f = Frequency::hertz(11025., 11025.);
        assert!(roughly_equal(&f.get_hertz(11025.), &11025.));
        assert!(roughly_equal(&f.get_pi_radians(), &2.));
        let f = Frequency::hertz(-5512.5, 11025.);
        assert!(roughly_equal(&f.get_hertz(11025.), &-5512.5));
        assert!(roughly_equal(&f.get_pi_radians(), &-1.));
        let f = Frequency::hertz(-2400., 11025.);
        assert!(roughly_equal(&f.get_hertz(11025.), &-2400.));
        assert!(roughly_equal(&f.get_pi_radians(), &-0.435374149659864));
    }
}
