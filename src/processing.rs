//! Image processing functions.

use image::{GenericImage, RgbaImage, Pixel};
use log::info;

use crate::decode::{PX_CHANNEL_IMAGE_DATA, PX_PER_CHANNEL, PX_SPACE_DATA, PX_SYNC_FRAME};
use crate::err;
use crate::geo;
use crate::imageext;
use crate::misc;
use crate::noaa_apt::{ColorSettings, OrbitSettings, RefTime};

/// Rotates the channels in place, keeping the sync bands and telemetry intact.
///
/// Takes as an argument a raw image, that is, with syncing frames and telemetry
/// bands. These will not be removed.
///
/// Care is taken to leave lines from the A channel at the same height as the B
/// channel. Otherwise there can be a vertical offset of one pixel between each
/// channel.
pub fn rotate(img: &mut RgbaImage) {
    info!("Rotating image");

    // where the actual image data starts, past the sync frames and deep space band
    let x_offset = PX_SYNC_FRAME + PX_SPACE_DATA;

    let mut channel_a = img.sub_image(x_offset, 0, PX_CHANNEL_IMAGE_DATA, img.height());
    image::imageops::rotate180_in_place(&mut *channel_a);

    let mut channel_b = img.sub_image(
        x_offset + PX_PER_CHANNEL,
        0,
        PX_CHANNEL_IMAGE_DATA,
        img.height(),
    );
    image::imageops::rotate180_in_place(&mut *channel_b);
}

/// Returns true if this was a south to north pass, and the image needs to be rotated.
pub fn south_to_north_pass(orbit_settings: &OrbitSettings) -> err::Result<bool> {
    let tle = match &orbit_settings.custom_tle {
        Some(t) => t.clone(),
        None => misc::get_current_tle()?,
    };

    let (sats, _errors) = satellite::io::parse_multiple(&tle);
    let sat_string = orbit_settings.sat_name.to_string();

    let sat = sats
        .iter()
        .find(|&sat| sat.name.as_ref() == Some(&sat_string))
        .ok_or_else(|| {
            err::Error::Internal(format!("Satellite \"{}\" not found in TLE", sat_string))
        })?
        .clone();

    let start_time = match orbit_settings.ref_time {
        RefTime::Start(time) => time,
        RefTime::End(time) => time,
    };

    // TODO: Remove unwrap()
    let result = satellite::propogation::propogate_datetime(&sat, start_time).unwrap();
    let gmst = satellite::propogation::gstime::gstime_datetime(start_time);
    let sat_start_pos = satellite::transforms::eci_to_geodedic(&result.position, gmst);

    let end_time = start_time + chrono::Duration::seconds(2);

    // TODO: Remove unwrap()
    let result = satellite::propogation::propogate_datetime(&sat, end_time).unwrap();
    let gmst = satellite::propogation::gstime::gstime_datetime(end_time);
    let sat_end_pos = satellite::transforms::eci_to_geodedic(&result.position, gmst);

    let azimuth = geo::azimuth(
        (sat_start_pos.latitude, sat_start_pos.longitude),
        (sat_end_pos.latitude, sat_end_pos.longitude),
    );

    use std::f64::consts::PI;
    return Ok(azimuth < PI / 4. || azimuth > 3. * PI / 4.);
}

/// Histogram equalization, in place, for each channel (A, B) separately.
/// If `has_color=false`, it will treat the image as grayscale (R = G = B, A = 255).
/// If `has_color=true`, it will convert image from Rgba to Lab, equalize the histogram
/// for L (lightness) channel, convert back to Rgb and adjust image values accordingly.
pub fn histogram_equalization(img: &mut RgbaImage, has_color: bool) {
    info!(
        "Performing histogram equalization, has color: {}",
        has_color
    );
    let height = img.height();

    let mut channel_a = img.sub_image(0, 0, PX_PER_CHANNEL, height);
    if has_color {
        imageext::equalize_histogram_color(&mut channel_a);
    } else {
        imageext::equalize_histogram_grayscale(&mut channel_a);
    }

    let mut channel_b = img.sub_image(PX_PER_CHANNEL, 0, PX_PER_CHANNEL, height);
    imageext::equalize_histogram_grayscale(&mut channel_b);
}

/// Attempts to produce a colored image from grayscale channel and IR data.
/// Works best when contrast is set to "telemetry" or "98 percent".
/// Uses a palette image to map channel brightness values to a color.
pub fn false_color(img: &mut RgbaImage, color_settings: &ColorSettings) -> err::Result<()>{

    let palette_img = image::open(&color_settings.palette_filename).map_err(
            |_| err::Error::InvalidInput(format!("Could not load {:?}", &color_settings.palette_filename))
        )?.into_rgb8();

    if palette_img.width() != 256 || palette_img.height() != 256 {
        return Err(err::Error::InvalidInput("Invalid palette image dimensions".to_string()))
    }

    // Determine region of channel A, which will be the only one colorized
    let x_start = PX_SYNC_FRAME + PX_SPACE_DATA;
    let x_end = x_start + PX_CHANNEL_IMAGE_DATA;
    let image_height = img.height();

    // Assumes input and output values from 0 to 255
    // Gives output in u32 because that is the input to the next function
    let tune_input_values = |in_a: u8, in_b: u8| -> (u32, u32) {
        let factor: f32 = 0.3; // Determines how much the values will be modified

        let in_a: f32 = in_a.into();
        let in_b: f32 = in_b.into();

        let s_a: f32 = color_settings.ch_a_tune_start * factor;
        let e_a: f32 = color_settings.ch_a_tune_end * factor;
        let s_b: f32 = color_settings.ch_b_tune_start * factor;
        let e_b: f32 = color_settings.ch_b_tune_end * factor;

        let out_a = in_a * (1. + e_a - s_a) - s_a * 255.;
        let out_b = in_b * (1. + e_b - s_b) - s_b * 255.;

        return (out_a.clamp(0., 255.) as u32, out_b.clamp(0., 255.) as u32);
    };

    // Colorize
    for x in x_start..x_end {
        for y in 0..image_height {
            let ch_a = img.get_pixel(x, y)[0]; // Red channel of channel A
            let ch_b = img.get_pixel(x + PX_PER_CHANNEL, y)[0]; // Red channel of channel B

            let (val_a, val_b) = tune_input_values(ch_a, ch_b);

            img.put_pixel(x, y,
                palette_img.get_pixel(val_a, val_b).to_rgba()
            );
        }
    }

    Ok(())
}
