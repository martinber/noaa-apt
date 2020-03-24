/// Functions for georreferencing images.
///
/// Most functions taken from
/// [APTDecoder.jl](https://github.com/Alexander-Barth/APTDecoder.jl)
///
/// Consider this file MIT licensed.
///
/// MIT License
///
/// Copyright (c) 2019-2020 Alexander Barth, Martin Bernardi
///
/// Permission is hereby granted, free of charge, to any person obtaining a copy
/// of this software and associated documentation files (the "Software"), to deal
/// in the Software without restriction, including without limitation the rights
/// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
/// copies of the Software, and to permit persons to whom the Software is
/// furnished to do so, subject to the following conditions:
///
/// The above copyright notice and this permission notice shall be included in all
/// copies or substantial portions of the Software.
///
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
/// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
/// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
/// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
/// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
/// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
/// SOFTWARE.
///
use std::f32::consts::PI;

use chrono::prelude::*;

/// Compute the great-circle distance between two points
///
/// The units of all input and output parameters are degrees.
fn distance(lat1: f32, lon1: f32, lat2: f32, lon2: f32) -> f32 {
    // https://en.wikipedia.org/w/index.php?title=Great-circle_distance&oldid=749078136#Computational_formulas

    // Convert to radians
    let delta_lon = PI/180. * (lon2 - lon1);
    let lat1 = PI/180. * lat1;
    let lat2 = PI/180. * lat2;

    let cos_central_angle = lat1.sin()*lat2.sin()
                            + lat1.cos()*lat2.cos()*delta_lon.cos();

    let cos_central_angle = cos_central_angle.max(-1.).min(1.);

    180./PI * cos_central_angle.acos()
}

/// Compute azimuth of line between two points.
///
/// The angle between the line segment defined by the points (`lat1`,`lon1`)
/// and (`lat2`,`lon2`) and the North.
///
/// The units of all input and output parameters are degrees.
fn azimuth(lat1: f32, lon1: f32, lat2: f32, lon2: f32) -> f32 {
    // https://en.wikipedia.org/w/index.php?title=Azimuth&oldid=750059816#Calculating_azimuth

    // Convert to radians
    let delta_lon = PI/180. * (lon2 - lon1);
    let lat1 = PI/180. * lat1;
    let lat2 = PI/180. * lat2;

    let azimuth = delta_lon.sin().atan2(lat1.cos()*lat2.tan() - lat1.sin()*delta_lon.cos());

    180./PI * azimuth
}

/// Compute the coordinates of the end-point of a displacement on a sphere.
///
/// `lat`,`lon` are the coordinates of the starting point, `range` is the
/// covered distance of the displacements along a great circle and `azimuth` is
/// the direction of the displacement relative to the North.
///
/// The units of all input and output parameters are degrees.
///
/// This function can also be used to define a spherical coordinate system with
/// rotated poles.
fn reckon(lat: f32, lon: f32, range: f32, azimuth: f32) -> (f32, f32) {

    // Based on reckon from Alexander Barth
    // https://sourceforge.net/p/octave/mapping/ci/3f19801d4b93d3b3923df9fa62d268660e5cb4fa/tree/inst/reckon.m
    // relicenced to LGPL-v3

    let deg2rad = PI/180.;

    let lat = lat * deg2rad;
    let lon = lon * deg2rad;
    let range = range * deg2rad;
    let azimuth = azimuth * deg2rad;

    let mut tmp = lat.sin() * range.cos() + lat.cos() * range.sin() * azimuth.cos();

    // clip tmp to -1 and 1
    tmp = tmp.max(-1.).min(1.);

    let lato = PI/2. - tmp.acos();

    let cos_y = (range.cos() - lato.sin() * lat.sin()) / (lato.cos() * lat.cos());
    let sin_y = azimuth.sin() * range.sin() / lato.cos();

    let y = sin_y.atan2(cos_y);

    let mut lono = lon + y;

    // bring the lono in the interval [-π π[

    lono = (lono + PI) % (2.*PI) - PI;

    // convert to degrees

    (lato/deg2rad, lono/deg2rad)
}


#[cfg(test)]
mod tests {

    use super::*;
    use approx::assert_abs_diff_eq;

    // Checks for equality allowing a difference of epsilon
    // assert_abs_diff_eq!(a, b, epsilon);

