//! Image processing functions.

use log::info;
use image::{GenericImageView, GenericImage};

use crate::noaa_apt::{Image, OrbitSettings, RefTime, SatName};
use crate::decode::PX_PER_CHANNEL;
use crate::err;
use crate::geo;
use crate::misc;

/// Rotate image without changing the location of the channels.
///
/// Takes as an argument a raw image, that is, with syncing frames and telemetry
/// bands. These will not be removed.
///
/// Care is taken to leave lines from the A channel at the same height as the B
/// channel. Otherwise there can be a vertical offset of one pixel between each
/// channel.
pub fn rotate(img: &Image) -> err::Result<Image> {

    info!("Rotating image");

    // Create image with channel A and B swapped
    let mut output = Image::new(img.width(), img.height());
    let channel_a = img.view(0, 0, PX_PER_CHANNEL, img.height());
    let channel_b = img.view(PX_PER_CHANNEL, 0, PX_PER_CHANNEL, img.height());
    output.copy_from(&channel_b, 0, 0)?;
    output.copy_from(&channel_a, PX_PER_CHANNEL, 0)?;

    image::imageops::rotate180_in_place(&mut output);

    Ok(output)
}

/// Rotate image if the pass was south to north.
pub fn south_to_north_pass(orbit_settings: &OrbitSettings) -> err::Result<bool> {

    let tle = match &orbit_settings.custom_tle {
        Some(t) => t.clone(),
        None => misc::get_current_tle()?,
    };

    let (sats, _errors) = satellite::io::parse_multiple(&tle);
    let sat_string = match orbit_settings.sat_name {
        SatName::Noaa15 => "NOAA 15",
        SatName::Noaa18 => "NOAA 18",
        SatName::Noaa19 => "NOAA 19",
    }.to_string();

    let mut sat = sats.iter()
        .find(|&sat| sat.name.as_ref() == Some(&sat_string))
        .ok_or_else(||
            err::Error::Internal(format!("Satellite \"{}\" not found in TLE", sat_string))
        )?.clone();

    let start_time = match orbit_settings.ref_time {
        RefTime::Start(time) => time,
        RefTime::End(time) => time,
    };

    // TODO: Remove unwrap()
    let result = satellite::propogation::propogate_datetime(&mut sat, start_time).unwrap();
    let gmst = satellite::propogation::gstime::gstime_datetime(start_time);
    let sat_start_pos = satellite::transforms::eci_to_geodedic(&result.position, gmst);

    let end_time = start_time + chrono::Duration::seconds(2);

    // TODO: Remove unwrap()
    let result = satellite::propogation::propogate_datetime(&mut sat, end_time).unwrap();
    let gmst = satellite::propogation::gstime::gstime_datetime(end_time);
    let sat_end_pos = satellite::transforms::eci_to_geodedic(&result.position, gmst);

    let azimuth = geo::azimuth(
        (sat_start_pos.latitude, sat_start_pos.longitude),
        (sat_end_pos.latitude, sat_end_pos.longitude),
    );

    use std::f64::consts::PI;
    return Ok(azimuth < PI / 4. || azimuth > 3. * PI / 4.);
}
