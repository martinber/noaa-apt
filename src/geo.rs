use chrono::prelude::*;


#[cfg(test)]
mod tests {

    use super::*;
    use approx::assert_abs_diff_eq;

    // Checks for equality allowing a difference of epsilon
    // assert_abs_diff_eq!(a, b, epsilon);

    /// Load a NOAA 18 test Satrec object from `test_tle.txt`.
    fn load_noaa_18() -> satellite::io::Satrec {
        let (sats, errors) = satellite::io::parse_multiple(include_str!("test_tle.txt"));
        sats.iter().find(|&sat| sat.name == Some("NOAA 18".to_string()))
            .expect("NOAA 16 not found in test_tle.txt").clone()
    }

    /// Check the satellite library against a known TLE and known satellite
    /// positions.
    /// Known positions calculated using predict.
    #[test]
    fn test_known_results() {

        let mut noaa_18 = load_noaa_18();

        let time = chrono::Utc.timestamp(1580257671, 0); // 0 milliseconds
        let result = satellite::propogation::propogate_datetime(&mut noaa_18, time).unwrap();
        let gmst = satellite::propogation::gstime::gstime_datetime(time);

        let sat_pos = satellite::transforms::eci_to_geodedic(&result.position, gmst);

        let lat = sat_pos.latitude * satellite::constants::RAD_TO_DEG;
        let lon = sat_pos.longitude * satellite::constants::RAD_TO_DEG;

        let tolerance = 0.001; // Degrees
        assert_abs_diff_eq!(lat, -23.131, epsilon = tolerance);
        assert_abs_diff_eq!(lon, 125.410, epsilon = tolerance);

        assert!(false);

    }

    fn test_against_predict() {

        // https://github.com/martinber/predict/blob/master/docs/pdf/predict.pdf
        // https://github.com/martinber/predict
        //
        // predict -t ./predict.tle -f OSCAR-7 10000 10000
        // 10000 Thu 01Jan70 02:46:40  -35  251  252   -9  145   9432 -118771 *
        //
        // Epoch time (since 1970-01-01)
        // Day of week
        // Day of year
        // Time
        // Elevation in degrees
        // Azimuth
        // Orbital phase (modulo 256)
        // Latitude
        // Longitude
        // Slant range to ground station in km
        // Orbit number
        // Information about sunlight
    }

} 
