//! Image processing functions.

use image::{GenericImageView, GenericImage, GrayImage, ImageBuffer, Rgba};
use log::info;

use crate::decode::{PX_PER_CHANNEL, PX_SYNC_FRAME, PX_SPACE_DATA, PX_CHANNEL_IMAGE_DATA};
use crate::err;
use crate::geo;
use crate::misc;
use crate::noaa_apt::{Image, OrbitSettings, RefTime};


/// Rotates the channels in place, keeping the sync bands and telemetry intact.
///
/// Takes as an argument a raw image, that is, with syncing frames and telemetry
/// bands. These will not be removed.
///
/// Care is taken to leave lines from the A channel at the same height as the B
/// channel. Otherwise there can be a vertical offset of one pixel between each
/// channel.
pub fn rotate(img: &mut Image) {
    info!("Rotating image");

    // where the actual image data starts, past the sync frames and deep space band
    let x_offset = PX_SYNC_FRAME + PX_SPACE_DATA - 1; // !

    // Note: the (-1) offsets are here (lines marked with // !) because it looks like
    // the image shifts ~2 px to the left during the sync phase.
    // So 2px that should be at the left edge turn out to be at the right edge.
    // This causes some artifacts when the channels are rotated, since the offsets
    // "bite into" the telemetry bands, that turn out on the left after rotation.
    // This seems to fix it, but try to find if it's possible to do it during the 
    // sync phase, and get rid of these here.
    // Details here: https://github.com/martinber/noaa-apt/issues/26

    let mut channel_a = img.sub_image(
        x_offset, 0, PX_CHANNEL_IMAGE_DATA - 1, img.height() // !
    );
    image::imageops::rotate180_in_place(&mut channel_a);
    
    let mut channel_b = img.sub_image(
        x_offset + PX_PER_CHANNEL, 0, PX_CHANNEL_IMAGE_DATA - 1, img.height() // !
    );
    image::imageops::rotate180_in_place(&mut channel_b);
}

/// Rotate image if the pass was south to north.
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

/// Histogram equalization, for each channel separately.
/// Works only on the grayscale image,
/// needs to be done before the RGBA conversion.
pub fn histogram_equalization(img: &GrayImage) -> err::Result<GrayImage> {
    info!("Performing histogram equalization");

    let mut output = GrayImage::new(img.width(), img.height());
    let mut channel_a = img.view(0, 0, PX_PER_CHANNEL, img.height()).to_image();
    let mut channel_b = img
        .view(PX_PER_CHANNEL, 0, PX_PER_CHANNEL, img.height())
        .to_image();

    imageproc::contrast::equalize_histogram_mut(&mut channel_a);
    imageproc::contrast::equalize_histogram_mut(&mut channel_b);

    output.copy_from(&channel_a, 0, 0)?;
    output.copy_from(&channel_b, PX_PER_CHANNEL, 0)?;

    Ok(output)
}

/// Attempts to produce a colored image from grayscale channel and IR data.
/// Works best when contrast is set to "telemetry".
/// Needs a way to allow tweaking hardcoded values for water, land, ice
/// and dirt detection, from the UI or command line.
pub fn false_color(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, values: (u8, u8, u8)) {
    let (water, vegetation, clouds) = values;
    info!("Colorize image (false color), water={}, vegetation={}, clouds={}",
        water, vegetation, clouds
    );

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
                g = (20.0 + val * 1.0).min(255.);
                b = (50.0 + val * 0.75).min(255.); // avoid overflow
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
