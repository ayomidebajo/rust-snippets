use anyhow::Ok as Okr;
use clap::builder::OsStr;
use clap::{arg, Arg, Command};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use tokio::stream;
use std::fs::File;
use std::io;
use std::io::{BufWriter, Write, Read};
use std::result::Result::Ok;
use std::sync::{Arc, Mutex};
use std::net::TcpStream;
use tokio::time::{sleep, Duration};

use tokio_tungstenite::client_async;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
// type AnyError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
struct Opt {
    #[cfg(all(
        any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
        feature = "jack"
    ))]
    jack: bool,

    device: String,
}

impl Opt {
    fn from_args() -> Self {
        // let app = clap::Command::new("record_wav").arg(arg!([DEVICE] "The audio device to use"));
        // let app = Command::new("record_wav")
        //     .about("stuff")
        //     .arg(
        //         Arg::new("DEVICE")
        //             .help("The audio device to use")
        //             .required(true),
        //     )
        //     .get_matches();
        #[cfg(all(
            any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
            feature = "jack"
        ))]
        let app = app.arg(arg!(-j --jack "Use the JACK host"));

        // println!("know me {:?}", app);

        let host = cpal::default_host();

        // let device = host
        //     .default_input_device()
        //     .expect("no input device available");

        // Ayo's solution type=fast😂
        let dev = String::from("USB PnP Sound Device");

        // Ayo's second solution bypassing clap issues
        // let plainarg = app.clone();

        // let apps = plainarg.get_raw("DEVICE").unwrap().enumerate();

        // let mut original = String::from("");
        // let handle = for (_, j) in apps.into_iter() {
        //     original = format!("{:?}", j)
        // };
        // original.retain(|c| c != '"');

        #[cfg(all(
            any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
            feature = "jack"
        ))]
        return Opt {
            jack: matches.is_present("jack"),
            device,
        };

        #[cfg(any(
            not(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd")),
            not(feature = "jack")
        ))]
        // Ayo's solution type=fast
        Opt { device: dev }

        // Ayo's solution bypassing the clap issues
        // Opt {
        //     device: original.trim().to_string(),
        // }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let addr = "127.0.0.1:9000";
    let url = format!("ws://{}", addr);
    let stream = TcpStream::connect(addr).unwrap();
    let opt = Opt::from_args();

    // Conditionally compile with jack if the feature is specified.
    #[cfg(all(
        any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
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
        not(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd")),
        not(feature = "jack")
    ))]
    let host = cpal::default_host();

    // Set up the input device and stream with the default input config.
    let device = if opt.device == "default" {
        host.default_input_device()
    } else {
        host.input_devices()?
            .find(|x| x.name().map(|y| y == opt.device).unwrap_or(false))
    }
    .expect("failed to find input device");

    println!("Input device: {}", device.name()?);

    let config = device
        .default_input_config()
        .expect("Failed to get default input config");
    println!("Default input config: {:?}", config);

    // The WAV file we're recording to.
    const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded.wav");
    let spec = wav_spec_from_config(&config);
    let writer = BufWriter::with_capacity(100, stream);
    let writer = Arc::new(Mutex::new(Some(writer)));

    // A flag to indicate that recording is in progress.
    println!("Begin recording...");

    // Run the input stream on a separate thread.
    let writer_2 = writer.clone();

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<f32, f32>(data, &writer_2),
            err_fn,
        )?,
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i16, i16>(data, &writer_2),
            err_fn,
        )?,
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<u16, i16>(data, &writer_2),
            err_fn,
        )?,
    };

    stream.play()?;



    // Let recording go for roughly three seconds.
    std::thread::sleep(std::time::Duration::from_secs(10));

    writer.lock().unwrap().take().unwrap();

    println!("Connected to {:?}", addr);

    let (mut ws_stream, _) = client_async(&url, stream).await?;

    println!("Handshake successful.");

    //    let strem =  mainstream();

    //  if strem.expect("Error play").play()? {
    if let Some(stream) = ws_stream {
        match item {
            Ok(msg) => {
                match msg {
                    Message::Binary(text) => {
                        println!("Received text message: {:?}", text);
                    }
                    Message::Close(frame) => {
                        println!("Received close message: {:?}", frame);

                        if let Err(e) = ws_stream.close(None).await {
                            match e {
                                WsError::ConnectionClosed => (),
                                _ => {
                                    println!("Error while closing: {}", e);
                                    // break;
                                }
                            }
                        }

                        // println!("Sent close message.");

                        println!("Closing...");
                        return Ok(());
                    }
                    _ => (),
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: \n{0:?}\n{0}", e);
            }
        }
        // }
    }

    // println!("Recording {} complete!", PATH);

    println!("Sent close message.");

    println!("Closing...");
    // Ok(())
    Ok(stream);

    ws_stream.close(None).await?;

    println!("Sent close message.");

    println!("Closing...");
    Ok(())
}

