//! High level decoding and resampling interface.
//!
//! Used by both the command-line and GUI versions of the program.

pub use crate::decode::{decode, Pixel, Image, FINAL_RATE, PX_PER_ROW};
pub use crate::resample::resample;

use std::path::Path;

use log::{info, warn, debug};

use crate::context::Context;
use crate::dsp::{Signal, Rate, Freq};
use crate::err;
use crate::dsp;
use crate::misc;
use crate::processing;
use crate::telemetry;
use crate::wav;

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

pub enum Rotate {
    Orbit,
    No,
    Yes
}

/// Settings that need orbit calculations
pub struct OrbitSettings {
    custom_tle: Option<String>,
    start_time: chrono::DateTime<chrono::Utc>,
    draw_map: Option<MapSettings>,
}

/// Settings related to map overlays
pub struct MapSettings {
    yaw: f64,
    hscale: f64,
    vscale: f64,
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
            info!("Adjusting contrast from telemetry");

            let telemetry = telemetry::read_telemetry(context, &signal)?;

            let low = telemetry.get_wedge_value(9, None);
            let high = telemetry.get_wedge_value(8, None);

            (low, high)
        },
        Contrast::Percent(p) => {
            info!("Adjusting contrast using {} percent", p * 100.);
            misc::percent(&signal, p)?
        },
        Contrast::MinMax => {
            info!("Mapping values (no contrast adjustment)");
            let low: f32 = *dsp::get_min(&signal)?;
            let high: f32 = *dsp::get_max(&signal)?;

            (low, high)
        }
    };

    // --------------------

    // context.status(0.92, "Generating image".to_string());

    let height = signal.len() as u32 / PX_PER_ROW;

    type LumaImage = image::ImageBuffer::<image::Luma<u8>, Vec<u8>>;
    use image::ConvertBuffer;

    let mut img: Image = LumaImage::from_vec(
            PX_PER_ROW, height, map(&signal, low, high))
        .map(|b| b.convert()) // Convert to RGB
        .ok_or(err::Error::Internal(
            "Could not create image, wrong buffer length".to_string()))?;

    // context.step(Step::signal(
            // "mapped",
            // &raw_data.iter().map(|x| f32::from(*x)).collect(),
            // Some(final_rate)
    // ))?;

    match rotate {
        Rotate::Yes => {
            // context.status(0.93, "Rotating output image".to_string());
            img = processing::rotate(&img);
        },
        Rotate::Orbit => {
            if let None = orbit {
                return Err(err::Error::Internal(
                    "Can't rotate from orbit if none provided".to_string()));
            }
            // TODO
        },
        Rotate::No => {},
    }

    Ok(img)
}

/*
    let timestamp = misc::read_timestamp(&settings.input_filename)?;
    map::draw_map(&mut img, timestamp, height);

    img.save(settings.output_filename)?;

    context.status(0.95, format!("Writing PNG to '{}'", settings.output_filename.display()));


    // --------------------

    context.status(1., "Finished".to_string());
    debug!("Finished");

    context.status(0.0, "Reading WAV file".to_string());

    // --------------------

*/

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
