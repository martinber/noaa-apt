//! Image processing functions.

use image::{GenericImageView, GenericImage};

use crate::decode::{Image, Pixel, PX_PER_CHANNEL};
use crate::err;

/// Rotate image without changing the location of the channels.
///
/// Takes as an argument a raw image, that is, with syncing frames and telemetry
/// bands. These will not be removed.
///
/// Care is taken to leave lines from the A channel at the same height as the B
/// channel. Otherwise there can be a vertical offset of one pixel between each
/// channel.
pub fn rotate(img: &Image) -> err::Result<Image> {

    // Create image with channel A and B swapped
    let mut output = Image::new(img.width(), img.height());
    let channel_a = img.view(0, 0, PX_PER_CHANNEL, img.height());
    let channel_b = img.view(PX_PER_CHANNEL, 0, PX_PER_CHANNEL, img.height());
    output.copy_from(&channel_b, 0, 0)?;
    output.copy_from(&channel_a, PX_PER_CHANNEL, 0)?;

    image::imageops::rotate180_in_place(&mut output);

    Ok(output)
}
