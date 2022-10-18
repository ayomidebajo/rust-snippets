use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Data, StreamConfig};

// #[derive(Debug)]
fn main() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
  let mut supported_configs_range = device.supported_output_configs()
    .expect("error while querying configs");
let supported_config = supported_configs_range.next()
    .expect("no supported config?!")
    .with_max_sample_rate();
let config: StreamConfig = supported_config.into();

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            println!("stay {:?}", data)
            // react to stream events and read or write stream data here.
        },
        move |err| {
            println!("stay err{:?}", err)
            // react to errors here.
        },
    );
    // println!("Host {:#?}", config);
}