// fn mainstream() -> Result<Stream, anyhow::Error> {
//     let opt = Opt::from_args();

//     // Conditionally compile with jack if the feature is specified.
//     #[cfg(all(
//         any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
//         feature = "jack"
//     ))]
//     // Manually check for flags. Can be passed through cargo with -- e.g.
//     // cargo run --release --example beep --features jack -- --jack
//     let host = if opt.jack {
//         cpal::host_from_id(cpal::available_hosts()
//             .into_iter()
//             .find(|id| *id == cpal::HostId::Jack)
//             .expect(
//                 "make sure --features jack is specified. only works on OSes where jack is available",
//             )).expect("jack host unavailable")
//     } else {
//         cpal::default_host()
//     };

//     #[cfg(any(
//         not(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd")),
//         not(feature = "jack")
//     ))]
//     let host = cpal::default_host();

//     // Set up the input device and stream with the default input config.
//     let device = if opt.device == "default" {
//         host.default_input_device()
//     } else {
//         host.input_devices()?
//             .find(|x| x.name().map(|y| y == opt.device).unwrap_or(false))
//     }
//     .expect("failed to find input device");

//     println!("Input device: {}", device.name()?);

//     let config = device
//         .default_input_config()
//         .expect("Failed to get default input config");
//     println!("Default input config: {:?}", config);

//     // The WAV file we're recording to.
//     const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded.wav");
//     let spec = wav_spec_from_config(&config);
//     let writer = BufWriter::with_capacity(100, i);
//     let writer = Arc::new(Mutex::new(Some(writer)));

//     // A flag to indicate that recording is in progress.
//     println!("Begin recording...");

//     // Run the input stream on a separate thread.
//     let writer_2 = writer.clone();

//     let err_fn = move |err| {
//         eprintln!("an error occurred on stream: {}", err);
//     };

//     let stream = match config.sample_format() {
//         cpal::SampleFormat::F32 => device.build_input_stream(
//             &config.into(),
//             move |data, _: &_| write_input_data::<f32, f32>(data, &writer_2),
//             err_fn,
//         )?,
//         cpal::SampleFormat::I16 => device.build_input_stream(
//             &config.into(),
//             move |data, _: &_| write_input_data::<i16, i16>(data, &writer_2),
//             err_fn,
//         )?,
//         cpal::SampleFormat::U16 => device.build_input_stream(
//             &config.into(),
//             move |data, _: &_| write_input_data::<u16, i16>(data, &writer_2),
//             err_fn,
//         )?,
//     };

// //    client(stream);

//     stream.play()?;

//     // Let recording go for roughly three seconds.
//     std::thread::sleep(std::time::Duration::from_secs(10));

//     //    ws_stream.send(stream);

//     // drop(stream);
//     writer.lock().unwrap().take().unwrap();
//     println!("Recording {} complete!", PATH);

//     println!("Sent close message.");

//     println!("Closing...");
//     // Ok(())
//     Ok(stream)
//         // drop(stream);
// }

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    match format {
        cpal::SampleFormat::U16 => hound::SampleFormat::Int,
        cpal::SampleFormat::I16 => hound::SampleFormat::Int,
        cpal::SampleFormat::F32 => hound::SampleFormat::Float,
    }
}

fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}

type WavWriterHandle = Arc<Mutex<Option<BufWriter<TcpStream>>>>;

fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: cpal::Sample,
    U: cpal::Sample + hound::Sample,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = cpal::Sample::from(&sample);
                
                writer.write_sample(sample).ok();
            }
        }
    }
}
