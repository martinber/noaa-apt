/// Functions to read shapefiles and draw maps.
///

use chrono::prelude::*;
use image::{ImageBuffer, Luma};
use shapefile::Shape;

use crate::draw;
use crate::geo;



pub fn draw_map(
    img: &mut ImageBuffer<Luma<u8>, Vec<u8>>, timestamp: i64,
) {

    let (sats, _errors) = satellite::io::parse_multiple(include_str!("../weather.txt"));
    let mut sat = sats.iter().find(|&sat| sat.name == Some("NOAA 19".to_string()))
        .expect("not found in test TLE file").clone();

    let time = chrono::Utc.timestamp(timestamp, 0); // 0 milliseconds
    println!("ts {:?}", time);
    let time = chrono::Utc.ymd(2018, 12, 22).and_hms(20, 39, 41); // 0 milliseconds
    println!("ts {:?}", time);
    let result = satellite::propogation::propogate_datetime(&mut sat, time).unwrap();
    let gmst = satellite::propogation::gstime::gstime_datetime(time);
    let sat_pos = satellite::transforms::eci_to_geodedic(&result.position, gmst);

    let ref_lat = (sat_pos.latitude * satellite::constants::RAD_TO_DEG) as f32;
    let ref_lon = (sat_pos.longitude * satellite::constants::RAD_TO_DEG) as f32;

    // let time = chrono::Utc.timestamp(timestamp + 100, 0); // 0 milliseconds
    let time = time + chrono::Duration::seconds(1);
    let result = satellite::propogation::propogate_datetime(&mut sat, time).unwrap();
    let gmst = satellite::propogation::gstime::gstime_datetime(time);
    let sat_pos = satellite::transforms::eci_to_geodedic(&result.position, gmst);

    let ref2_lat = (sat_pos.latitude * satellite::constants::RAD_TO_DEG) as f32;
    let ref2_lon = (sat_pos.longitude * satellite::constants::RAD_TO_DEG) as f32;

    let ref_az = geo::azimuth(ref_lat, ref_lon, ref2_lat, ref2_lon);
    let y_res = geo::distance(ref_lat, ref_lon, ref2_lat, ref2_lon) / 2.; // Lineas por segundo

    println!("time {:?}, ref_lat {}, ref_lon {}, ref_az {}, y_res {}", time, ref_lat, ref_lon, ref_az, y_res);

    ///////////////////////

    let x_res = y_res;
    let latlon_to_px = |lat: f32, lon: f32| -> (u32, u32) {

        use std::f32::consts::PI;

        let dist = geo::distance(lat, lon, ref_lat, ref_lon);
        let az = geo::azimuth(ref_lat, ref_lon, lat, lon) + 180.;

        let tmp = geo::azimuth(ref_lat, ref_lon, lat, lon);

        let B = (tmp - ref_az) / 360. * 2. * PI;
        let c = dist / 360. * 2. * PI;

        let a = (B.cos() * c.tan()).atan();
        let b = (B.sin() * c.sin()).asin();

        let a = a / (2. * PI) * 360.;
        let b = b / (2. * PI) * 360.;

        let x = ((-b / x_res) + 539.).max(1.).min(2070.) as u32;
        let y = ((a / y_res)).max(1.).min(1600.) as u32;

        if x > 400 && x < 500 && y > 800 && y < 900 {
            println!("lat {}, lon {}, dist {}, az {}, x {}, y {}", lat, lon, dist, az, x, y);
        }

        (x, y)

    };
    println!("ref {}, {}", latlon_to_px(ref_lat, ref_lon).0, latlon_to_px(ref_lat, ref_lon).1);

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
                // y: lat, x: lon,
                let (x, y) = latlon_to_px(pt.y as f32, pt.x as f32);
                let (x2, y2) = latlon_to_px(prev_pt.y as f32, prev_pt.x as f32);
                draw::draw_line(img, x, y, x2, y2);
                prev_pt = pt;
            }
        }
    }
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
