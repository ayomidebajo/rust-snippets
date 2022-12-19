//! Feeds back the input stream directly into the output stream.
//!
//! Assumes that the input and output devices can use the same stream configuration and that they
//! support the f32 sample format.
//!
//! Uses a delay of `LATENCY_MS` milliseconds in case the default input and output streams are not
//! precisely synchronised.

use regex::Regex;
use std::future::Future;
use std::time;
// use std::time::Duration;
use futures::sink::SinkExt;
use futures::stream::StreamExt;

use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

use tokio_tungstenite::client_async;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

// use clap::Parser;
use anyhow::{self, Ok};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::HeapRb;
use std::result::Result::Ok as Okr;

#[derive(Debug)]
// #[command(version, about = "CPAL feedback example", long_about = None)]
struct Opt {
    /// The input audio device to use
    // #[arg(short, long, value_name = "IN", default_value_t = String::from("default"))]
    input_device: String,

    // /// The output audio device to use
    // #[arg(short, long, value_name = "OUT", default_value_t = String::from("default"))]
    output_device: String,

    /// Specify the delay between input and output
    // #[arg(short, long, value_name = "DELAY_MS", default_value_t = 150.0)]
    latency: f32,

    /// Use the JACK host
    #[cfg(all(
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd"
        ),
        feature = "jack"
    ))]
    #[arg(short, long)]
    #[allow(dead_code)]
    jack: bool,
}

// for parsing duration
fn parse_duration(duration: &str) -> time::Duration {
    let re = Regex::new(r"((?P<hour>\d+)h)?((?P<minute>\d+)m)?((?P<second>\d+)s)?").unwrap();
    let caps = re.captures(duration).unwrap();
    let h: u64 = caps.name("hour").map_or(0, |m| m.as_str().parse().unwrap());
    let m: u64 = caps
        .name("minute")
        .map_or(0, |m| m.as_str().parse().unwrap());
    let s: u64 = caps
        .name("second")
        .map_or(0, |m| m.as_str().parse().unwrap());
    time::Duration::new(3600 * h + 60 * m + s, 0)
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt {
        input_device: String::from("MacBook Pro Microphone"),
        output_device: String::from("MacBook Pro Speakers"),
        latency: 150.0,
    };

    // Conditionally compile with jack if the feature is specified.
    #[cfg(all(
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd"
        ),
        feature = "jack"
    ))]
    // Manually check for flags. Can be passed through cargo with -- e.g.
    // cargo run --release --example beep --features jack -- --jack
    let host = if opt.jack {
        cpal::host_from_id(cpal::available_hosts()
            .into_iter()
            .find(|id| *id == cpal::HostId::Jack)
            .expect(
                "make sure --features jack is specified. only works on OSes where jack is available",
            )).expect("jack host unavailable")
    } else {
        cpal::default_host()
    };

    #[cfg(any(
        not(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd"
        )),
        not(feature = "jack")
    ))]
    let host = cpal::default_host();

    // Find devices.
    // let input_device = if opt.input_device == "default" {
    //     host.default_input_device()
    // } else {
    //     host.input_devices()?
    //         .find(|x| x.name().map(|y| y == opt.input_device).unwrap_or(false))
    // }
    let (send_audio, recv_audio) = tokio::sync::watch::channel(Vec::new());

    let audio_thread = tokio::spawn(async move {
        let pa = cpal::default_host();;

        println!("PortAudio:");
        // println!("version: {}", pa.);
        // println!("version text: {:?}", pa.devices());
        println!("device count: {}", pa.devices().unwrap());

        // let default_host = pa.default_host_api().unwrap();
        // println!("default host: {:#?}", pa.host_api_info(default_host));

        let def_input = pa.default_input_device().unwrap();
        // let input_info = pa.device_info(def_input).unwrap();
        // println!("Default input device info: {:#?}", &input_info);

        // Construct the input stream parameters.
        let latency = input_info.default_low_input_latency;
        println!("how far?");

        let input_params =
            pa::StreamParameters::<u8>::new(def_input, CHANNELS, INTERLEAVED, latency);
        // Check that the stream format is supported.
        pa.is_input_format_supported(input_params, SAMPLE_RATE)
            .unwrap();
        println!("how far??");
        // Construct the settings with which we'll open our input stream.
        let settings = pa::InputStreamSettings::new(input_params, SAMPLE_RATE, FRAMES);
        println!("how far???");

        // Keep track of the last `current_time` so we can calculate the delta time.
        let mut maybe_last_time = None;
        println!("how goes?");

        // We'll use this channel to send the count_down to the main thread for fun.
        let (sender, receiver) = ::std::sync::mpsc::channel();
        println!("how goess??");

        // A callback to pass to the non-blocking stream.
        let callback = move |pa::InputStreamCallbackArgs {
                                 buffer,
                                 frames,
                                 time,
                                 ..
                             }| {
            let current_time = time.current;
            let prev_time = maybe_last_time.unwrap_or(current_time);
            let dt = current_time - prev_time;
            maybe_last_time = Some(current_time);

            assert!(frames == FRAMES as usize);
            sender.send(buffer.to_vec()).unwrap();
            println!("buffer: {:?}", buffer);
            pa::Continue
        };
        // println!("callback: {:?}", callback);

        // Construct a stream with input and output sample types of f32.
        let mut stream = pa.open_non_blocking_stream(settings, callback).unwrap();

        stream.start().unwrap();

        // Loop while the non-blocking stream is active.
        while let true = stream.is_active().unwrap() {
            // Do some stuff!
            while let Ok(data) = receiver.try_recv() {
                // println!("test: {:?}", &data);
                let test = send_audio.send(data).unwrap();
                // println!("test: {:?}", test);
                // println!("Data: {:?}", data);
            }
        }

        stream.stop().unwrap();
    });

    Ok(())
}
