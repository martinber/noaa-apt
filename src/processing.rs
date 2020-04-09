//! Image processing functions.

use crate::decode::{Image, Pixel, PX_PER_CHANNEL};

/// Rotate image without changing the location of the channels.
pub fn rotate(img: &Image) -> Image {

    // Reverses the array and keeps the A channel on the left. Care is taken to
    // leave lines from the A channel at the same height as the B channel.
    // Otherwise there is a vertical offset of one pixel between each channel.
    // This is done by swapping the interleaving chunks corresponding to A and B
    // frames.

    // let reversed: Vec<Pixel> = img.pixels().rev().copied().collect();
    // Image::from_raw(
        // img.width(),
        // img.height(),
        // reinterleave(&reversed, PX_PER_CHANNEL as usize)
    // ).expect("reinterleave() returned a Vec of different length")
    img.clone()
}

/// Takes an interleaved array and swaps the order of the chunks.
///
/// Used for rotating the image. Takes a Vec and the length of each chunk.
///
/// `reinterleave(array, 4)` will take this array
///
///     [a1, a2, a3, a4, b1, b2, b3, b4, a5, a6, a7, a8, b5, b6, b7, b8]
///
/// And return:
///
///     [b1, b2, b3, b4, a1, a2, a3, a4, b5, b6, b7, b8, a5, a6, a7, a8]
fn reinterleave<T: std::marker::Copy>(signal: &Vec<T>, chunk_size: usize) -> Vec<T> {

    let a_chunks = signal.chunks(chunk_size).step_by(2);
    let b_chunks = signal.chunks(chunk_size).skip(1).step_by(2);

    itertools::interleave(b_chunks, a_chunks).flatten().copied().collect()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_reinterleave() {
        let expected = vec![ 6,  7,  8,  9, 10,  1,  2,  3,  4,  5,
                            16, 17, 18, 19, 20, 11, 12, 13, 14, 15,
                            26, 27, 28, 29, 30, 21, 22, 23, 24, 25,
                            36, 37, 38, 39, 40, 31, 32, 33, 34, 35];

        let test_values = (1..=40).collect();

        assert_eq!(expected, reinterleave(&test_values, 5));
    }
}
