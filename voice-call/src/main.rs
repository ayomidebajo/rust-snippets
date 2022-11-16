use anyhow::Ok;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Data, Sample, SampleFormat, SupportedStreamConfig};
use std::sync::mpsc::Sender;
use std::sync::{mpsc::channel, Arc, Mutex};
use tokio_stream::StreamExt;
// use tokio::sync::watch;
use tokio::time::sleep;
use ringbuf::HeapRb;

use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

use tokio_tungstenite::{client_async, WebSocketStream};
use tokio_tungstenite::tungstenite::{Message, Error as WsError};
// type StateHandle = Arc<Mutex<Option<Vec<f32>>>>;

fn main() -> Result<(), AnyError> {
    let addr = "127.0.0.1:9000";
    let url = format!("ws://{}", addr);

    let stream = TcpStream::connect(addr).await?;

    println!("Connected to {:?}", addr);

    let (mut ws_stream, response) = client_async(&url, stream).await?;

    println!("Handshake successful.");
    // let (sender, receiver) = tokio::sync::watch::channel(Vec::<u16>::new());

    // let (sender_output, receiver_output) = tokio::sync::watch::channel(Vec::<f32>::new());

    let host = cpal::default_host();
    let device_input = host.default_input_device().expect("no input devices found");

    let device_output = host
        .default_output_device()
        .expect("no output devices found");

    let mut supported_configs_range = device_output
        .supported_output_configs()
        .expect("error while querying configs");

    let supported_config = supported_configs_range
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();

    let mut supported_configs_range_output = device_output
        .supported_output_configs()
        .expect("error while querying configs");

    let supported_config_output = supported_configs_range_output
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();

    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let sample_format = supported_config.sample_format();
    let config: SupportedStreamConfig = supported_config.into();

    let sample_format_output = supported_config_output.sample_format();
    let config_output: SupportedStreamConfig = supported_config_output.into();

    // let stream_ouput = device_output.build_output_stream(
    //     &config,
    //     move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
    //         // react to stream events and read or write stream data here.
    //     },
    //     move |err| {
    //         // react to errors here.
    //         panic!("panice")
    //     },
    // );

    // let channels = config.channels();
    // println!("number of channels {}", channels);
    let stream = match sample_format {
        cpal::SampleFormat::F32 => device_input.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<f32>(data, 1.0, &ws_stream),
            err_fn,
        ),
        _ => panic!("Unsupported"),
    }
    .unwrap();
    let mut counter = 0;
    stream.play().unwrap();

    // let state = Arc::new(Mutex::new(Some(Vec::new())));

    let stream_play = match sample_format_output {
        cpal::SampleFormat::F32 => device_output
            .build_output_stream(
                &config_output.into(),
                move |data, _: &_| write_output_data::<f32>(data, 1, &response),
                err_fn,
            )
            .map_err(|_e| {
                println!("U16 error? {}", _e);
                // tide::http::Error::from_str(tide::StatusCode::BadRequest, "Error happened")
            })
            .unwrap(),
        _ => panic!("Unsupported"),
    };

    stream_play.play().unwrap();

    // Let recording go for roughly ten seconds.
    // std::thread::sleep(std::time::Duration::from_secs(10));
    while counter < 10 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        counter += 1
    }
    drop(stream);
    drop(stream_play);
    // writer.lock().unwrap().take().unwrap().finalize()?;
    // println!("Recording {} complete!", PATH);

    pub fn write_input_data<T>(
        input: &[T],
        channels: f32,
        sender: WebSocketStream<f32>,
    ) where
        T: cpal::Sample,
    {
        let mut samples = vec![];
        for frame in input.chunks(channels.into()) {
            // println!("loop");
            samples.push(frame[0].to_u16());
        }
        println!("samples {:?}", &samples);
        // send samples to the thread that sends it to client
        sender.send(samples.clone()).unwrap();
    }
    // #[derive(Debug)]
    // struct SomeShit {
    //     shit: dyn Sample,
    // }

    pub fn write_output_data<T>(
        output: &[T],
        channels: f32,
        writer: WebSocketStream<f32>
    ) where
        T: cpal::Sample,
    {
        let mut samples = vec![];

        for frame in  writer {
            samples.push(*frame[0])
        }

       
    }
    Ok("yh")
}

// Create player thread.

// differentiate between player thread and reciever thread
