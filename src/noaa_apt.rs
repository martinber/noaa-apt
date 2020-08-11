//! High level decoding and resampling interface.
//!
//! Used by both the command-line and GUI versions of the program.

pub use crate::decode::{decode, FINAL_RATE, PX_PER_CHANNEL, PX_PER_ROW};
pub use crate::resample::resample;

use std::path::Path;

use log::warn;

use crate::context::Context;
use crate::dsp;
use crate::dsp::{Rate, Signal};
use crate::err;
use crate::map;
use crate::misc;
use crate::processing;
use crate::telemetry;
use crate::wav;

pub type Image = image::RgbaImage;

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

    /// Histogram equalization, per channel.
    /// See also: [Histogram equalization (wikipedia)](https://en.wikipedia.org/wiki/Histogram_equalization)
    Histogram,
}

/// Available rotation settings.
#[derive(Clone, Debug)]
pub enum Rotate {
    Orbit,
    No,
    Yes,
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
    pub countries_color: (u8, u8, u8, u8),
    pub states_color: (u8, u8, u8, u8),
    pub lakes_color: (u8, u8, u8, u8),
}

/// Available satellites enum.
#[derive(Clone, Debug)]
pub enum SatName {
    Noaa15,
    Noaa18,
    Noaa19,
}

impl ToString for SatName {
    fn to_string(&self) -> String {
        match self {
            SatName::Noaa15 => "NOAA 15".to_owned(),
            SatName::Noaa18 => "NOAA 18".to_owned(),
            SatName::Noaa19 => "NOAA 19".to_owned(),
        }
    }
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
    orbit: Option<OrbitSettings>,
) -> err::Result<Image> {
    let (low, high) = match contrast_adjustment {
        Contrast::Telemetry => {
            context.status(0.1, "Adjusting contrast from telemetry".to_string());

            let telemetry = telemetry::read_telemetry(context, &signal)?;

            let low = telemetry.get_wedge_value(9, None);
            let high = telemetry.get_wedge_value(8, None);

            (low, high)
        }
        Contrast::Percent(p) => {
            context.status(
                0.1,
                format!("Adjusting contrast using {} percent", p * 100.),
            );
            misc::percent(&signal, p)?
        }
        Contrast::MinMax | Contrast::Histogram => {
            context.status(0.1, "Mapping values".to_string());
            let low: f32 = *dsp::get_min(&signal)?;
            let high: f32 = *dsp::get_max(&signal)?;

            (low, high)
        }
    };

    // --------------------

    context.status(0.3, "Generating image".to_string());

    let height = signal.len() as u32 / PX_PER_ROW;

    use image::GrayImage;

    let mut img: GrayImage = GrayImage::from_vec(PX_PER_ROW, height, map(&signal, low, high))
        .ok_or_else(|| {
            err::Error::Internal("Could not create image, wrong buffer length".to_string())
        })?;

    if let Contrast::Histogram = contrast_adjustment {
        img = processing::histogram_equalization(&img)?;
    }

    let mut img: Image = image::DynamicImage::ImageLuma8(img).into_rgba(); // convert to RGBA
    let mut img_clone = img.clone();
    // --------------------

    // colorize
    for x in 0..PX_PER_CHANNEL {
        for y in 0..height {
            let val_pixel = img.get_pixel_mut(x, y);
            let irval_pixel = img_clone.get_pixel_mut(x + PX_PER_CHANNEL, y);

            let val = val_pixel[0];
            let irval = irval_pixel[0];

            let r;
            let g;
            let b;

            // Water identification
            if val < (13000 * 256 / 65536) as u8 {
                r = (8.0 + val as f32 * 0.2) as u8;
                g = (20.0 + val as f32 * 1.0) as u8;
                b = (50.0 + val as f32 * 0.75) as u8;
            }
            // Cloud/snow/ice identification
            // IR channel helps distinguish clouds and water, particularly in arctic areas
            else if irval > (35000 * 256 / 65536) as u8 {
                r = (irval as f32 * 0.5 + val as f32) as u8; // Average the two for a little better cloud distinction
                g = r;
                b = r;
            }
            // Vegetation identification
            else if val < (27000 * 256 / 65536) as u8 {
                // green
                r = (val as f32 * 0.8) as u8;
                g = (val as f32 * 0.9) as u8;
                b = (val as f32 * 0.6) as u8;
            }
            // Desert/dirt identification
            else if val <= (35000 * 256 / 65536) as u8 {
                // brown
                r = (val as f32 * 1.0) as u8;
                g = (val as f32 * 0.9) as u8;
                b = (val as f32 * 0.7) as u8;
            }
            // Everything else, but this was probably captured by the IR channel above
            else {
                // Clouds, snow, and really dry desert
                r = val;
                g = val;
                b = val;
            }

            // if (j < SPACE_WORDS || j >= SPACE_WORDS + CHANNEL_DATA_WORDS) {
            //     r = 0;
            //     g = 0;
            //     b = 0;
            //   }
            
            *val_pixel = image::Rgba([r, g, b, 255]);
            *irval_pixel = image::Rgba([r, g, b, 255]);
        }
    }

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
                tle,
            )?;
        }
    }

    // --------------------

    match rotate {
        Rotate::Yes => {
            context.status(0.90, "Rotating output image".to_string());
            img = processing::rotate(&img)?;
        }
        Rotate::Orbit => {
            if let Some(orbit_settings) = orbit {
                if processing::south_to_north_pass(&orbit_settings)? {
                    context.status(0.90, "Rotating output image".to_string());
                    img = processing::rotate(&img)?;
                }
            } else {
                warn!("Can't rotate automatically if no orbit information is provided");
            }
        }
        Rotate::No => {}
    }

    Ok(img)
}

/// Maps float signal values to `u8`.
///
/// `low` becomes 0 and `high` becomes 255. Values are clamped to prevent `u8`
/// overflow.
fn map(signal: &Signal, low: f32, high: f32) -> Vec<u8> {
    let range = high - low;
    let raw_data: Vec<u8> = signal
        .iter()
        .map(|x|
             // Map and clamp between 0 and 255 using min() and max()
             ((x - low) / range * 255.).max(0.).min(255.).round() as u8)
        .collect();

    raw_data
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_map() {
        let expected: Vec<u8> = vec![0, 0, 0, 0, 1, 2, 50, 120, 200, 255, 255, 255];
        let test_values: Signal = vec![
            -10., -5., -1., 0., 1., 2.4, 50., 120., 199.6, 255., 256., 300.,
        ];

        // Shift values somewhere
        let shifted_values: Signal = test_values.iter().map(|x| x * 123.123 - 234.234).collect();

        // See where 0 and 255 end up after that
        let low = 0. * 123.123 - 234.234;
        let high = 255. * 123.123 - 234.234;

        assert_eq!(expected, map(&shifted_values, low, high));
    }
}
