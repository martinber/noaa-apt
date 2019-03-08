//! Contains the Context struct.

use dsp::{Signal, Rate};
use noaa_apt::PX_PER_ROW;
use err;
use wav;


/// Different kinds of steps available.
#[derive(Debug, PartialEq)]
enum Variant {
    Signal,
    Filter,
}

/// Represents a step on the decoding process.
///
/// Some steps can happen twice or can not happen, the `Context` has information
/// about the order in which the steps can ocurr.
///
/// The `Rate` is optional because some functions on the dsp module don't know
/// the sample rate. In those cases, the metadata should have it. Also, when
/// saving filters, there is no rate.
///
/// The references only need to be valid until calling `Context::step()`.
#[derive(Debug)]
pub struct Step<'a> {
    variant: Variant,
    id: &'a str,
    signal: &'a Signal,
    rate: Option<Rate>,
}

impl<'a> Step<'a> {

    /// Create a signal step.
    pub fn signal(id: &'a str, signal: &'a Signal, rate: Option<Rate>) -> Step<'a> {
        Step {
            variant: Variant::Signal,
            id,
            signal,
            rate,
        }
    }

    /// Create a filter step.
    pub fn filter(id: &'a str, filter: &'a Signal) -> Step<'a> {
        Step {
            variant: Variant::Filter,
            id,
            signal: filter,
            rate: None,
        }
    }
}

/// Holds information about each step.
struct StepMetadata {
    description: &'static str,
    id: &'static str,
    filename: &'static str,
    variant: Variant,
    rate: Option<Rate>,
}

/// Keep track of settings and export the results of each step of the decoding
/// process.
///
/// I wanted to keep track of settings and manage the results of each step of
/// the decoding process, because I want to debug every step of the decode by
/// storing the samples as a WAV file.
///
/// Meanwhile in the future maybe instead of saving the samples on WAV files I
/// can plot them on the GUI. Also I don't want clutter every function on the
/// `dsp` and `noaa_apt` modules with code for WAV export.
///
/// So every interesting function (on the `dsp` or `noaa_apt` module) should get
/// an instance of `Context`, and send to it results of each step. Then the
/// `Context` will save them as WAV or do nothing depending on the user's
/// settings.
///
/// Also the `Context` has information (`StepMetadata`) about each Step: like a
/// description and filename to use when saving to disk.
///
/// One problem I had is that the Context needs the sample rates of every signal
/// for correct WAV export, so the `Rate` is given when calling
/// `Context.step()`.
/// Some functions on module `dsp` don't know the sample rate of the signal they
/// are working on and pass `None` instead of a valid `Rate` so those steps
/// have in their metadata the sample rate.
pub struct Context {
    steps_metadata: Vec<StepMetadata>,

    /// If we are exporting something, functions like `noaa_apt::find_sync()`
    /// check this to decide if they should do things fast or they should do
    /// extra work and save intermediate signals. Anyways, for now it's always
    /// the same as `export_wav` because there is no implementation to export to
    /// something else.
    pub export_steps: bool,

    /// If we are exporting the filtered signal on resample. When using
    /// `fast_resampling()` this step is VERY slow and RAM heavy (gigabytes!),
    /// so that function checks if this variable is set before doing extra work.
    pub export_resample_filtered: bool,

    /// Private field, if we are exporting to WAV.
    export_wav: bool,

    /// Current step index.
    index: usize,
}

impl Context {

    /// Export step.
    pub fn step(&mut self, step: Step) -> err::Result<()> {
        if self.export_wav {

            debug!("Got step: {}", step.id);

            // Metadata about the step we expect to receive
            let metadata = match self.steps_metadata.get(self.index) {
                Some(e) => e,
                None => {
                    debug!("Ignoring step \"{}\", no more steps expected", step.id);
                    return Ok(())
                },
            };

            // We got an unexpected step, we should ignore it because sometimes
            // the step we want comes after
            if step.id != metadata.id {
                debug!("Ignoring step \"{}\", expecting \"{}\"", step.id, metadata.id);
                return Ok(())
            } else {
                self.index += 1;
            }

            if ! self.export_resample_filtered && step.id == "resample_filtered" {
                debug!("Ignoring step \"resample_filtered\", disabled by options");
                return Ok(())
            }

            if step.variant != metadata.variant {
                return Err(err::Error::Internal(format!(
                    "Expected variant {:?}, got {:?}", metadata.variant, step.variant)));
            }

            // Happens if syncing is disabled and the correlation step is sent.
            if step.signal.is_empty() {
                return Ok(())
            }

            match step.variant {
                Variant::Filter => {

                    let writer_spec = hound::WavSpec {
                        channels: 1,
                        sample_rate: 1, // Could be anything
                        bits_per_sample: 32,
                        sample_format: hound::SampleFormat::Float,
                    };

                    let mut filename = metadata.filename.to_string();
                    filename.push_str(".wav");

                    wav::write_wav(filename.as_str(), &step.signal, writer_spec)?;
                },
                Variant::Signal => {

                    let unpacked_rate = match step.rate.or(metadata.rate) {
                        Some(r) => r,
                        None => return Err(err::Error::Internal(format!(
                            "Unknown rate for step \"{}\"", step.id))),
                    };

                    let writer_spec = hound::WavSpec {
                        channels: 1,
                        sample_rate: unpacked_rate.get_hz(),
                        bits_per_sample: 32,
                        sample_format: hound::SampleFormat::Float,
                    };

                    let mut filename = metadata.filename.to_string();
                    filename.push_str(".wav");

                    wav::write_wav(filename.as_str(), &step.signal, writer_spec)?;
                },
            };
        }

        Ok(())
    }

