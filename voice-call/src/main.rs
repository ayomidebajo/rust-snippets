use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Data, Sample, SampleFormat};
use std::sync::mpsc::Sender;
use std::sync::{mpsc::channel, Arc, Mutex};
use tokio_stream::StreamExt;
// use tokio::sync::watch;
use tokio::time::sleep;

fn main() {
    let (sender, receiver) = tokio::sync::watch::channel(Vec::new());

    let host = cpal::default_host();
    let device_input = host.default_input_device().expect("no ouput devices found");
    let device_output = host
        .default_output_device()
        .expect("no input devices found");
    let mut supported_configs_range = device_output
        .supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();

    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let sample_format = supported_config.sample_format();
    let config = supported_config.into();

    let stream = device_output.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // react to stream events and read or write stream data here.
        },
        move |err| {
            // react to errors here.
            panic!("panice")
        },
    );

    // let channels = config.channels();
    // println!("number of channels {}", channels);
    let stream = match sample_format {
        cpal::SampleFormat::F32 => device_input.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<f32>(data, 2, &sender),
            err_fn,
        ),
        _ => panic!("Unsupported"),
    }
    .unwrap();
    let mut counter = 0;
    stream.play().unwrap();

    // Let recording go for roughly ten seconds.
    // std::thread::sleep(std::time::Duration::from_secs(10));
    while counter < 10 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        counter += 1
    }
    drop(stream);
    // writer.lock().unwrap().take().unwrap().finalize()?;
    // println!("Recording {} complete!", PATH);

    pub fn write_input_data<T>(
        input: &[T],
        channels: u16,
        sender: &tokio::sync::watch::Sender<Vec<u16>>,
    ) where
        T: cpal::Sample,
    {
        let mut samples = vec![];
        for frame in input.chunks(channels.into()) {
            println!("loop");
            samples.push(frame[0].to_u16());
        }
        println!("samples {:?}", &samples);
        // send samples to the thread that sends it to client
        sender.send(samples).unwrap();
    }
}

// Create player thread.

// differentiate between player thread and reciever thread

