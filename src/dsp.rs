pub type Sample = i32;
pub type Signal = Vec<Sample>;

/// Get biggest sample.
pub fn get_max(vector: &Signal) -> &Sample {
    let mut max: &Sample = &0;
    for sample in vector.iter() {
        if sample > max {
            max = sample;
        }
    }

    max
}

/// Resample signal by upsampling, filtering and downsampling..
///
/// L is the interpolation factor and M the decimation one.
pub fn resample(signal: &Signal, l: u8, m: u8) -> Signal{
    let l = l as usize;
    let m = m as usize;
    let mut upsampled: Signal = vec![0; signal.len() * l];

    for (i, sample) in signal.iter().enumerate() {
        upsampled[i * l] = *sample;
    }

    upsampled
}
