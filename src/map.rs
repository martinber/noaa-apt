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

    let (sats, _errors) = satellite::io::parse_multiple(include_str!("../weather-2020-03.txt"));
    let mut sat = sats.iter().find(|&sat| sat.name == Some("NOAA 19".to_string()))
        .expect("not found in test TLE file").clone();

    let mut time: Vec<chrono::DateTime<_>> = Vec::with_capacity(height as usize);

    // let start_time = chrono::Utc.timestamp(timestamp, 0); // 0 milliseconds
    // println!("ts {:?}", time);
    time.push(chrono::Utc.ymd(2020, 03, 26).and_hms(00, 02, 04)); // 0 milliseconds
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

        if let Some(prev_state) = sat_state.last() {
            let mut side_az = geo::azimuth(lat, lon, prev_state.lat, prev_state.lon) - 88.;
            // TODO:
            if side_az == 92. || side_az == -88. {
                side_az = prev_state.side_az;
            }
            sat_state.push(SatState { lat, lon, side_az });
            println!("{:?}", sat_state.last().unwrap());
        } else {
            sat_state.push(SatState { lat, lon, side_az: 0. });
        }
    }



    ///////////////////////

    // use std::f32::consts::PI;
//
    // let a = 6378137.; // meters, a is the semi-major axis
//
    // // swath with in meters
    // // https://directory.eoportal.org/web/eoportal/satellite-missions/n/noaa-poes-series-5th-generation
    // let swath_m = 2900000.; //m
//
    // // swath with in degree (for a spherical earth)
    // let swath = swath_m / (a*PI/180.);

    use std::env;

    let y_res = match env::var("Y_RES") {
        Ok(v) => {
            match v.parse::<f64>() {
                Ok(n) => n,
                Err(_) => 0.028659123475760308,
            }
        },
        Err(_) => 0.028659123475760308,
    };

    let x_res = match env::var("X_RES") {
        Ok(v) => {
            match v.parse::<f64>() {
                Ok(n) => n,
                Err(_) => y_res,
            }
        },
        Err(_) => y_res,
    };

    ///////////////////////

    let mut geoimg: Vec<[(f64, f64); 2080]> = vec![[(0., 0.); 2080]; height as usize];
    // let mut geoimg: [[(f64, f64); 1000]; 2080] = [[(0., 0.); 1000]; 2080];

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



    ///////////////////////

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
                draw::draw_line(
                    &mut img2,
                    lon_to_x(pt.x),
                    lat_to_y(pt.y),
                    lon_to_x(prev_pt.x),
                    lat_to_y(prev_pt.y)
                );
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
