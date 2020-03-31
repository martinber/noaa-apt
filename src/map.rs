/// Functions to read shapefiles and draw maps.
///

use chrono::prelude::*;
use image::{ImageBuffer, Luma};
use shapefile::Shape;

use crate::draw;
use crate::geo;

#[derive(Debug)]
struct SatState {
    lat: f64,
    lon: f64,
    side_az: f64,
}



pub fn draw_map(
    img: &mut ImageBuffer<Luma<u8>, Vec<u8>>, timestamp: i64, height: u32
) {
    let yaw = match env::var("YAW") {
        Ok(v) => {
            match v.parse::<f64>() {
                Ok(n) => n,
                Err(_) => 0.,
            }
        },
        Err(_) => 0.,
    };

    let (sats, _errors) = satellite::io::parse_multiple(include_str!("../weather-2020-03.txt"));
    let mut sat = sats.iter().find(|&sat| sat.name == Some("NOAA 18".to_string()))
        .expect("not found in test TLE file").clone();

    // Generar vector de estados ///////////////////////////////////////////////

    let start_time = match env::var("DATE") {
        Ok(s) => {
            match chrono::Utc.datetime_from_str(&s, "%Y-%m-%d %T") {
                Ok(n) => n,
                Err(_) => panic!("No se pudo parsear fecha")
            }
        },
        Err(_) => panic!("No hay fecha")
    };

    let mut time: Vec<chrono::DateTime<_>> = Vec::with_capacity(height as usize);

    // let start_time = chrono::Utc.timestamp(timestamp, 0); // 0 milliseconds
    // println!("ts {:?}", time);
    time.push(start_time); // 0 milliseconds
    let time_step = chrono::Duration::milliseconds(500); // Seconds per line

    for i in 0..height {
        time.push(*time.last().unwrap() + time_step);
        println!("{:?}", time.last().unwrap());

    }

    let mut sat_state: Vec<SatState> = Vec::with_capacity(height as usize);
    for t in time {
        let result = satellite::propogation::propogate_datetime(&mut sat, t).unwrap();
        let gmst = satellite::propogation::gstime::gstime_datetime(t);
        let sat_pos = satellite::transforms::eci_to_geodedic(&result.position, gmst);

        let lat = (sat_pos.latitude * satellite::constants::RAD_TO_DEG);
        let lon = (sat_pos.longitude * satellite::constants::RAD_TO_DEG);

        sat_state.push(SatState { lat, lon, side_az: 0. });
    }

    for i in 1..sat_state.len()-1 {
        let s1 = &sat_state[i-1];
        let s2 = &sat_state[i+1];
        let mut side_az = geo::azimuth(s1.lat, s1.lon, s2.lat, s2.lon) + 90. + yaw;
        sat_state[i].side_az = side_az;
    }

    // Calcular resolucion /////////////////////////////////////////////////////

    let s1 = &sat_state[0];
    let s2 = sat_state.last().unwrap();
    let y_res = geo::distance(s1.lat, s1.lon, s2.lat, s2.lon) / height as f64;

    let x_res = 0.02866;

    use std::env;

    let y_res = match env::var("Y_RES") {
        Ok(v) => {
            match v.parse::<f64>() {
                Ok(n) => n,
                Err(_) => y_res,
            }
        },
        Err(_) => y_res,
    };

    let x_res = match env::var("X_RES") {
        Ok(v) => {
            match v.parse::<f64>() {
                Ok(n) => n,
                Err(_) => x_res,
            }
        },
        Err(_) => x_res,
    };

    // latlon_to_rel_px ////////////////////////////////////////////////////////

    let ref_lat = sat_state[0].lat;
    let ref_lon = sat_state[0].lon;
    let ref2_lat = sat_state.last().unwrap().lat;
    let ref2_lon = sat_state.last().unwrap().lon;
    let ref_az = geo::azimuth(ref_lat, ref_lon, ref2_lat, ref2_lon);
    let latlon_to_rel_px = |lat: f64, lon: f64| -> (f64, f64) {

        use std::f64::consts::PI;

        let dist = geo::distance(lat, lon, ref_lat, ref_lon);
        let az = geo::azimuth(ref_lat, ref_lon, lat, lon) + 180.;

        let tmp = geo::azimuth(ref_lat, ref_lon, lat, lon);

        let B = (tmp - ref_az) / 360. * 2. * PI;
        let c = dist / 360. * 2. * PI;

        let a = (B.cos() * c.tan()).max(-PI/2.).min(PI/2.).atan();
        let b = (B.sin() * c.sin()).max(-PI/2.).min(PI/2.).asin();

        let a = a / (2. * PI) * 360.;
        let b = b / (2. * PI) * 360.;

        let x = -b / x_res;
        let y = a / y_res + yaw * x;

        (x, y)
    };

    // px_rel_to_abs ///////////////////////////////////////////////////////////

    let px_rel_to_abs = |x: f64, y: f64| -> (u32, u32) {

        let x_abs = (x + 539.).max(0.).min(2070.) as u32;
        let y_abs = y.max(0.).min(height as f64) as u32;

        (x_abs, y_abs)
    };

    // Generar imagen equirectangular //////////////////////////////////////////

    let mut geoimg: Vec<[(f64, f64); 2080]> = vec![[(0., 0.); 2080]; height as usize];

    let mut max_lat = sat_state[0].lat;
    let mut min_lat = sat_state[0].lat;
    let mut max_lon = sat_state[0].lon;
    let mut min_lon = sat_state[0].lon;
    for y in 0..(height as usize) {
        for x in 0..2080 {
            geoimg[y][x] = geo::reckon(
                sat_state[y].lat, sat_state[y].lon,
                -((x as f64) - 539.) * x_res,
                sat_state[y].side_az
            );
            max_lat = max_lat.max(geoimg[y][x].0);
            min_lat = min_lat.min(geoimg[y][x].0);
            max_lon = max_lon.max(geoimg[y][x].1);
            min_lon = min_lon.min(geoimg[y][x].1);
        }
        println!("{:?}, {:?}", geoimg[y][539], geoimg[y][0]);
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

    // Dibujar linea central ///////////////////////////////////////////////////

    let mut prev_s = &sat_state[0];
    for s in sat_state.iter() {
        // y: lat, x: lon,
        let (x, y) = latlon_to_rel_px(s.lat, s.lon);
        let (x2, y2) = latlon_to_rel_px(prev_s.lat, prev_s.lon);
        let (xa, ya) = px_rel_to_abs(x, y);
        let (xa2, ya2) = px_rel_to_abs(x2, y2);

        draw::draw_line(img, image::Luma([0]), xa, ya, xa2, ya2);
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
                let (x, y) = latlon_to_rel_px(pt.y, pt.x);
                let (x2, y2) = latlon_to_rel_px(prev_pt.y, prev_pt.x);
                let est_y = (y.max(0.) as usize).min(height as usize);
                let (x_offset, y_offset) = latlon_to_rel_px(sat_state[est_y].lat, sat_state[est_y].lon);
                let (x2_offset, y2_offset) = latlon_to_rel_px(sat_state[est_y].lat, sat_state[est_y].lon);

                let (xa, ya) = px_rel_to_abs(x - x_offset, y);
                let (xa2, ya2) = px_rel_to_abs(x2 - x2_offset, y2);
                draw::draw_line(img, image::Luma([255]), xa, ya, xa2, ya2);
                draw::draw_line(&mut img2, image::Luma([255]), lon_to_x(pt.x), lat_to_y(pt.y), lon_to_x(prev_pt.x), lat_to_y(prev_pt.y));
                prev_pt = pt;
            }
        }
    }
    img2.save("./a.png").unwrap();
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
            // println!("{:?}", polygon.rings());
        // }
        // assert!(false);
//
    // }
}
