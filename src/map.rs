/// Functions to read shapefiles and draw maps.
///

use std::f64::consts::PI;

use chrono::prelude::*;
use shapefile::Shape;

use crate::draw;
use crate::geo;
use crate::noaa_apt::{SatName, Image, Pixel, MapSettings};

#[derive(Debug)]
struct SatState {
    latlon: (f64, f64),
    side_az: f64,
}

pub fn draw_map(
    img: &mut Image,
    start_time: chrono::DateTime<chrono::Utc>,
    settings: MapSettings,
) {
    let height = img.height();

    let (sats, _errors) = satellite::io::parse_multiple(include_str!("../weather-2018-12.txt"));
    let mut sat = sats.iter().find(|&sat| sat.name == Some("NOAA 19".to_string()))
        .expect("not found in test TLE file").clone();

    // Generar vector de estados ///////////////////////////////////////////////

    let mut time: Vec<chrono::DateTime<_>> = Vec::with_capacity(height as usize);

    time.push(start_time); // 0 milliseconds
    let time_step = chrono::Duration::milliseconds(500); // Seconds per line

    for i in 0..height {
        time.push(*time.last().unwrap() + time_step);
    }

    let mut sat_state: Vec<SatState> = Vec::with_capacity(height as usize);
    for t in time {
        let result = satellite::propogation::propogate_datetime(&mut sat, t).unwrap();
        let gmst = satellite::propogation::gstime::gstime_datetime(t);
        let sat_pos = satellite::transforms::eci_to_geodedic(&result.position, gmst);

        sat_state.push(SatState {
            latlon: (sat_pos.latitude, sat_pos.longitude),
            side_az: 0.,
        });
    }

    // TODO emprolijar
    for i in 1..sat_state.len()-1 {
        let s1 = &sat_state[i-1];
        let s2 = &sat_state[i+1];
        sat_state[i].side_az = geo::azimuth(s1.latlon, s2.latlon) + PI/2. + settings.yaw;
    }

    // Calcular resolucion /////////////////////////////////////////////////////

    let s1 = &sat_state[0];
    let s2 = sat_state.last().unwrap();
    let y_res = geo::distance(s1.latlon, s2.latlon) / height as f64 * settings.vscale;
    let x_res = 0.0005001960653876187 * settings.hscale;

    // latlon_to_rel_px ////////////////////////////////////////////////////////

    let start_latlon = sat_state[0].latlon;
    let end_latlon = sat_state.last().unwrap().latlon;
    let ref_az = geo::azimuth(start_latlon, end_latlon);
    let latlon_to_rel_px = |latlon: (f64, f64)| -> (f64, f64) {

        let dist = geo::distance(latlon, start_latlon);
        let az = geo::azimuth(start_latlon, latlon) + PI;

        let tmp = geo::azimuth(start_latlon, latlon);

        let B = tmp - ref_az;
        let c = dist;

        let a = (B.cos() * c.tan()).max(-PI/2.).min(PI/2.).atan();
        let b = (B.sin() * c.sin()).max(-PI/2.).min(PI/2.).asin();

        let x = -b / x_res;
        let y = a / y_res + settings.yaw * x;

        (x, y)
    };

    // px_rel_to_abs ///////////////////////////////////////////////////////////

    let px_rel_to_abs = |(x, y): (f64, f64)| -> (u32, u32) {

        let x_abs = (x + 539.).max(0.).min(2070.) as u32;
        let y_abs = y.max(0.).min(height as f64) as u32;

        (x_abs, y_abs)
    };

    // Generar imagen equirectangular //////////////////////////////////////////

    /*
    let mut geoimg: Vec<[(f64, f64); 2080]> = vec![[(0., 0.); 2080]; height as usize];

    let mut max_lat = sat_state[0].latlon.0;
    let mut min_lat = sat_state[0].latlon.0;
    let mut max_lon = sat_state[0].latlon.1;
    let mut min_lon = sat_state[0].latlon.1;
    for y in 0..(height as usize) {
        for x in 0..2080 {
            geoimg[y][x] = geo::reckon(
                sat_state[y].latlon,
                -((x as f64) - 539.) * x_res,
                sat_state[y].side_az
            );
            max_lat = max_lat.max(geoimg[y][x].0);
            min_lat = min_lat.min(geoimg[y][x].0);
            max_lon = max_lon.max(geoimg[y][x].1);
            min_lon = min_lon.min(geoimg[y][x].1);
        }
    }

    let lat_to_y = |lat: f64| -> u32 {
        (-(lat - max_lat)*40.).max(0.) as u32
    };
    let lon_to_x = |lon: f64| -> u32 {
        ((lon - min_lon)*40.).max(0.) as u32
    };

    let mut img2 = image::ImageBuffer::<image::Luma<u8>, Vec<u8>>::new(
        lon_to_x(max_lon), lat_to_y(min_lat)
    );

    for y in 0..(height as usize) {
        for x in 0..2080 {
            img2.put_pixel(lon_to_x(geoimg[y][x].1).min(img2.width()-1), lat_to_y(geoimg[y][x].0).min(img2.height()-1),
                *img.get_pixel(x as u32, y as u32)
            );
        }
    }
    */

    // Dibujar linea central ///////////////////////////////////////////////////

    let mut prev_s = &sat_state[0];
    for s in sat_state.iter() {
        // y: lat, x: lon,
        let (x, y) = px_rel_to_abs(latlon_to_rel_px(s.latlon));
        let (x2, y2) = px_rel_to_abs(latlon_to_rel_px(prev_s.latlon));
        draw::draw_line(img, image::Rgb([255, 0, 0]), x, y, x2, y2);
        prev_s = s;
    }


    // Dibujar mapa ////////////////////////////////////////////////////////////

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
                let est_y = (y.max(0.) as usize).min(height as usize);
                let est_y2 = (y2.max(0.) as usize).min(height as usize);
                let (x_offset, _) = latlon_to_rel_px(sat_state[est_y].latlon);
                let (x2_offset, _) = latlon_to_rel_px(sat_state[est_y2].latlon);

                let (xa, ya) = px_rel_to_abs((x - x_offset, y));
                let (xa2, ya2) = px_rel_to_abs((x2 - x2_offset, y2));
                draw::draw_line(img, image::Rgb([0, 255, 0]), xa, ya, xa2, ya2);
                // draw::draw_line(&mut img2, image::Luma([255]), lon_to_x(pt.x), lat_to_y(pt.y), lon_to_x(prev_pt.x), lat_to_y(prev_pt.y));
                prev_pt = pt;
            }
        }
    }
    // img2.save("./a.png").unwrap();
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
