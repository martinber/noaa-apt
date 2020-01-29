use chrono::prelude::*;

/*
/// Compute the great-circle distance between two points
///
/// The units of all input and output parameters are degrees.
///
/// From [APTDecoder.jl](https://github.com/Alexander-Barth/APTDecoder.jl)
///
/// MIT License
///
/// Copyright (c) 2019 Alexander Barth, Martin Bernardi
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
"""
function distance(lat1,lon1,lat2,lon2)
    #https://en.wikipedia.org/w/index.php?title=Great-circle_distance&oldid=749078136#Computational_formulas

    Δλ = π/180 * (lon2 - lon1)
    ϕ1 = π/180 * lat1
    ϕ2 = π/180 * lat2

    cosΔσ = sin(ϕ1)*sin(ϕ2) + cos(ϕ1)*cos(ϕ2)*cos(Δλ)

    eins = one(cosΔσ)
    cosΔσ = max(min(cosΔσ,eins),-eins)
    Δσ = acos(cosΔσ)
    return 180/π * Δσ
end

"""
    az = azimuth(lat1,lon1,lat2,lon2)
Compute azimuth, i.e. the angle between the line segment defined by the points (`lat1`,`lon1`) and (`lat2`,`lon2`)
and the North.
The units of all input and output parameters are degrees.
/// From [APTDecoder.jl](https://github.com/Alexander-Barth/APTDecoder.jl)
///
/// MIT License
///
/// Copyright (c) 2019 Alexander Barth, Martin Bernardi
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
function azimuth(lat1,lon1,lat2,lon2)
    # https://en.wikipedia.org/w/index.php?title=Azimuth&oldid=750059816#Calculating_azimuth

    Δλ = π/180 * (lon2 - lon1)
    ϕ1 = π/180 * lat1
    ϕ2 = π/180 * lat2

    α = atan(sin(Δλ), cos(ϕ1)*tan(ϕ2) - sin(ϕ1)*cos(Δλ))
    return 180/π * α
end

# Base on reckon from myself
# https://sourceforge.net/p/octave/mapping/ci/3f19801d4b93d3b3923df9fa62d268660e5cb4fa/tree/inst/reckon.m
# relicenced to LGPL-v3

"""
    lato,lono = reckon(lat,lon,range,azimuth)
Compute the coordinates of the end-point of a displacement on a
sphere. `lat`,`lon` are the coordinates of the starting point, `range`
is the covered distance of the displacements along a great circle and
`azimuth` is the direction of the displacement relative to the North.
The units of all input and output parameters are degrees.
This function can also be used to define a spherical coordinate system
with rotated poles.
"""
function reckon(lat,lon,range,azimuth)

    # convert to radian
    rad2deg = π/180

    lat = lat*rad2deg
    lon = lon*rad2deg
    range = range*rad2deg
    azimuth = azimuth*rad2deg

    tmp = sin.(lat).*cos.(range) + cos.(lat).*sin.(range).*cos.(azimuth)

    # clip tmp to -1 and 1
    eins = one(eltype(tmp))
    tmp = max.(min.(tmp,eins),-eins)

    lato = π/2 .- acos.(tmp)

    cos_γ = (cos.(range) - sin.(lato).*sin.(lat))./(cos.(lato).*cos.(lat))
    sin_γ = sin.(azimuth).*sin.(range)./cos.(lato)

    γ = atan.(sin_γ,cos_γ)

    lono = lon .+ γ

    # bring the lono in the interval [-π π[

    lono = mod.(lono .+ π,2*π) .- π

    # convert to degrees

    lono = lono/rad2deg
    lato = lato/rad2deg

    return lato,lono
end
*/


#[cfg(test)]
mod tests {

    use super::*;
    use approx::assert_abs_diff_eq;

    // Checks for equality allowing a difference of epsilon
    // assert_abs_diff_eq!(a, b, epsilon);

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

    /*
    /// Download latest TLEs and confirm current position of satellites.
    ///
    /// Test disabled by default as this is not reproducible I guess.
    ///
    /// N2YO.com provides a REST API where you can get the current position of
    /// satellites: https://www.n2yo.com/api/
    #[test] #[ignore]
    fn test_against_n2yo() {

        // NORAD IDs
        let noaa_15_id = 25338;
        let noaa_18_id = 28654;
        let noaa_19_id = 33591;

        // TODO

        // https://celestrak.com/NORAD/elements/weather.txt
    }
    */

}
