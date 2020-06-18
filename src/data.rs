use std::mem;
use std::pin::Pin;
use std::sync::{Arc, Mutex, Once};

use async_std::stream::Stream;
use async_std::task::Context;
use audrey::read::Reader;
use audrey::sample::interpolate::{Converter, Linear};
use audrey::sample::signal::{from_iter, Signal};
use futures::task::Poll;

use crate::start_time::StartTime;
use crate::SAMPLE_RATE;
use std::io::{Cursor, Seek, SeekFrom};

#[derive(Clone, Debug)]
pub struct Data {
    buffer: Arc<Mutex<Vec<i16>>>,
}

fn packets_to_wav(data: Vec<i16>) -> Cursor<Vec<u8>> {
    println!("Make WAV");
    let wavspec = hound::WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mew_data = Vec::<u8>::new();
    let mut cursor = Cursor::new(mew_data);

    let mut writer = hound::WavWriter::new(&mut cursor, wavspec).unwrap();
    let mut i16_writer = writer.get_i16_writer((data.clone().len()) as u32);

    for sample in data.clone().into_iter().step_by(1) {
        i16_writer.write_sample(sample);
    }

    i16_writer.flush().unwrap();
    drop(writer);
    cursor.seek(SeekFrom::Start(0)).unwrap();
    println!("Return WAV");
    cursor
}

/// Interpolate the wav to the sample rate used by the deepspeech model.
fn interpolate<F>(wav: F) -> Vec<i16>
where
    F: std::io::Read,
    F: std::io::Seek,
{
    println!("interpolate WAV");
    let mut reader = Reader::new(wav).unwrap();
    let description = reader.description();

    let audio_buffer: Vec<_> = if description.sample_rate() == SAMPLE_RATE {
        reader.samples().map(|s| s.unwrap()).collect()
    } else {
        let interpolator = Linear::new([0i16], [0]);
        let conv = Converter::from_hz_to_hz(
            from_iter(reader.samples::<i16>().map(|s| [s.unwrap()])),
            interpolator,
            description.sample_rate() as f64,
            SAMPLE_RATE as f64,
        );
        conv.until_exhausted().map(|v| v[0]).collect()
    };
    println!("return interpolated WAV");
    audio_buffer
}

impl Stream for Data {
    type Item = Vec<i16>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let buffer = self.buffer.lock().unwrap();
        let data = buffer.clone();

        let start_time_struct = StartTime::singleton();
        let ready = start_time_struct.get();
        if !data.is_empty() && ready {
            let audio_buffer = packets_to_wav(data.clone());
            let audio_buffer = interpolate(audio_buffer);
            Poll::Ready(Some(audio_buffer))
        } else {
            // Hack to keep alive
            Poll::Ready(Some(Vec::new()))
        }
    }
}

impl Data {
    pub(crate) fn singleton() -> Data {
        // Initialize it to a null value
        static mut DATA_SINGLETON: *const Data = 0 as *const Data;
        static DATA_ONCE: Once = Once::new();

        unsafe {
            DATA_ONCE.call_once(|| {
                // Make it
                let singleton = Data {
                    buffer: Arc::new(Mutex::new(Vec::new())),
                };

                // Put it in the heap so it can outlive this call
                DATA_SINGLETON = mem::transmute(Box::new(singleton));
            });

            // Now we give out a copy of the data that is safe to use concurrently.
            (*DATA_SINGLETON).clone()
        }
    }

    pub fn reset(&mut self) {
        let mut state = self.buffer.lock().expect("Could not lock mutex");
        let data = Vec::new();
        mem::replace(&mut *state, data.clone());
    }

    pub fn add(&mut self, buf: cpal::UnknownTypeInputBuffer) {
        let mut state = self.buffer.lock().expect("Could not lock mutex");
        match buf {
            cpal::UnknownTypeInputBuffer::I16(buffer) => {
                let mut local_state = state.clone();
                local_state.extend_from_slice(&*buffer);
                mem::replace(&mut *state, local_state.clone());
            }
            _ => panic!("Unexpected buffer type"),
        }
    }
}
