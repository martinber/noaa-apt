//! Code for telemetry decoding.

use noaa_apt::PX_PER_ROW;
use dsp::Signal;
use err;
use context::{Context, Step};


/// Determines if working channel A or B.
pub enum Channel {
    A,
    B,
}

/// Contains the telemetry data.
///
/// Also methods to extract values from it.
pub struct Telemetry {
    // One value for each wedge on each band
    values_a: Vec<f32>,
    values_b: Vec<f32>,
}

impl Telemetry {

    /// Read telemetry from received bands.
    ///
    /// Takes two signals containing the horizontal averages of each band (8
    /// values per wedge), also takes the row where to start reading the frame.
    pub fn from_bands(means_a: &Signal, means_b: &Signal, row: usize) -> Telemetry{

        // Calculate the mean of contiguous 8 values, starting from the given
        // one until wedge 9 of the next frame
        // As a result we have the average of wedges 1-16 and 1-9
        let means_a: Signal = means_a[row..]
            .chunks_exact(8).map(|x| x.iter().sum::<f32>() / 8.).take(16 + 9).collect();
        let means_b: Signal = means_b[row..]
            .chunks_exact(8).map(|x| x.iter().sum::<f32>() / 8.).take(16 + 9).collect();

        let telemetry = Telemetry {
            values_a: (1..=16).map(|wedge|
                // Contrast wedges 1-9 are averaged to the ones on the next frame
                if wedge <= 9 {
                    (means_a[wedge - 1] + means_a[wedge + 16 - 1]) / 2.
                } else {
                    means_a[wedge - 1]
                }
            ).collect(),

            values_b: (1..=16).map(|wedge|
                // Contrast wedges 1-9 are averaged to the ones on the next frame
                if wedge <= 9 {
                    (means_b[wedge - 1] + means_b[wedge + 16 - 1]) / 2.
                } else {
                    means_b[wedge - 1]
                }
            ).collect(),
        };

        debug!("Telemetry wedges_a: {:?}, wedges_b: {:?}",
            telemetry.values_a, telemetry.values_b);

        telemetry
    }

    /// Get value of wedge.
    ///
    /// Float values that doesn't mean anything by itself, you should use it for
    /// comparison against another wedge.
    pub fn get_wedge_value(&self, wedge: u32, channel: Option<Channel>) -> f32 {

        let wedge = wedge as usize;
        // Substract one to given wedge because indices start at zero and wedges
        // start at one
        match channel {
            Some(Channel::A) => self.values_a[wedge - 1],
            Some(Channel::B) => self.values_b[wedge - 1],

            // Average both channels
            None => (self.values_a[wedge - 1] + self.values_b[wedge - 1]) / 2.,
        }
    }

    /// Get channel name.
    pub fn get_channel_name(&self, channel: Channel) -> &str {

        // Find the contrast wedge (1 to 9) closest to the channel
        // identification wedge (16)

        // See documentation.

        let value = self.get_wedge_value(16, Some(channel));

        let contrast_wedges = (1..=9).map(|i| self.get_wedge_value(i, None));

        let channel_names = ["1", "2", "3a", "4", "5", "3b", "Unknown", "Unknown", "Unknown"];

        let (name, _difference) = channel_names.iter().zip(contrast_wedges)
            .min_by(|a, b|
                (a.1 - value).abs().partial_cmp(&(b.1 - value).abs()).expect("Can't compare values")
            ).expect("Empty zip");

        name
    }
}

