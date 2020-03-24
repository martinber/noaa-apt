/// Functions to read shapefiles and draw maps.
///

use image::{ImageBuffer, Luma};
use shapefile::Shape;

use crate::draw;

pub fn draw_map(
    img: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
) {
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
                println!("{:?}", pt.x.max(1.) as u32);
                draw::draw_line(
                    img,
                    (prev_pt.x.max(1.) * 5.) as u32, (prev_pt.y.max(1.) * 5.) as u32,
                    (pt.x.max(1.) * 5.) as u32, (pt.y.max(1.) * 5.) as u32
                );
                prev_pt = pt;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test() {

        let filename = "./res/shapefiles/countries.shp";
        let reader = shapefile::Reader::from_path(filename).unwrap();

        for result in reader.iter_shapes_as::<shapefile::Polygon>() {
            let polygon = result.unwrap(); //TODO
            println!("{:?}", polygon.rings());
        }
        assert!(false);

    }
}
