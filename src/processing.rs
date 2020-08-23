//! Image processing functions.

use image::{GenericImage, Rgba, RgbaImage, Pixel, GenericImageView};
use lab::Lab;
use log::{warn, info};

use crate::decode::{PX_PER_CHANNEL, PX_SYNC_FRAME, PX_SPACE_DATA, PX_CHANNEL_IMAGE_DATA};
use crate::err;
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
    let x_offset = PX_SYNC_FRAME + PX_SPACE_DATA; // !

    let mut channel_a = img.sub_image(
        x_offset, 0, PX_CHANNEL_IMAGE_DATA, img.height() // !
    );
    image::imageops::rotate180_in_place(&mut channel_a);

    let mut channel_b = img.sub_image(
        x_offset + PX_PER_CHANNEL, 0, PX_CHANNEL_IMAGE_DATA, img.height() // !
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
pub fn histogram_equalization(img: &mut RgbaImage, has_color: bool) -> err::Result<RgbaImage> {
    info!("Performing histogram equalization, has color: {}", has_color);

    let mut output = RgbaImage::new(img.width(), img.height());
    let mut channel_a = img.view(0, 0, PX_PER_CHANNEL, img.height()).to_image();
    let mut channel_b = img
        .view(PX_PER_CHANNEL, 0, PX_PER_CHANNEL, img.height()).to_image();

    if has_color {
        equalize_histogram_color(&mut channel_a);
    } else {
        equalize_histogram_grayscale(&mut channel_a);
    }
    equalize_histogram_grayscale(&mut channel_b);

    output.copy_from(&channel_a, 0, 0)?;
    output.copy_from(&channel_b, PX_PER_CHANNEL, 0)?;

    Ok(output)
}

fn equalize_histogram_grayscale(image: &mut RgbaImage) {
    // since image is grayscale (R = G = B, A = 255), use R channel for histogram:
    let hist = imageproc::stats::cumulative_histogram(image).channels[0];
    let total = hist[255] as f32;

    image.pixels_mut().for_each(|p| {
        // Each channel of CumulativeChannelHistogram has length 256, and Image has 8 bits per pixel
        let fraction = unsafe { *hist.get_unchecked(p.channels()[0] as usize) as f32 / total };
        // apply f to each channel and g to alpha
        p.apply_with_alpha(
            // for R, G, B, use equalized values:
            |_| (f32::min(255f32, 255f32 * fraction)) as u8,
            // for A, leave unmodified
            |alpha| alpha
        );
    });
}


fn equalize_histogram_color(image: &mut RgbaImage) {
    let mut lab_pixels: Vec<Lab> = rgb_to_lab(&image);
    
    let lab_hist = cumulative_lab_histogram(&lab_pixels);
    let total = lab_hist[100] as f32;

    lab_pixels.iter_mut().for_each(|p: &mut Lab| {
        let fraction = unsafe { *lab_hist.get_unchecked(p.l as usize) as f32 / total };
        p.l = f32::min(100f32, 100f32 * fraction);
    });
    lab_to_rgb_mut(&lab_pixels, image);
}

fn rgb_to_lab(image: &RgbaImage) -> Vec<Lab> {
    image.pixels().map(|p| {
        let (r, g, b, _) = p.channels4();
        Lab::from_rgb(&[r, g, b])
    }).collect()
}

fn lab_to_rgb_mut(lab_pixels: &Vec<Lab>, image: &mut RgbaImage) {
    let rgb_pixels: Vec<[u8; 3]> = lab_pixels.iter().map(|x: &Lab| x.to_rgb()).collect();

    image.pixels_mut().enumerate().for_each(|(i, p)| {
        let [r, g, b] = rgb_pixels[i];
        let (_, _, _, a) = p.channels4(); // alpha channel
        *p = Pixel::from_channels(r, g, b, a);
    })
}

fn cumulative_lab_histogram(lab_pixels: &Vec<Lab>) -> [u32; 101] {
    let mut hist = lab_histogram(lab_pixels);
    for i in 1..hist.len() {
        hist[i] += hist[i - 1];
    }
    hist
}

fn lab_histogram(lab_pixels: &Vec<Lab>) -> [u32; 101] {
    let mut hist = [0u32; 101];
    for p in lab_pixels {
        hist[p.l as usize] += 1;
    }
    hist
}


/// Attempts to produce a colored image from grayscale channel and IR data.
/// Works best when contrast is set to "telemetry".
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
