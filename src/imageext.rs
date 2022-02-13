/// Some extra utilities for working with images, that use or complement
/// available functions from `image` crate
use image::{GenericImage, GenericImageView, Pixel, Rgba, RgbaImage, SubImage};
use lab::Lab;
use std::convert::TryInto;


/// A set of per-channel histograms from an image with 8 bits per channel.
pub struct ChannelHistogram {
    /// Per-channel histograms.
    pub channels: Vec<[u32; 256]>,
}

/// A set of per-channel cumulative histograms from an image with 8 bits per channel.
pub struct CumulativeChannelHistogram {
    /// Per-channel cumulative histograms.
    pub channels: Vec<[u32; 256]>,
}

/// Equalize the histogram of the grayscale (but still Rgba image with
/// R = G = B, A = 255), by equalizing the histogram of one of channels (R),
/// and using that for all the other (G, B). Alpha channel is not modified.
pub fn equalize_histogram_grayscale(sub_image: &mut SubImage<&mut RgbaImage>) {
    // since it's a grayscale image (R = G = B, A = 255), use R channel histogram:
    let hist = cumulative_histogram_rgba(sub_image).channels[0];
    let total = hist[255] as f32;

    for y in 0..sub_image.height() {
        for x in 0..sub_image.width() {
            let mut p = sub_image.get_pixel(x, y);

            // Each histogram has length 256 and RgbaImage has 8 bits per pixel
            let fraction = hist[p.channels()[0] as usize] as f32 / total;

            // apply f to channels r, g, b and apply g to alpha channel
            p.apply_with_alpha(
                // for R, G, B, use equalized values:
                |_| (255. * fraction) as u8,
                // for A, leave unmodified
                |alpha| alpha,
            );

            sub_image.put_pixel(x, y, p);
        }
    }
}

/// Equalize the histogram of the color subimage by converting Rgb -> Lab,
/// equalizing the L (lightness) histogram, and converting back Lab -> Rgb.
pub fn equalize_histogram_color(sub_image: &mut SubImage<&mut RgbaImage>) {
    let mut lab_pixels: Vec<Lab> = rgb_to_lab(sub_image);

    let lab_hist = cumulative_histogram_lab(&lab_pixels);
    let total = lab_hist[100] as f32;

    lab_pixels.iter_mut().for_each(|p: &mut Lab| {
        // casting p.l from f32 to usize rounds towards 0
        // l is in range [0..100] inclusive, lab_hist has lenght 101
        let fraction = lab_hist[p.l as usize] as f32 / total;
        p.l = 100. * fraction;
    });
    lab_to_rgb_mut(&lab_pixels, sub_image);
}

/// Returns a vector of Lab pixel values, alpha channel value is not used.
fn rgb_to_lab(sub_image: &mut SubImage<&mut RgbaImage>) -> Vec<Lab> {
    sub_image
        .pixels()
        .map(|(_x, _y, p)| {
            let rgb: [u8; 3] = p.channels()[..3].try_into().unwrap();
            Lab::from_rgb(&rgb)
        })
        .collect()
}

/// Converts Lab to Rgb and modifies the R, B, G values of pixels
/// in the original subimage. The value of the alpha channel is unmodified.
fn lab_to_rgb_mut(lab_pixels: &Vec<Lab>, sub_image: &mut SubImage<&mut RgbaImage>) {
    let rgb_pixels: Vec<[u8; 3]> = lab_pixels.iter().map(|x: &Lab| x.to_rgb()).collect();

    let height = sub_image.height();
    let width = sub_image.width();

    for y in 0..height {
        for x in 0..width {
            let p = sub_image.get_pixel(x, y);
            let [r, g, b] = rgb_pixels[(y * width + x) as usize];
            let a = p.channels()[3]; // get original alpha channel
            sub_image.put_pixel(x, y, Rgba([r, g, b, a]));
        }
    }
}

/// Calculates the cumulative histograms for each channel of the subimage.
fn cumulative_histogram_rgba(
    sub_image: &mut SubImage<&mut RgbaImage>,
) -> CumulativeChannelHistogram {
    let mut hist = histogram_rgba(sub_image);
    for c in 0..hist.channels.len() {
        for i in 1..hist.channels[c].len() {
            hist.channels[c][i] += hist.channels[c][i - 1];
        }
    }
    CumulativeChannelHistogram {
        channels: hist.channels,
    }
}

/// Calculates the histograms for each channel of the subimage.
fn histogram_rgba(sub_image: &mut SubImage<&mut RgbaImage>) -> ChannelHistogram {
    let mut hist = vec![[0u32; 256]; 4];

    sub_image.pixels().for_each(|(_x, _y, p)| {
        for (i, c) in p.channels().iter().enumerate() {
            hist[i][*c as usize] += 1;
        }
    });
    ChannelHistogram { channels: hist }
}

/// Calculates the cumulative histogram using the L (lightness) channel.
/// L values are in range [0..100] inclusive, so the resulting array
/// has 101 elements: `[u32; 101]`
fn cumulative_histogram_lab(lab_pixels: &Vec<Lab>) -> [u32; 101] {
    let mut hist = histogram_lab(lab_pixels);
    for i in 1..hist.len() {
        hist[i] += hist[i - 1];
    }
    hist
}

/// Calculates the histogram using the L (lightness) channel.
/// L values are in range [0..100] inclusive, so the resulting array
/// has 101 elements: `[u32; 101]`.
/// If the histogram for the other channels is needed in the future,
/// consider defining a struct similar to `ChannelHistogram`.
fn histogram_lab(lab_pixels: &Vec<Lab>) -> [u32; 101] {
    let mut hist = [0u32; 101];
    for p in lab_pixels {
        hist[p.l as usize] += 1; // use L (lightness) channel
    }
    hist
}
