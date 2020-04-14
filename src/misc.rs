//! Small things that don't fit anywhere else.

use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use chrono::prelude::*;
use log::{error, info, warn};

use crate::config::Settings;
use crate::dsp::{self, Signal};
use crate::err;
use crate::noaa_apt::RefTime;


/// Lookup table for numbers used in `bessel_i0()`
///
/// 1 / (n! * 2^n)^2
#[allow(clippy::excessive_precision, clippy::unreadable_literal)]
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

/// First Kind modified Bessel function of order zero.
///
/// From this
/// [post](https://dsp.stackexchange.com/questions/37714/kaiser-window-approximation/37715#37715).
pub fn bessel_i0(x: f32) -> f32 {
    let mut result: f32 = 0.;
    let limit: usize = 8;

    for k in (1..=limit).rev() {
        result += BESSEL_TABLE[k];
        result *= x.powi(2);
    }

    result + 1.
}

/// Check if there is an update to this program.
///
/// Takes a `String` with the current version being used.
///
/// Returns a tuple of a `bool` idicating if there are new updates and a
/// `String` with the latest version. Wrapped in `Option`, returns `None` if
/// there was a problem retrieving new versions and logs the error.
pub fn check_updates(current: &str) -> Option<(bool, String)> {
    let addr = format!("https://noaa-apt.mbernardi.com.ar/version_check?{}", current);

    let latest: Option<String> = match reqwest::blocking::get(addr.as_str()) {
        Ok(response) => {
            match response.text() {
                Ok(text) => {
                    Some(text.trim().to_string())
                }
                Err(e) => {
                    warn!("Error checking for updates: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            warn!("Error checking for updates: {}", e);
            None
        }
    };

    match latest {
        Some(latest) => {
            if latest.len() > 10 {
                warn!("Error checking for updates: Response too long");
                None
            } else {
                // Return true if there are updates
                Some((latest != current, latest))
            }
        }
        None => None,
    }
}

/// Returns lowest and highest values that fall inside the percent given.
///
/// Returns tuple of `(low, high)`. The values returned are approximate. The
/// percent given should be between 0 and 1.
///
/// Means that `percent` samples of the `Signal` are bigger than `low` and
/// smaller than `high`. Also, `(1 - percent) / 2` are smaller than `low` and
/// `(1 - percent) / 2` are bigger than `high`.
///
/// For example
/// -----------
///
/// - If the signal has values uniformly distributed between 0 and 1 and the
///   percent given is `0.50`, `low` will be 0.25 and `high` 0.75.
///
/// - If the signal has values uniformly distributed between 1 and 2 and the
///   percent given is `0.90`, `low` will be 1.05 and `high` 1.95.
///
/// How it works
/// ------------
///
/// Creates 1000 buckets, uniformly distributed from the minimum and maximum
/// values on `signal`. For each sample, increment one on the bucket the sample
/// falls in.
///
/// Finally count the values on each bucket and return an approximate value for
/// `low` and `high`
pub fn percent(signal: &Signal, percent: f32) -> err::Result<(f32, f32)> {

    if percent < 0. || percent > 1. {
        return Err(err::Error::Internal(
            "Percent given should be between 0 and 1".to_string())
        );
    }

    let remainder = (1. - percent) / 2.;

    // Amount of buckets
    let num_buckets: usize = 1000;

    // Count on samples that fall on each bucket
    let mut buckets: Vec<u32> = vec![0; num_buckets];

    // Range of input samples
    let min = dsp::get_min(&signal)?;
    let max = dsp::get_max(&signal)?;
    let total_range = max - min;

    // Get the index of the bucket where the sample falls in
    let get_bucket = |x: &f32| {
        (((x - min) / total_range * num_buckets as f32)
            .trunc() as usize)
            .max(0).min(num_buckets - 1) // Avoid going to an invalid bucket
    };

    // Count samples on each bucket
    for sample in signal {
        buckets[get_bucket(sample)] += 1;
    }

    // Find `low` and high`
    let mut accum = 0;
    let mut low_bucket = None;
    let mut high_bucket = None;
    for (bucket, count) in buckets.iter().enumerate() {
        accum += count;

        if low_bucket.is_none()
            && (accum as f32 / signal.len() as f32) > remainder {

            low_bucket = Some(bucket);

        } else if high_bucket.is_none()
            && (accum as f32 / signal.len() as f32) > 1. - remainder {

            high_bucket = Some(bucket);

        }
    }

    if high_bucket.is_none() {
        // Can happen if remainder is too close to zero, so the high_bucket
        // should be the last one.
        high_bucket = Some(num_buckets - 1);
    }

    Ok((low_bucket.unwrap() as f32 / num_buckets as f32 * total_range + min,
        high_bucket.unwrap() as f32 / num_buckets as f32 * total_range + min))

}

/// Read timestamp from file.
///
/// Returns the timestamp as the amount of seconds from the Unix epoch
/// (Jan 1, 1970, 0:00:00hs UTC). I ignore the nanoseconds precision.
pub fn read_timestamp(filename: &Path) -> err::Result<i64> {
    let metadata = fs::metadata(filename)
        .map_err(|e| err::Error::Internal(
            format!("Could not read metadata from input file: {}", e)
        ))?;

    // Read modification timestamp from file. The filetime library returns
    // the amount of seconds from the Unix epoch (Jan 1, 1970). I ignore the
    // nanoseconds precision.
    // I use the chrono library to convert seconds to date and time.
    // As far as I know the unix_seconds are relative to 0:00:00hs UTC, then
    // if I use chrono::Local I'm going to get time relative to my timezone.

    Ok(filetime::FileTime::from_last_modification_time(&metadata).unix_seconds())
}

/// Write timestamp to file.
///
/// The argument timestamp is the amount of seconds from the Unix epoch
/// (Jan 1, 1970, 0:00:00hs UTC).
pub fn write_timestamp(timestamp: i64, filename: &Path) -> err::Result<()> {
    filetime::set_file_mtime(
        filename,
        filetime::FileTime::from_unix_time(timestamp, 0),
    ).map_err(|_|
        err::Error::Internal("Could not write timestamp to file".to_string())
    )?;

    Ok(())
}

/// Infer recording time from filename and timestamp.
pub fn infer_ref_time(settings: &Settings, path: &Path) ->
    err::Result<RefTime>
{
    use chrono::{TimeZone, FixedOffset, Utc, NaiveDateTime};
    let filename: &str = path.file_name().and_then(std::ffi::OsStr::to_str).ok_or_else(||
        err::Error::Internal("Could not get filename".to_string()))?;
    match settings.prefer_timestamps {
        true => {
            return Ok(RefTime::End(Utc.timestamp(read_timestamp(&path)?, 0)));
        },
        false => {
            let offset_seconds = (settings.filename_timezone * 3600.) as i32;
            let timezone: FixedOffset = TimeZone::from_offset(&FixedOffset::east(offset_seconds));
            for format in settings.filename_formats.iter() {
                if let Ok(time) = timezone.datetime_from_str(filename, format) {
                    return Ok(RefTime::Start(time.with_timezone(&Utc)));
                }
            }
            warn!("Could not parse date and time from filename, using timestamp");
            return Ok(RefTime::End(Utc.timestamp(read_timestamp(&path)?, 0)));
        },
    }
}

/// Try downloading TLE from URL.
fn download_tle(addr: &str) -> err::Result<String> {
    Ok(reqwest::blocking::get(addr)?.text()?.to_string())
}

/// Try reading TLE from file.
fn read_from_file(filename: &Path) -> err::Result<String> {
    let mut file = fs::File::open(filename)?;
    let mut tle = String::new();
    file.read_to_string(&mut tle)?;

    Ok(tle)
}

/// Download, save and return TLE from URL.
///
/// Returns an error if unable to download TLE. Logs error message if unable to
/// save to file.
fn download_save_return_tle(addr: &str, filename: &Path) -> err::Result<String> {
    let tle = download_tle(addr)?;

    let mut file = match fs::File::create(filename) {
        Ok(f) => f,
        Err(e) => {
            error!("Could not cache TLE at {}: {}", filename.display(), e);
            return Ok(tle);
        },
    };
    if let Err(e) = file.write_all(tle.as_bytes()) {
        error!("Could not cache TLE at {}: {}", filename.display(), e);
        return Ok(tle);
    }

    Ok(tle)
}

/// Use cached TLE or download update from celestrak.com
///
/// Returns an error if no cached TLE and if unable to download TLE.
pub fn get_current_tle() -> err::Result<String> {
    let addr = "https://www.celestrak.com/NORAD/elements/weather.txt";

    // Load cached TLE

    if let Some(proj_dirs) = directories::ProjectDirs::from("ar.com.mbernardi", "", "noaa-apt") {

        let filename = proj_dirs.config_dir().join("weather.txt");

        match read_timestamp(&filename) {
            Ok(ts) => {
                let file_date = chrono::Utc.timestamp(ts, 0); // 0 milliseconds
                let now = chrono::Utc::now();
                if now - file_date < chrono::Duration::days(7) {
                    info!("Found recent cached TLE. Date: {}", file_date);
                    match read_from_file(&filename) {
                        Ok(tle) => return Ok(tle),
                        Err(e) => {
                            error!("Could not read cached TLE at {}: {}. \
                                   Downloading and caching new TLE", filename.display(), e);
                            return download_save_return_tle(addr, &filename);
                        }
                    }

                } else {
                    info!("Found outdated cached TLE. Date: {}
                          Downloading and caching new TLE", file_date);
                    return download_save_return_tle(addr, &filename);
                }
            },
            Err(e) => {
                warn!("Unable to read cached TLE timestamp: {}. Downloading \
                      and caching new TLE", e);
                return download_save_return_tle(addr, &filename);
            }
        }
    } else {
        // Descargar y devolver
        error!("Could not get system settings directory, can't cache downloaded TLE");
        return download_tle(addr);
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

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

    #[test]
    fn test_percent() {

        use std::iter::Iterator;

        // Use a vector integers from 0 to 10000. It's a quite bad test signal
        // because it has a uniform distribution. In practice signals have a
        // distribution closer to bell shape.
        let test_signal: Signal = (0..10000).map(|x| x as f32).collect();

        // Percent values to use
        let test_values: Vec<f32> = vec![1., 0.95, 0.90, 0.80, 0.50];

        for value in test_values {
            let (min, max) = percent(&test_signal, value).unwrap();

            // Percent of values that fall below min or fall above max
            let remainder = (1. - value) / 2.;

            // Allow 1% of error
            let min_remainder = remainder - 0.005;
            let max_remainder = remainder + 0.005;

            assert!(min / 10000. > min_remainder);
            assert!(min / 10000. < max_remainder);

            assert!(max / 10000. > 1. - max_remainder);
            assert!(max / 10000. < 1. - min_remainder);
        }
    }
}
