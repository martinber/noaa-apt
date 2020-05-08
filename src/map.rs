/// Code to read shapefiles and draw the map overlay.

use std::f64::consts::PI;

use image::Pixel;
use line_drawing::XiaolinWu;
use log::info;

use crate::err;
use crate::geo;
use crate::noaa_apt::{SatName, RefTime, Image, MapSettings};


/// Draws the map overlay mutating the image.
#[allow(non_snake_case)]
#[allow(clippy::many_single_char_names)]
pub fn draw_map(
    img: &mut Image,
    ref_time: RefTime,
    settings: MapSettings,
    sat_name: SatName,
    tle: String,
) -> err::Result<()> {

    info!("Drawing map overlay");

    let height = img.height();
    let line_duration = chrono::Duration::milliseconds(500); // Two lines per sec

    // Load satellite from TLE

    let (sats, _errors) = satellite::io::parse_multiple(&tle);
    let sat_string = match sat_name {
        SatName::Noaa15 => "NOAA 15",
        SatName::Noaa18 => "NOAA 18",
        SatName::Noaa19 => "NOAA 19",
    }.to_string();

    let mut sat = sats.iter()
        .find(|&sat| sat.name.as_ref() == Some(&sat_string))
        .ok_or_else(||
            err::Error::Internal(format!("Satellite \"{}\" not found in TLE", sat_string))
        )?.clone();

    // Calculate satellite trajectory

    let start_time = match ref_time {
        RefTime::Start(time) => time,
        RefTime::End(time) => time - line_duration * height as i32,
    };

    // (latitude, longitude) of the satellite for each line
    let mut sat_positions: Vec<(f64, f64)> = Vec::with_capacity(height as usize);

    for i in 0..height {
        let t = start_time + line_duration * i as i32;
        // TODO: Remove unwrap()
        let result = satellite::propogation::propogate_datetime(&mut sat, t).unwrap();
        let gmst = satellite::propogation::gstime::gstime_datetime(t);
        let sat_pos = satellite::transforms::eci_to_geodedic(&result.position, gmst);
        sat_positions.push((sat_pos.latitude, sat_pos.longitude));
    }
    let start_latlon = sat_positions[0];
    let end_latlon = *sat_positions.last().unwrap();

    // Get image resolution (radians per pixel)

    let y_res = geo::distance(start_latlon, end_latlon) / height as f64 / settings.vscale;
    let x_res = 0.0005 / settings.hscale;

    // Map (latitude, longitude) to pixel coordinates

    let ref_az = geo::azimuth(start_latlon, end_latlon);

    let latlon_to_rel_px = |latlon: (f64, f64)| -> (f64, f64) {
        // To understand this, you should look at the illustrations on my how
        // it works page.

        let az = geo::azimuth(start_latlon, latlon);
        let B = az - ref_az;

        // Set maximum, otherwise we get wrapping problems I do not fully
        // understand: opposite parts of the world are mapped to the same
        // position because of the cyclic nature of sin(), cos(), etc.
        let c = geo::distance(latlon, start_latlon).max(-PI/3.).min(PI/3.);

        let a = (B.cos() * c.tan()).atan();
        let b = (B.sin() * c.sin()).asin();

        let x = -b / x_res;

        // Add the yaw correction value. I should be calculating sin(yaw) * x
        // but yaw is always a small value.
        let y = a / y_res + settings.yaw * x;

        (x, y)
    };

    // Draw line function

    let mut draw_line = |
        latlon1: (f64, f64),
        latlon2: (f64, f64),
        (r, g, b, a): (u8, u8, u8, u8)
    | {

        // Convert latlon to (x, y)
        let (mut x1, y1) = latlon_to_rel_px(latlon1);
        let (mut x2, y2) = latlon_to_rel_px(latlon2);

        // Offset correction on x
        let est_y1 = (y1.max(0.) as usize).min(height as usize - 1);
        let est_y2 = (y2.max(0.) as usize).min(height as usize - 1);
        let (x1_offset, _) = latlon_to_rel_px(sat_positions[est_y1]);
        let (x2_offset, _) = latlon_to_rel_px(sat_positions[est_y2]);
        x1 -= x1_offset;
        x2 -= x2_offset;

        let h = img.height() as i32;

        // See if at least one point is inside
        if (x1 > -456. && x1 < 456. && y1 > 0. && y1 < h as f64)
            || (x1 > -600. && x1 < 600. && y1 > 0. && y1 < h as f64)
        {

            for ((x, y), value) in XiaolinWu::<f64, i32>::new((x1, y1), (x2, y2)) {
                // Draw A channel
                if x > -456 && x < 456 && y > 0 && y < h {
                    img.get_pixel_mut((x + 539) as u32, y as u32).blend(
                        //value is between 0 and 1. a is between 0 and 255
                        &image::Rgba([r, g, b, (value * a as f64) as u8]),
                    );
                    img.get_pixel_mut((x + 1579) as u32, y as u32).blend(
                        &image::Rgba([r, g, b, (value * a as f64) as u8]),
                    );
                }
            }

        }
    };

    // Draw shapefiles

    let filename = res_path!("shapefiles", "states.shp");
    let reader = shapefile::Reader::from_path(&filename).map_err(|_| {
        err::Error::Internal(format!("Could not load {:?}", filename))
    })?;
    for result in reader.iter_shapes_as::<shapefile::Polyline>() {
        let polyline = result?;
        for points in polyline.parts() {

            let mut prev_pt = &points[0];
            for pt in points {
                draw_line(
                    (pt.y / 180. * PI, pt.x / 180. * PI),
                    (prev_pt.y / 180. * PI, prev_pt.x / 180. * PI),
                    settings.states_color,
                );
                prev_pt = pt;
            }
        }
    }

    let filename = res_path!("shapefiles", "countries.shp");
    let reader = shapefile::Reader::from_path(&filename).map_err(|_| {
        err::Error::Internal(format!("Could not load {:?}", filename))
    })?;
    for result in reader.iter_shapes_as::<shapefile::Polygon>() {
        let polygon = result?;
        for ring in polygon.rings() {

            use shapefile::record::polygon::PolygonRing;
            let points = match ring {
                PolygonRing::Outer(p) | PolygonRing::Inner(p) => p,
            };

            let mut prev_pt = &points[0];
            for pt in points {
                draw_line(
                    (pt.y / 180. * PI, pt.x / 180. * PI),
                    (prev_pt.y / 180. * PI, prev_pt.x / 180. * PI),
                    settings.countries_color,
                );
                prev_pt = pt;
            }
        }
    }

    let filename = res_path!("shapefiles", "lakes.shp");
    let reader = shapefile::Reader::from_path(&filename).map_err(|_| {
        err::Error::Internal(format!("Could not load {:?}", filename))
    })?;
    for result in reader.iter_shapes_as::<shapefile::Polygon>() {
        let polygon = result?;
        for ring in polygon.rings() {

            use shapefile::record::polygon::PolygonRing;
            let points = match ring {
                PolygonRing::Outer(p) | PolygonRing::Inner(p) => p,
            };

            let mut prev_pt = &points[0];
            for pt in points {
                draw_line(
                    (pt.y / 180. * PI, pt.x / 180. * PI),
                    (prev_pt.y / 180. * PI, prev_pt.x / 180. * PI),
                    settings.lakes_color,
                );
                prev_pt = pt;
            }
        }
    }

    Ok(())
}
