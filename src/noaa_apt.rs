//! High level decoding and resampling interface.
//!
//! Used by both the command-line and GUI versions of the program.

pub use crate::decode::{decode, FINAL_RATE, PX_PER_ROW};
pub use crate::resample::resample;

use std::path::Path;

use crate::context::Context;
use crate::dsp::{Signal, Rate};
use crate::err;
use crate::dsp;
use crate::map;
use crate::misc;
use crate::processing;
use crate::telemetry;
use crate::wav;

pub type Pixel = image::Rgb<u8>;
pub type Image = image::RgbImage;

/// Available settings for contrast adjustment.
#[derive(Clone, Debug)]
pub enum Contrast {
    /// From telemetry bands, requires syncing to be enabled.
    Telemetry,

    /// Takes only a given percent of the samples, clamping the rest. Something
    /// like a percentile.
    Percent(f32),

    /// Don't do anything, map the minimum value to zero and the maximum value
    /// to 255
    MinMax,
}

/// Available rotation settings.
#[derive(Clone, Debug)]
pub enum Rotate {
    Orbit,
    No,
    Yes
}

/// Reference time.
///
/// Indicates start or end time of recording. Sometimes we have the recording
/// start time (indicated by the filename) and sometimes we have the recording
/// end time (indicated by the file modification timestamp).
#[derive(Clone, Debug)]
pub enum RefTime {
    Start(chrono::DateTime<chrono::Utc>),
    End(chrono::DateTime<chrono::Utc>),
}

/// Settings that need orbit calculations.
#[derive(Clone, Debug)]
pub struct OrbitSettings {
    pub sat_name: SatName,
    pub custom_tle: Option<String>,
    pub ref_time: RefTime,
    pub draw_map: Option<MapSettings>,
}

/// Settings related to map overlays.
#[derive(Clone, Debug)]
pub struct MapSettings {
    pub yaw: f64,
    pub hscale: f64,
    pub vscale: f64,
}

/// Available satellites enum.
#[derive(Clone, Debug)]
pub enum SatName {
    Noaa15,
    Noaa18,
    Noaa19,
}

/// Load WAV from file.
///
/// Returns the Signal and its sample rate.
pub fn load(input_filename: &Path) -> err::Result<(Signal, Rate)> {
    let (signal, input_spec) = wav::load_wav(input_filename)?;

    Ok((signal, Rate::hz(input_spec.sample_rate)))
}

pub fn process(
    context: &mut Context,
    signal: &Signal,
    contrast_adjustment: Contrast,
    rotate: Rotate,
    orbit: Option<OrbitSettings>
) -> err::Result<Image> {


    let (low, high) = match contrast_adjustment {
        Contrast::Telemetry => {
            context.status(0.1, "Adjusting contrast from telemetry".to_string());

            let telemetry = telemetry::read_telemetry(context, &signal)?;

            let low = telemetry.get_wedge_value(9, None);
            let high = telemetry.get_wedge_value(8, None);

            (low, high)
        },
        Contrast::Percent(p) => {
            context.status(0.1,
               format!("Adjusting contrast using {} percent", p * 100.)
            );
            misc::percent(&signal, p)?
        },
        Contrast::MinMax => {
            context.status(0.1, "Mapping values (no contrast adjustment)".to_string());
            let low: f32 = *dsp::get_min(&signal)?;
            let high: f32 = *dsp::get_max(&signal)?;

            (low, high)
        }
    };

    // --------------------

    context.status(0.3, "Generating image".to_string());

    let height = signal.len() as u32 / PX_PER_ROW;

    type LumaImage = image::ImageBuffer::<image::Luma<u8>, Vec<u8>>;
    use image::ConvertBuffer;

    let mut img: Image = LumaImage::from_vec(
            PX_PER_ROW, height, map(&signal, low, high))
        .map(|b| b.convert()) // Convert to RGB
        .ok_or(err::Error::Internal(
            "Could not create image, wrong buffer length".to_string()))?;

    // --------------------

    if let Some(orbit_settings) = orbit.clone() {
        let tle = match orbit_settings.custom_tle {
            Some(t) => t,
            None => misc::get_current_tle()?,
        };

        if let Some(map_settings) = orbit_settings.draw_map {

            context.status(0.5, "Drawing map".to_string());

            map::draw_map(
                &mut img,
                orbit_settings.ref_time,
                map_settings,
                orbit_settings.sat_name,
                tle
            )?;
        }
    }

    // --------------------

    match rotate {
        Rotate::Yes => {
            context.status(0.90, "Rotating output image".to_string());
            img = processing::rotate(&img)?;
        },
        Rotate::Orbit => {
            if let Some(orbit_settings) = orbit.clone() {
                let tle = match orbit_settings.custom_tle {
                    Some(t) => t,
                    None => misc::get_current_tle()?,
                };
                // TODO
            } else {
                return Err(err::Error::Internal(
                    "Can't rotate from orbit if none provided".to_string()));
            }
        },
        Rotate::No => {},
    }

    Ok(img)
}


/// Maps float signal values to `u8`.
///
/// `low` becomes 0 and `high` becomes 255. Values are clamped to prevent `u8`
/// overflow.
fn map(signal: &Signal, low: f32, high: f32) -> Vec<u8> {

    let range = high - low;
    let raw_data: Vec<u8> = signal.iter()
        .map(|x|
             // Map and clamp between 0 and 255 using min() and max()
             ((x - low) / range * 255.).max(0.).min(255.).round() as u8
        ).collect();

    raw_data
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_map() {
        let expected: Vec<u8> = vec![
            0, 0, 0, 0, 1, 2, 50, 120, 200, 255, 255, 255];
        let test_values: Signal = vec![
            -10., -5., -1., 0., 1., 2.4, 50., 120., 199.6, 255., 256., 300.];

        // Shift values somewhere
        let shifted_values: Signal =
            test_values.iter().map(|x| x * 123.123 - 234.234).collect();

        // See where 0 and 255 end up after that
        let low = 0. * 123.123 - 234.234;
        let high = 255. * 123.123 - 234.234;

        assert_eq!(expected, map(&shifted_values, low, high));
    }
}
