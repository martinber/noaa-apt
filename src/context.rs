//! I wanted to keep track of some settings and manage the results of each step
//! of the decoding process, because I want to debug every step of the decode,
//! by storing the samples as a WAV file.
//!
//! Meanwhile in the future maybe instead of saving the samples on WAV files I
//! can plot them on the GUI. Also I don't want to add code on every function
//! on the dsp module.
//!
//! So every interesting function (on the dsp or noaa_apt module) should get an
//! instance of Context, and send to it results of each step. Then the Context
//! will save them as WAV or do nothing depending on the user's settings.
//!
//! Also the Context has information (Metadata) about each Step like a
//! description and filename to use when saving to disk.
//!
//! One problem I had is that some functions on module dsp don't know the
//! sample rate of the signal they are working on. The Context needs the sample
//! rates of every signal for correct WAV export. So some steps have in their
//! metadata the sample rate.

use dsp::{Signal, Rate};
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
/// Some steps can happen twice or can not happen, the Context has information
/// about the order in which the steps can ocurr.
///
/// The Rate is optional because some functions on the dsp module don't know the
/// sample rate. In those cases, the metadata should have it. Also, when saving
/// filters, there is no rate.
///
/// The references only need to be valid until calling Context::step().
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
    description: String,
    id: String,
    filename: String,
    variant: Variant,
    rate: Option<Rate>,
}

/// Keep track of some settings and manage the results of each step of the
/// decoding process.
///
/// So every interesting function should get an instance of Context, and give it
/// results of each steps. Then the Context will save them as WAV or do nothing
/// depending on the user's settings.
///
/// Has a list of StepMetadata, with information about each step we expect to
/// get when someone calls our step() function.
pub struct Context {
    /// Information about each step we expect to receive.
    steps_metadata: Vec<StepMetadata>,

    /// If we are exporting something, functions like noaa_apt::find_sync()
    /// check this to decide if they should do things fast or they should do
    /// extra work and save intermediate signals.
    pub export: bool,

    /// If we are exporting the filtered signal on resample. When using
    /// fast_resampling() this step es VERY slow and RAM heavy (gigabytes!), so
    /// that function checks if this variable is set before doing extra work.
    pub export_resample_filtered: bool,

    /// Private field, if we are exporting to WAV.
    export_wav: bool,

    /// Current step index
    index: usize,
}

impl Context {

    /// Store information about one step.
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

            match step.variant {
                Variant::Filter => {

                    let writer_spec = hound::WavSpec {
                        channels: 1,
                        sample_rate: 1,
                        bits_per_sample: 32,
                        sample_format: hound::SampleFormat::Float,
                    };

                    let mut filename = metadata.filename.clone();
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

                    let mut filename = String::new();
                    filename.push_str(metadata.filename.as_str());
                    filename.push_str(".wav");

                    wav::write_wav(filename.as_str(), &step.signal, writer_spec)?;
                },
            };
        }

        Ok(())
    }

    pub fn resample(
        export_wav: bool,
        export_resample_filtered: bool
    ) -> Context {

        Context {
            steps_metadata: vec![
                StepMetadata {
                    description: "Samples read from WAV".to_string(),
                    id: "input".to_string(),
                    filename: "00_input".to_string(),
                    variant: Variant::Signal,
                    rate: None,
                },
                StepMetadata {
                    description: "Filter used on resample".to_string(),
                    id: "resample_filter".to_string(),
                    filename: "01_resample_filter".to_string(),
                    variant: Variant::Filter,
                    rate: None,
                },
                StepMetadata {
                    description: "Expanded and filtered signal".to_string(),
                    id: "resample_filtered".to_string(),
                    filename: "02_resample_filtered".to_string(),
                    variant: Variant::Signal,
                    rate: None,
                },
                StepMetadata {
                    description: "Result of resample".to_string(),
                    id: "resample_decimated".to_string(),
                    filename: "03_resample_result".to_string(),
                    variant: Variant::Signal,
                    rate: None,
                }
            ],
            export: export_wav,
            export_resample_filtered,
            export_wav,
            index: 0,
        }
    }

    pub fn decode(
        work_rate: Rate,
        final_rate: Rate,
        export_wav: bool,
        export_resample_filtered: bool
    ) -> Context {

        Context {
            steps_metadata: vec![
                StepMetadata {
                    description: "Samples read from WAV".to_string(),
                    id: "input".to_string(),
                    filename: "00_input".to_string(),
                    variant: Variant::Signal,
                    rate: None,
                },
                StepMetadata {
                    description: "Filter used on first resample".to_string(),
                    id: "resample_filter".to_string(),
                    filename: "01_resample_filter".to_string(),
                    variant: Variant::Filter,
                    rate: None,
                },
                StepMetadata {
                    description: "Expanded and filtered on first resample".to_string(),
                    id: "resample_filtered".to_string(),
                    filename: "02_resample_filtered".to_string(),
                    variant: Variant::Signal,
                    rate: None,
                },
                StepMetadata {
                    description: "Result of first resample".to_string(),
                    id: "resample_decimated".to_string(),
                    filename: "03_resample_decimated".to_string(),
                    variant: Variant::Signal,
                    rate: None,
                },
                StepMetadata {
                    description: "Raw demodulated signal".to_string(),
                    id: "demodulation_result".to_string(),
                    filename: "04_demodulated_unfiltered".to_string(),
                    variant: Variant::Signal,
                    rate: Some(work_rate),
                },
                StepMetadata {
                    description: "Filter for demodulated signal".to_string(),
                    id: "filter_filter".to_string(),
                    filename: "05_demodulation_filter".to_string(),
                    variant: Variant::Filter,
                    rate: None,
                },
                StepMetadata {
                    description: "Filtered demodulated signal".to_string(),
                    id: "filter_result".to_string(),
                    filename: "06_demodulated".to_string(),
                    variant: Variant::Signal,
                    rate: Some(work_rate),
                },
                StepMetadata {
                    description: "Cross correlation used in syncing".to_string(),
                    id: "sync_correlation".to_string(),
                    filename: "07_sync_correlation".to_string(),
                    variant: Variant::Signal,
                    rate: Some(work_rate),
                },
                StepMetadata {
                    description: "Synced signal".to_string(),
                    id: "sync_result".to_string(),
                    filename: "08_synced".to_string(),
                    variant: Variant::Signal,
                    rate: None,
                },
                StepMetadata {
                    description: "Filter used on second resample".to_string(),
                    id: "resample_filter".to_string(),
                    filename: "09_resample_filter".to_string(),
                    variant: Variant::Filter,
                    rate: None,
                },
                StepMetadata {
                    description: "Expanded and filtered on second resample".to_string(),
                    id: "resample_filtered".to_string(),
                    filename: "10_resample_filtered".to_string(),
                    variant: Variant::Signal,
                    rate: Some(final_rate),
                },
                StepMetadata {
                    description: "Result of second resample".to_string(),
                    id: "resample_decimated".to_string(),
                    filename: "11_resample_decimated".to_string(),
                    variant: Variant::Signal,
                    rate: Some(final_rate),
                },
                StepMetadata {
                    description: "Result of signal mapping, contrast check".to_string(),
                    id: "mapped".to_string(),
                    filename: "12_mapped".to_string(),
                    variant: Variant::Signal,
                    rate: None,
                },
            ],
            export: export_wav,
            export_resample_filtered,
            export_wav,
            index: 0,
        }
    }
}