    #[test]
    fn test_distance() {
        let tolerance = 0.001; // Degrees
        assert_abs_diff_eq!(distance(  0.,   0.,   0.,  30.),  30., epsilon = tolerance);
        assert_abs_diff_eq!(distance(  0.,   0.,  30.,   0.),  30., epsilon = tolerance);
        assert_abs_diff_eq!(distance(  0.,   0., -30.,   0.),  30., epsilon = tolerance);
        assert_abs_diff_eq!(distance( 30.,   0.,   0.,   0.),  30., epsilon = tolerance);
        assert_abs_diff_eq!(distance(-30.,   0.,   0.,   0.),  30., epsilon = tolerance);
        assert_abs_diff_eq!(distance(  0.,  30.,   0.,   0.),  30., epsilon = tolerance);
        assert_abs_diff_eq!(distance(  0.,   0., 180.,   0.), 180., epsilon = tolerance);
        assert_abs_diff_eq!(distance(  0.,   0.,   0., 180.), 180., epsilon = tolerance);
        assert_abs_diff_eq!(distance(  0.,   0.,   0.,-180.), 180., epsilon = tolerance);
        assert_abs_diff_eq!(distance( 60.,   0.,  80., 180.),  40., epsilon = tolerance);

        // The function is less precise for small angles
        let tolerance = 0.036; // Roughly the angular distance of a pixel
        assert_abs_diff_eq!(distance(  0.,   0.,   0., 0.1 ), 0.1 , epsilon = tolerance);
        assert_abs_diff_eq!(distance( 40.,  40.,  40.,  40.),   0., epsilon = tolerance);
        assert_abs_diff_eq!(distance(  0.,   0.,   0., 360.),   0., epsilon = tolerance);
    }

    #[test]
    fn test_azimuth() {
        let tolerance = 0.001; // Degrees
        assert_abs_diff_eq!(azimuth(  0.,   0.,   0.,  30.),  90., epsilon = tolerance);
        assert_abs_diff_eq!(azimuth(  0.,   0.,  30.,   0.),   0., epsilon = tolerance);
        assert_abs_diff_eq!(azimuth(  0.,   0., -30.,   0.), 180., epsilon = tolerance);
        assert_abs_diff_eq!(azimuth( 30.,   0.,   0.,   0.), 180., epsilon = tolerance);
        assert_abs_diff_eq!(azimuth(-30.,   0.,   0.,   0.),   0., epsilon = tolerance);
        assert_abs_diff_eq!(azimuth(  0.,  30.,   0.,   0.), -90., epsilon = tolerance);
    }

    #[test]
    fn test_reckon() {
        // Test against the distance() and azimuth() functions

        let tolerance = 0.001; // Degrees

        let test_values = [
            (   0.,    0.,  10.,   0.),
            ( 180., -100.,  50.,  40.),
            (  20.,   30.,  10., 170.),
            ( 123.,  234.,  34.,  99.),
            (  20.,   30.,  10., 300.),
        ];

        for test in test_values.iter() {
            let (lat1, lon1, dist, az) = *test;
            let (lat2, lon2) = reckon(lat1, lon1, dist, az);
            println!("{}, {}", lat2, lon2);
            assert_abs_diff_eq!(distance(lat1, lon1, lat2, lon2), dist, epsilon = tolerance)
        }

    }

    /// Load a NOAA 18 test Satrec object from `test_tle.txt`.
    fn load_test_sat(name: &str) -> satellite::io::Satrec {
        let (sats, _errors) = satellite::io::parse_multiple(include_str!("test_tle_2020-01.txt"));
        sats.iter().find(|&sat| sat.name == Some(name.to_string()))
            .expect(&format!("{} not found in test TLE file", name)).clone()
        // TODO: Replace `.expect(format!` with `unwrap_or_else(|_| panic!("Some useful error"))`
        // On other files too
    }

    /// Check the satellite library against a known TLE and known satellite
    /// positions.
    ///
    /// The test TLE file is from January 2020. It is not necessary to update
    /// it, but the tests should use dates not too far away from that point of
    /// time.
    ///
    /// Predictions are more precise when asking for moments close to the TLE
    /// epoch (when the TLE was downloaded), so on these tests I've written
    /// different tolerances for each case. Keep in mind that a degree is at
    /// most 111km, each pixel on the image is 4km (around 0.036 degrees).
    ///
    /// Tolerances are roughly the differences I've seen when first testing
    /// this, leaving them I can see roughly what to expect from this library
    /// and I can check for regressions on it.
    ///
    /// Reference values were calculated using `predict`:
    /// https://github.com/martinber/predict
    ///
    /// This fork of `predict` shows latitude and longitude with more decimal
    /// places. Usage:
    ///
    ///     > predict -t ./test_tle_2020-01.txt -f "NOAA 18" 1580000000 1580000000
    ///     1580000000 Sun 26Jan20 00:53:20  -59  339  13.051 124.959  180  11967  75666 *
    ///
    /// Format (see /docs/pdf/predict.pdf):
    ///
    ///     timestamp day date time alt az orbit_phase lat long range orbit sunlight
    ///
    #[test]
    fn test_against_predict() {

        // Known results, we test against each one of these.
        // Everything in degrees. Fields:
        // (satellite, timestamp, latitude, longitude, tolerance)
        let test_values = [
            ("NOAA 15", 1577836800, -22.135, 103.093, 0.005), // 2020-01-01
            ("NOAA 18", 1580257671, -23.131, 125.410, 0.005), // 2020-01-28
            ("NOAA 19", 1580000000, -16.414,  66.815, 0.005), // 2020-01-26
            ("NOAA 15", 1590000000, -53.152,  19.884, 0.036), // 2020-05-20
            ("NOAA 18", 1565395200,  68.577, 287.984, 0.05 ), // 2019-08-10
            ("NOAA 15", 1672531200, -79.203,  64.941, 1.   ), // 2023-01-01
            ("NOAA 19", 1514764800, -36.389,  46.125, 1.   ), // 2018-01-01
        ];

        for test in test_values.iter() {
            let tolerance = test.4; // Degrees
            let mut sat = load_test_sat(test.0);
            let time = chrono::Utc.timestamp(test.1, 0); // 0 milliseconds
            let result = satellite::propogation::propogate_datetime(&mut sat, time).unwrap();
            let gmst = satellite::propogation::gstime::gstime_datetime(time);
            let sat_pos = satellite::transforms::eci_to_geodedic(&result.position, gmst);

            let lat = sat_pos.latitude * satellite::constants::RAD_TO_DEG;
            let lon = sat_pos.longitude * satellite::constants::RAD_TO_DEG;

            assert_abs_diff_eq!(lat, test.2, epsilon = tolerance);
            // Predict gives longitudes from 0:360, satellite-rs from -180:180
            assert_abs_diff_eq!((lon + 360.) % 360., test.3, epsilon = tolerance);
        }

    }

