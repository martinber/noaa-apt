/// Functions to read shapefiles and draw maps.
///

use std::f64::consts::PI;

use line_drawing::XiaolinWu;
use log::info;

use crate::draw;
use crate::err;
use crate::geo;
use crate::noaa_apt::{SatName, RefTime, Image, MapSettings};

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

    // (latitude, longitude) of the satellite for each line
    let mut sat_positions: Vec<(f64, f64)> = Vec::with_capacity(height as usize);

    let start_time = match ref_time {
        RefTime::Start(time) => time,
        RefTime::End(time) => time - line_duration * height as i32,
    };

    for i in 0..height {
        let t = start_time + line_duration * i as i32;
        let result = satellite::propogation::propogate_datetime(&mut sat, t).unwrap();
        let gmst = satellite::propogation::gstime::gstime_datetime(t);
        let sat_pos = satellite::transforms::eci_to_geodedic(&result.position, gmst);
        sat_positions.push((sat_pos.latitude, sat_pos.longitude));
    }
    let start_latlon = sat_positions[0];
    let end_latlon = *sat_positions.last().unwrap();

    // Get image resolution (radians per pixel)

    let y_res = geo::distance(start_latlon, end_latlon) / height as f64 / settings.vscale;
    let x_res = 0.0005001960653876187 / settings.hscale;

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

    // px_rel_to_abs ///////////////////////////////////////////////////////////

    let px_rel_to_abs = |(x, y): (f64, f64)| -> (f64, f64) {

        let x_abs = x + 539.;
        let y_abs = y;
        // let x_abs = (x + 539.).max(0.).min(2070.) as u32;
        // let y_abs = y.max(0.).min(height as f64) as u32;

        (x_abs, y_abs)
    };

    // Dibujar mapa ////////////////////////////////////////////////////////////

    let filename = "./res/shapefiles/states.shp";
    let reader = shapefile::Reader::from_path(filename).unwrap();

    for result in reader.iter_shapes_as::<shapefile::Polyline>() {
        let polyline = result.unwrap(); //TODO
        for points in polyline.parts() {

            let mut prev_pt = &points[0];

            for pt in points {
                // y: lat, x: lon,
                let (x, y) = latlon_to_rel_px((pt.y / 180. * PI, pt.x / 180. * PI));
                let (x2, y2) = latlon_to_rel_px((prev_pt.y / 180. * PI, prev_pt.x / 180. * PI));
                let est_y = (y.max(0.) as usize).min(height as usize - 1);
                let est_y2 = (y2.max(0.) as usize).min(height as usize - 1);
                let (x_offset, _) = latlon_to_rel_px(sat_positions[est_y]);
                let (x2_offset, _) = latlon_to_rel_px(sat_positions[est_y2]);

                let p1 = px_rel_to_abs((x - x_offset, y));
                let p2 = px_rel_to_abs((x2 - x2_offset, y2));

                let w = img.width() as i32;
                let h = img.height() as i32;
                for ((x, y), value) in XiaolinWu::<f64, i32>::new(p1, p2) {
                    if x > 0 && y > 0 && x < w && y < h {
                        img.put_pixel(
                            x as u32, y as u32,
                            image::Rgb([100, 150, 0]),
                        );
                    }
                }
                prev_pt = pt;
            }
        }
    }
    let filename = "./res/shapefiles/countries.shp";
    let reader = shapefile::Reader::from_path(filename).unwrap();

    for result in reader.iter_shapes_as::<shapefile::Polygon>() {
        let polygon = result.unwrap(); //TODO
        for ring in polygon.rings() {

            use shapefile::record::polygon::PolygonRing;

            let points = match ring {
                PolygonRing::Outer(p) | PolygonRing::Inner(p) => p,
            };

            let mut prev_pt = &points[0];

            for pt in points {
                // y: lat, x: lon,
                let (x, y) = latlon_to_rel_px((pt.y / 180. * PI, pt.x / 180. * PI));
                let (x2, y2) = latlon_to_rel_px((prev_pt.y / 180. * PI, prev_pt.x / 180. * PI));
                let est_y = (y.max(0.) as usize).min(height as usize - 1);
                let est_y2 = (y2.max(0.) as usize).min(height as usize - 1);
                let (x_offset, _) = latlon_to_rel_px(sat_positions[est_y]);
                let (x2_offset, _) = latlon_to_rel_px(sat_positions[est_y2]);

                let p1 = px_rel_to_abs((x - x_offset, y));
                let p2 = px_rel_to_abs((x2 - x2_offset, y2));

                let w = img.width() as i32;
                let h = img.height() as i32;
                for ((x, y), value) in XiaolinWu::<f64, i32>::new(p1, p2) {
                    if x > 0 && y > 0 && x < w && y < h {
                        img.put_pixel(
                            x as u32, y as u32,
                            image::Rgb([0, (255.*value) as u8, 0]),
                        );
                    }
                }
                prev_pt = pt;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    // #[test]
    // fn test() {
//
        // let filename = "./res/shapefiles/countries.shp";
        // let reader = shapefile::Reader::from_path(filename).unwrap();
//
        // for result in reader.iter_shapes_as::<shapefile::Polygon>() {
            // let polygon = result.unwrap(); //TODO
        // }
        // assert!(false);
//
    // }
}
