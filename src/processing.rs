//! Image processing functions.

use image::{GenericImage, Rgba, RgbaImage};
use log::{warn, info};

use crate::decode::{PX_PER_CHANNEL, PX_SYNC_FRAME, PX_SPACE_DATA, PX_CHANNEL_IMAGE_DATA};
use crate::err;
use crate::imageext;
use crate::geo;
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

    let mut channel_a = img.sub_image(
        x_offset, 0, PX_CHANNEL_IMAGE_DATA, img.height()
    );
    image::imageops::rotate180_in_place(&mut channel_a);

    let mut channel_b = img.sub_image(
        x_offset + PX_PER_CHANNEL, 0, PX_CHANNEL_IMAGE_DATA, img.height()
    );
    image::imageops::rotate180_in_place(&mut channel_b);
}

/// Returns true if this was a south to north pass, and the image needs to be rotated.
pub fn south_to_north_pass(orbit_settings: &OrbitSettings) -> err::Result<bool> {

    let tle = match &orbit_settings.custom_tle {
        Some(t) => t.clone(),
        None => misc::get_current_tle()?,
    };

    let (sats, _errors) = satellite::io::parse_multiple(&tle);
    let sat_string = orbit_settings.sat_name.to_string();

    let sat = sats.iter()
        .find(|&sat| sat.name.as_ref() == Some(&sat_string))
        .ok_or_else(||
            err::Error::Internal(format!("Satellite \"{}\" not found in TLE", sat_string))
        )?.clone();

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
    info!("Performing histogram equalization, has color: {}", has_color);
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
/// Needs a way to allow tweaking hardcoded values for water, land, ice
/// and dirt detection, from the UI or command line.
pub fn false_color(img: &mut RgbaImage, color_settings: &ColorSettings) {
    let water = color_settings.water_threshold;
    let vegetation = color_settings.vegetation_threshold;
    let clouds = color_settings.clouds_threshold;

    info!("Colorize image (false color), water={}, vegetation={}, clouds={}",
        water, vegetation, clouds
    );

    if water > vegetation || vegetation > clouds {
        warn!("Condition not satisfied: 'water < vegetation < clouds'. Expect wrong results");
    }

    let x_start = PX_SYNC_FRAME + PX_SPACE_DATA;
    let x_end = x_start + PX_CHANNEL_IMAGE_DATA;
    let image_height = img.height();

    // colorize
    for x in x_start..x_end {
        for y in 0..image_height {
            let val_pixel = img.get_pixel(x, y);
            let irval_pixel = img.get_pixel(x + PX_PER_CHANNEL, y);

            let val = val_pixel[0] as f32;
            let irval = irval_pixel[0] as f32;

            let r: f32;
            let g: f32;
            let b: f32;

            if val < water as f32 {
                // Water identification
                r = (8.0 + val * 0.2).min(255.);
                g = (20.0 + val * 1.0).min(255.);  // avoid overflow
                b = (50.0 + val * 0.75).min(255.);
            }
            else if irval > clouds as f32 {
                // Cloud/snow/ice identification
                // IR channel helps distinguish clouds and water, particularly in arctic areas
                r = (irval + val) * 0.5; // Average the two for a little better cloud distinction
                g = r;
                b = r;
            }
            else if val < vegetation as f32 {
                // Vegetation identification
                // green
                r = val * 0.8;
                g = val * 0.9;
                b = val * 0.6;
            }
            else if val <= clouds as f32 {
                // Desert/dirt identification
                // brown
                r = val * 1.0;
                g = val * 0.9;
                b = val * 0.7;
            }
            else {
                // Everything else, but this was probably captured by the IR channel above
                // Clouds, snow, and really dry desert
                r = val;
                g = val;
                b = val;
            }

            img.put_pixel(x, y, Rgba([r as u8, g as u8, b as u8, 255]));
        }
    }
}