    /// Download latest TLEs and confirm current position of satellites.
    ///
    /// Test disabled by default as this is not reproducible I guess. Only
    /// compares the position of NOAA 15.
    ///
    /// N2YO.com provides a REST API where you can get the current position of
    /// satellites: https://www.n2yo.com/api/
    #[test] #[ignore]
    fn test_against_n2yo() {

        use std::env;

        /// Return current satellite position from N2YO.com.
        ///
        /// Returns latitude, longitude and Unix timestamp.
        fn get_n2yo_pos(satid: u32) -> (f32, f32, i64) {

            let api_key: String = env::var("N2YO_KEY")
                .expect("Provide an N2YO.com API key with N2YO_KEY=ASDHA... cargo test...");

            let url = format!(
                "https://www.n2yo.com/rest/v1/satellite/positions/{}/0/0/0/1/&apiKey={}",
                satid,
                api_key.as_str()
            );

            let json = reqwest::blocking::get(url.as_str())
                .expect("Error doing request to N2YO.com")
                .text()
                .expect("Error getting response text from N2Y0.com");

            #[derive(serde::Serialize, serde::Deserialize)]
            struct Info {
                satname: String,
                satid: u32,
                transactionscount: u32,
            }

            #[derive(serde::Serialize, serde::Deserialize)]
            struct Position {
                satlatitude: f32,
                satlongitude: f32,
                sataltitude: f32,
                azimuth: f32,
                elevation: f32,
                ra: f32,
                dec: f32,
                timestamp: i64,
            }

            #[derive(serde::Serialize, serde::Deserialize)]
            struct Data {
                info: Info,
                positions: Vec<Position>,
            }

            let data: Data = serde_json::from_str(json.as_str())
                .expect("Error parsing JSON");

            (
                data.positions[0].satlatitude,
                data.positions[0].satlongitude,
                data.positions[0].timestamp,
            )
        }

        /// Download latest TLE and calculate satellite position.
        ///
        /// name argument is e.g. "NOAA 15".
        fn calculate_tle_pos(name: &str, timestamp: i64) -> (f32, f32) {
            let tle = reqwest::blocking::get("https://celestrak.com/NORAD/elements/weather.txt")
                .expect("Error doing request to celestrak.com")
                .text()
                .expect("Error getting response text from celestrak.com");

            let (sats, _errors) = satellite::io::parse_multiple(tle.as_str());
            let mut sat = sats.iter().find(|&sat| sat.name == Some(name.to_string()))
                .expect(&format!("{} not found in weather.txt TLE file", name)).clone();

            let time = chrono::Utc.timestamp(timestamp, 0); // 0 milliseconds
            let result = satellite::propogation::propogate_datetime(&mut sat, time).unwrap();
            let gmst = satellite::propogation::gstime::gstime_datetime(time);
            let sat_pos = satellite::transforms::eci_to_geodedic(&result.position, gmst);

            let lat = sat_pos.latitude * satellite::constants::RAD_TO_DEG;
            let lon = sat_pos.longitude * satellite::constants::RAD_TO_DEG;

            (lat as f32, lon as f32)
        }

        let noaa_15_id = 25338;
        // let noaa_18_id = 28654;
        // let noaa_19_id = 33591;

        let n2yo_pos = get_n2yo_pos(noaa_15_id);
        // n2y0_pos.2 is the timestamp
        let tle_pos = calculate_tle_pos("NOAA 15", n2yo_pos.2);

        println!("{:?}", n2yo_pos);
        println!("{:?}", tle_pos);

        let tolerance = 0.036 * 5.; // Roughly the angular distance of 5 pixels
        assert_abs_diff_eq!(n2yo_pos.0, tle_pos.0, epsilon = tolerance);
        assert_abs_diff_eq!(n2yo_pos.1, tle_pos.1, epsilon = tolerance);
    }
}