/// Read telemetry from aligned signal.
///
/// Takes already synced signal, it's a Vec where the first PX_PER_ROW values
/// represent the first line of the image, the next PX_PER_ROW represent the
/// next line, etc.
pub fn read_telemetry(context: &mut Context, signal: &Signal) -> err::Result<Telemetry> {

    // Sample of telemetry band used for correlation. Only contrast wedges
    // (1 to 9) are given. Each value is repeated 8 times because the height of
    // the wedges is 8 pixels
    let telemetry_sample: Signal = [
        31., 63., 95., 127., 159., 191., 224., 255., 0., // For contrast
        0., 0., 0., 0., 0., 0., 0.,                      // Variable
        31., 63., 95., 127., 159., 191., 224., 255., 0.  // For contrast
    ].iter().flat_map(|x| std::iter::repeat(*x).take(8)).collect();

    // Reserve vectors, vector length is the height of the image

    // Horizontal average of both bands
    let mut mean_a: Signal = Vec::with_capacity(signal.len() / PX_PER_ROW as usize);
    let mut mean_b: Signal = Vec::with_capacity(signal.len() / PX_PER_ROW as usize);
    // Horizontal variance of both telemetry bands, indicates if there is noise
    let mut variance: Signal = Vec::with_capacity(signal.len() / PX_PER_ROW as usize);

    // Iterate a row at a time (each row is one pixel high)
    for line in signal.chunks_exact(PX_PER_ROW as usize) {

        // Values on each bamd
        let a_values = &line[994..(994+44)];
        let b_values = &line[2034..(2034+44)];

        // Horizontal average
        let curr_mean_a: f32 = a_values.iter().sum::<f32>() / 44.;
        let curr_mean_b: f32 = b_values.iter().sum::<f32>() / 44.;
        mean_a.push(curr_mean_a);
        mean_b.push(curr_mean_b);

        // Horizontal variance
        variance.push(
            (a_values.iter().map(|x| (x - curr_mean_a).powi(2)).sum::<f32>() +
             b_values.iter().map(|x| (x - curr_mean_b).powi(2)).sum::<f32>()) / 88.
        );
    }

    // Cross correlation between telemetry band averages and the telemetry
    // sample, has peaks where telemetry frames start
    let mut corr: Signal = Vec::new();

    // Cross correlation divided by the standard deviation, has peaks where
    // telemetry frames with low standard deviation start
    let mut quality: Signal = Vec::new();

    // These will be used only if the steps are exported
    if context.export {
        corr.reserve(signal.len() / PX_PER_ROW as usize);
        quality.reserve(signal.len() / PX_PER_ROW as usize)
    };

    // Row with the best quality
    let mut best: (usize, f32) = (0, 0.); // (row, quality)

    // Cross correlation of both telemetry bands with a sample
    for i in 0 .. mean_a.len() - telemetry_sample.len() {
        let mut sum: f32 = 0.;
        for j in 0..telemetry_sample.len() {
            sum += telemetry_sample[j] * mean_a[i + j];
            sum += telemetry_sample[j] * mean_b[i + j];
        }

        // sqrt() for standard deviation instead of variance, otherwise variance
        // is too big and noise affects the quality estimation too much compared
        // to the correlation
        // Check standard deviation on the same places where we cross correlate,
        // that's why I use telemetry_sample.len()
        let q = sum / variance[i..(i + telemetry_sample.len())].iter()
            .map(|x| x.sqrt()).sum::<f32>();

        if q > best.1 {
            best = (i, q);
        }

        if context.export {
            corr.push(sum);
            quality.push(q);
        }
    }

    let telemetry = Telemetry::from_bands(&mean_a, &mean_b, best.0);
    info!("Channel A: {}, Channel B: {}",
        telemetry.get_channel_name(Channel::A), telemetry.get_channel_name(Channel::B));

    context.step(Step::signal("telemetry_a", &mean_a, None))?;
    context.step(Step::signal("telemetry_b", &mean_b, None))?;
    context.step(Step::signal("telemetry_correlation", &corr, None))?;
    context.step(Step::signal("telemetry_variance", &variance, None))?;
    context.step(Step::signal("telemetry_quality", &quality, None))?;

    Ok(telemetry)
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Check if two floats are equal given some margin of precision
    fn assert_roughly_equal(a: f32, b: f32) {
        assert_ulps_eq!(a, b, max_ulps = 10)
    }

    #[test]
    fn test_telemetry_from_bands() {
        // Sample wedge with mean = 1
        let sample_wedge: [f32; 8] = [1., 1.2, 0.8, 1.1, 0.9, 0.7, 1.3, 1.];

        let index_row = 8; // Where telemetry starts

        // Sample telemetry for channel A
        let means_a: Signal = sample_wedge.iter().map(|x| x * -5234.) // Should not read this
            .chain(sample_wedge.iter().map(|x| x * 1.)) // Wedge 1
            .chain(sample_wedge.iter().map(|x| x * 2.)) // Wedge 2
            .chain(sample_wedge.iter().map(|x| x * 3.)) // Wedge 3
            .chain(sample_wedge.iter().map(|x| x * 4.)) // Wedge 4
            .chain(sample_wedge.iter().map(|x| x * 5.)) // Wedge 5
            .chain(sample_wedge.iter().map(|x| x * 6.)) // Wedge 6
            .chain(sample_wedge.iter().map(|x| x * 7.)) // Wedge 7
            .chain(sample_wedge.iter().map(|x| x * 8.)) // Wedge 8
            .chain(sample_wedge.iter().map(|x| x * 9.)) // Wedge 9
            .chain(sample_wedge.iter().map(|x| x * 10.)) // Wedge 10
            .chain(sample_wedge.iter().map(|x| x * 11.)) // Wedge 11
            .chain(sample_wedge.iter().map(|x| x * 12.)) // Wedge 12
            .chain(sample_wedge.iter().map(|x| x * 13.)) // Wedge 13
            .chain(sample_wedge.iter().map(|x| x * 14.)) // Wedge 14
            .chain(sample_wedge.iter().map(|x| x * 15.)) // Wedge 15
            .chain(sample_wedge.iter().map(|x| x * 16.)) // Wedge 16
            .chain(sample_wedge.iter().map(|x| x * 1.)) // Wedge 1
            .chain(sample_wedge.iter().map(|x| x * 2.)) // Wedge 2
            .chain(sample_wedge.iter().map(|x| x * 3.)) // Wedge 3
            .chain(sample_wedge.iter().map(|x| x * 4.)) // Wedge 4
            .chain(sample_wedge.iter().map(|x| x * 5.)) // Wedge 5
            .chain(sample_wedge.iter().map(|x| x * 6.)) // Wedge 6
            .chain(sample_wedge.iter().map(|x| x * 7.)) // Wedge 7
            .chain(sample_wedge.iter().map(|x| x * 8.)) // Wedge 8
            .chain(sample_wedge.iter().map(|x| x * 9.)) // Wedge 9
            .chain(sample_wedge.iter().map(|x| x * -5234.)) // Should not read this
            .collect();

        // Sample telemetry for channel B is one plus telemetry A, just to check
        // if channel A is being averaged to channel B
        let means_b = means_a.iter().map(|x| x + 1.).collect();

        let telemetry = Telemetry::from_bands(&means_a, &means_b, index_row);

        for wedge in 1..=16 {
            assert_roughly_equal(
                telemetry.get_wedge_value(wedge, Some(Channel::A)),
                wedge as f32
            );
            assert_roughly_equal(
                telemetry.get_wedge_value(wedge, Some(Channel::B)),
                (wedge as f32) + 1.
            );
            assert_roughly_equal(
                telemetry.get_wedge_value(wedge, None),
                (wedge as f32) + 0.5
            );
        }
    }

    #[test]
    fn test_telemetry_get_channel() {
        // Means for wedges 1 to 15
        let sample_means: [f32; 15] = [
            1., 2., 3., 4., 5., 6., 7., 8., 9., // Contrast
            3., 3., 3., 3., 3., 3.,
        ];

        // Create telemetry with given values for channel identification wedges
        let create_telemetry = |channel_a, channel_b| {
            let mut values_a = sample_means.to_vec();
            let mut values_b = sample_means.to_vec();
            values_a.push(channel_a);
            values_b.push(channel_b);
            Telemetry { values_a, values_b }
        };

        // (Expected channel A name, Channel A wedge 16 value,
        //  Expected channel B name, Channel B wedge 16 value)
        let test_cases: [(&str, f32, &str, f32); 8]= [
            ("1", 1.,       "2", 2.),
            ("3a", 3.,      "3b", 6.),
            ("4", 4.,       "5", 5.),
            ("Unknown", 7., "Unknown", 8.),
            ("Unknown", 9., "Unknown", 1000.),

            ("1", 1.4,      "2", 1.6),
            ("3a", 2.6,     "3a", 3.4),
            ("1", -1000.,   "5", 5.4),
        ];

        for case in test_cases.iter() {
            let telemetry = create_telemetry(case.1, case.3);
            assert_eq!(telemetry.get_channel_name(Channel::A), case.0);
            assert_eq!(telemetry.get_channel_name(Channel::B), case.2);
        }
    }
}