    /// Create `Context` for a resampling process.
    pub fn resample(
        export_wav: bool,
        export_resample_filtered: bool
    ) -> Self {

        Self {
            steps_metadata: vec![
                StepMetadata {
                    description: "Samples read from WAV",
                    id: "input",
                    filename: "00_input",
                    variant: Variant::Signal,
                    rate: None,
                },
                StepMetadata {
                    description: "Filter used on resample",
                    id: "resample_filter",
                    filename: "01_resample_filter",
                    variant: Variant::Filter,
                    rate: None,
                },
                StepMetadata {
                    description: "Expanded and filtered signal",
                    id: "resample_filtered",
                    filename: "02_resample_filtered",
                    variant: Variant::Signal,
                    rate: None,
                },
                StepMetadata {
                    description: "Result of resample",
                    id: "resample_decimated",
                    filename: "03_resample_result",
                    variant: Variant::Signal,
                    rate: None,
                }
            ],
            export_steps: export_wav,
            export_resample_filtered,
            export_wav,
            index: 0,
        }
    }

    /// Create `Context` for a decoding process.
    pub fn decode(
        work_rate: Rate,
        final_rate: Rate,
        export_wav: bool,
        export_resample_filtered: bool
    ) -> Self {

        Self {
            steps_metadata: vec![
                StepMetadata {
                    description: "Samples read from WAV",
                    id: "input",
                    filename: "00_input",
                    variant: Variant::Signal,
                    rate: None,
                },
                StepMetadata {
                    description: "Filter used on first resample",
                    id: "resample_filter",
                    filename: "01_resample_filter",
                    variant: Variant::Filter,
                    rate: None,
                },
                StepMetadata {
                    description: "Expanded and filtered on first resample",
                    id: "resample_filtered",
                    filename: "02_resample_filtered",
                    variant: Variant::Signal,
                    rate: None,
                },
                StepMetadata {
                    description: "Result of first resample",
                    id: "resample_decimated",
                    filename: "03_resample_decimated",
                    variant: Variant::Signal,
                    rate: None,
                },
                StepMetadata {
                    description: "Raw demodulated signal",
                    id: "demodulation_result",
                    filename: "04_demodulated_unfiltered",
                    variant: Variant::Signal,
                    rate: Some(work_rate),
                },
                StepMetadata {
                    description: "Filter for demodulated signal",
                    id: "filter_filter",
                    filename: "05_demodulation_filter",
                    variant: Variant::Filter,
                    rate: None,
                },
                StepMetadata {
                    description: "Filtered demodulated signal",
                    id: "filter_result",
                    filename: "06_demodulated",
                    variant: Variant::Signal,
                    rate: Some(work_rate),
                },
                StepMetadata {
                    description: "Cross correlation used in syncing",
                    id: "sync_correlation",
                    filename: "07_sync_correlation",
                    variant: Variant::Signal,
                    rate: Some(work_rate),
                },
                StepMetadata {
                    description: "Synced signal",
                    id: "sync_result",
                    filename: "08_synced",
                    variant: Variant::Signal,
                    rate: None,
                },
                StepMetadata {
                    description: "Filter used on second resample",
                    id: "resample_filter",
                    filename: "09_resample_filter",
                    variant: Variant::Filter,
                    rate: None,
                },
                StepMetadata {
                    description: "Expanded and filtered on second resample",
                    id: "resample_filtered",
                    filename: "10_resample_filtered",
                    variant: Variant::Signal,
                    rate: Some(final_rate),
                },
                StepMetadata {
                    description: "Result of second resample",
                    id: "resample_decimated",
                    filename: "11_resample_decimated",
                    variant: Variant::Signal,
                    rate: Some(final_rate),
                },
                StepMetadata {
                    description: "Telemetry A horizontal averages",
                    id: "telemetry_a",
                    filename: "12_telemetry_a",
                    variant: Variant::Signal,
                    rate: Some(final_rate / PX_PER_ROW),
                },
                StepMetadata {
                    description: "Telemetry B horizontal averages",
                    id: "telemetry_b",
                    filename: "13_telemetry_b",
                    variant: Variant::Signal,
                    rate: Some(final_rate / PX_PER_ROW),
                },
                StepMetadata {
                    description: "Correlation of telemetry with sample",
                    id: "telemetry_correlation",
                    filename: "14_telemetry_correlation",
                    variant: Variant::Signal,
                    rate: Some(final_rate / PX_PER_ROW),
                },
                StepMetadata {
                    description: "Horizontal variance of telemetry bands",
                    id: "telemetry_variance",
                    filename: "15_telemetry_variance",
                    variant: Variant::Signal,
                    rate: Some(final_rate / PX_PER_ROW),
                },
                StepMetadata {
                    description: "Telemetry quality estimation",
                    id: "telemetry_quality",
                    filename: "16_telemetry_quality",
                    variant: Variant::Signal,
                    rate: Some(final_rate / PX_PER_ROW),
                },
                StepMetadata {
                    description: "Result of signal mapping, contrast check",
                    id: "mapped",
                    filename: "17_mapped",
                    variant: Variant::Signal,
                    rate: None,
                },
            ],
            export_steps: export_wav,
            export_resample_filtered,
            export_wav,
            index: 0,
        }
    }
}
