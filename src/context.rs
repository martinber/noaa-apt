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

/// Represents a step on the decoding process.
///
/// Enum of every kind of step, some steps can happen twice or can not happen,
/// the Context has information about the order in which the steps can ocurr.
///
/// The Rate is optional because some functions on the dsp module don't know the
/// sample rate. In those cases, the metadata should have it.
#[derive(Debug)]
pub enum Step<'a> {
    Signal(&'a Signal, Option<Rate>),
    Filter(&'a Signal),
}

/// Holds information about each step.
struct StepMetadata {
    description: String,
    filename: String,
    variant: String,
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
    steps_metadata: std::vec::IntoIter<StepMetadata>,
    export_wav: bool,
}

impl Context {

    /// Store information about one step.
    pub fn step(&mut self, step: Step) -> err::Result<()> {
        if self.export_wav {
            // Metadata about the step we expect to receive
            let metadata = match self.steps_metadata.next() {
                Some(e) => e,
                None => return Err(err::Error::Internal(
                    "Got too many steps".to_string())),
            };

            debug!("Got step: {}", metadata.description);

            match step {
                Step::Filter(signal) => {
                    if metadata.variant != "filter" {
                        return Err(err::Error::Internal(format!(
                            "Expected step {}, got {:?}", metadata.description, step)));
                    }

                    let writer_spec = hound::WavSpec {
                        channels: 1,
                        sample_rate: 1,
                        bits_per_sample: 32,
                        sample_format: hound::SampleFormat::Float,
                    };

                    debug!("Writing WAV to '{}'", metadata.filename);

                    let mut filename = metadata.filename.clone();
                    filename.push_str(".wav");

                    wav::write_wav(filename.as_str(), &signal, writer_spec)?;
                },
                Step::Signal(signal, rate) => {
                    if metadata.variant != "signal" {
                        return Err(err::Error::Internal(format!(
                            "Expected step \"{}\", got {:?}", metadata.description, step)));
                    }

                    let unpacked_rate = match rate.or(metadata.rate) {
                        Some(r) => r,
                        None => return Err(err::Error::Internal(format!(
                            "Unknown rate for step \"{}\"", metadata.description))),
                    };

                    let writer_spec = hound::WavSpec {
                        channels: 1,
                        sample_rate: unpacked_rate.get_hz(),
                        bits_per_sample: 32,
                        sample_format: hound::SampleFormat::Float,
                    };

                    debug!("Writing WAV to '{}'", metadata.filename);

                    let mut filename = metadata.filename.clone();
                    filename.push_str(".wav");

                    wav::write_wav(filename.as_str(), &signal, writer_spec)?;
                },
            };
        }

        Ok(())
    }

    pub fn resample(export_wav: bool) -> Context {
        Context {
            steps_metadata: vec![
                StepMetadata {
                    description: "Samples read from WAV".to_string(),
                    filename: "0_input".to_string(),
                    variant: "signal".to_string(),
                    rate: None,
                },
                StepMetadata {
                    description: "Filter used on resample".to_string(),
                    filename: "1_resample_filter".to_string(),
                    variant: "filter".to_string(),
                    rate: None,
                },
                StepMetadata {
                    description: "Result of resample".to_string(),
                    filename: "2_resampled".to_string(),
                    variant: "signal".to_string(),
                    rate: None,
                }
            ].into_iter(),
            export_wav,
        }
    }

    pub fn decode(work_rate: Rate, final_rate: Rate, export_wav: bool) -> Context {
        Context {
            steps_metadata: vec![
                StepMetadata {
                    description: "Samples read from WAV".to_string(),
                    filename: "0_input".to_string(),
                    variant: "signal".to_string(),
                    rate: None,
                },
                StepMetadata {
                    description: "Filter used on first resample".to_string(),
                    filename: "1_resample_filter".to_string(),
                    variant: "filter".to_string(),
                    rate: None,
                },
                StepMetadata {
                    description: "Result of first resample".to_string(),
                    filename: "2_resampled".to_string(),
                    variant: "signal".to_string(),
                    rate: None,
                },
                StepMetadata {
                    description: "Raw demodulated signal".to_string(),
                    filename: "3_demodulated_unfiltered".to_string(),
                    variant: "signal".to_string(),
                    rate: Some(work_rate),
                },
                StepMetadata {
                    description: "Filter for demodulated signal".to_string(),
                    filename: "4_demodulation_filter".to_string(),
                    variant: "filter".to_string(),
                    rate: None,
                },
                StepMetadata {
                    description: "Filtered demodulated signal".to_string(),
                    filename: "5_demodulated".to_string(),
                    variant: "signal".to_string(),
                    rate: Some(work_rate),
                },
                StepMetadata {
                    description: "Synced signal".to_string(),
                    filename: "6_synced".to_string(),
                    variant: "signal".to_string(),
                    rate: None,
                },
                StepMetadata {
                    description: "Filter used on second resample".to_string(),
                    filename: "7_resample_filter".to_string(),
                    variant: "filter".to_string(),
                    rate: None,
                },
                StepMetadata {
                    description: "Result of second resample".to_string(),
                    filename: "8_resampled".to_string(),
                    variant: "signal".to_string(),
                    rate: Some(final_rate),
                },
                StepMetadata {
                    description: "Result of signal mapping, contrast check".to_string(),
                    filename: "9_mapped".to_string(),
                    variant: "signal".to_string(),
                    rate: None,
                },
            ].into_iter(),
            export_wav,
        }
    }
}
