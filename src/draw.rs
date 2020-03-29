/// Functions for drawing lines, polygons.
///

use image::{ImageBuffer, Luma};

/// Draw line.
pub fn draw_line(
    img: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32,
) {
    // Algorithm from https://www.redblobgames.com/grids/line-drawing.html
    // MIT license, by Amit Patel

    let x1 = x1 as i32;
    let y1 = y1 as i32;
    let x2 = x2 as i32;
    let y2 = y2 as i32;

    let dx = x2 - x1;
    let dy = y2 - y1;
    let n = i32::max(dx.abs(), dy.abs());
    let div_n: f32 = if n == 0 {
        0.
    } else {
        1. / (n as f32)
    };
    let xstep = dx as f32 * div_n;
    let ystep = dy as f32 * div_n;

    let mut x = x1 as f32;
    let mut y = y1 as f32;
    for step in 0 ..= n {
        x += xstep;
        y += ystep;
        // TODO
        img.put_pixel(
            (x.max(0.) as u32).min(img.width() - 1),
            (y.max(0.) as u32).min(img.height() - 1),
            Luma([255])
        );
    }
}

// http://alienryderflex.com/polygon_fill/
