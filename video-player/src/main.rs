use std::{
    collections::VecDeque,
    convert::{TryFrom, TryInto},
    path::Path,
    sync::{Arc, Mutex, Weak, RwLock},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use cpal::{
    Format, SampleFormat as CpalSampleFormat, StreamData as CpalStreamData, UnknownTypeOutputBuffer,
};
use ffmpeg::{
    codec::{
        decoder::{Audio as AudioDecoder, Decoder as FFmpegDecoder, Video as VideoDecoder},
        packet::{Packet, traits::{Mut, Ref}},
    },
    format::{context::Input, stream::Stream},
    media::Type,
    software::{
        resampling::Context as FFmpegResamplingContext,
        scaling::{self, Context as FFmpegScalingContext},
    },
    util::{
        channel_layout::{self, ChannelLayout},
        format::{
            pixel::Pixel as PixelFormat,
            sample::{Sample as SampleFormat, Type as SampleType},
        },
        frame::{Audio as AudioFrame, Video as VideoFrame},
    },
};
use flutter_engine::{texture_registry::ExternalTexture, RuntimeData};
use log::{error, warn};

use lazy_static::lazy_static;

use crate::audio::AudioStream;

const VIDEO_PACKET_QUEUE_MAX: usize = 1024;
const AUDIO_PACKET_QUEUE_MAX: usize = 512;
const AUDIO_BUFFER_MAX: usize = 50_000;

const QUEUE_FULL_SLEEP: u64 = 50;
const NO_PACKET_SLEEP: u64 = 10;
const NO_TEXTURE_SLEEP: u64 = 50;
const FRAME_TIMEOUT_MAX: u64 = 5;

lazy_static! {
    static ref FRAME_SLEEP_EPSILON: Duration = Duration::from_millis(1);
}

struct VideoState {
    input: Input,
    loop_count: u32,
    video: Arc<Mutex<VideoStreamData>>,
    audio: Arc<Mutex<AudioStreamData>>,
    time: Arc<RwLock<TimeData>>,
}

struct VideoStreamData {
    stream: StreamData,
    width: Option<u32>,
    height: Option<u32>,
    texture: Arc<ExternalTexture>,
    source_frame: Option<(VideoFrame, u32)>,
    scaled_frame: Option<(VideoFrame, u32)>,
    scaler: Option<ScalingContext>,
}

struct ScalingContext {
    context: FFmpegScalingContext,
}

unsafe impl Send for ScalingContext {}

struct AudioStreamData {
    stream: StreamData,
    output_stream: Option<(Arc<AudioStream>, OutputFormat)>,
    source_frames: VecDeque<AudioFrame>,
    resampled_frames: VecDeque<AudioFrame>,
    resampler: Option<ResamplingContext>,
    sample_buffer: Arc<Mutex<Option<SampleBuffer>>>,
}

struct ResamplingContext {
    context: FFmpegResamplingContext,
}

unsafe impl Send for ResamplingContext {}

struct OutputFormat {
    format: FFmpegSampleFormat,
    channel_layout: ChannelLayout,
    rate: u32,
}

enum SampleBuffer {
    I16 { buffer: VecDeque<i16> },
    F32 { buffer: VecDeque<f32> },
}

struct StreamData {
    stream_index: usize,
    decoder: Decoder,
    time_base: f64,
    duration: i64,
    time: Arc<RwLock<TimeData>>,
    packet_queue: VecDeque<PacketData>,
}

enum PacketData {
    Packet(Packet, u32),
    Flush,
}

enum Decoder {
    Video(VideoDecoder),
    Audio(AudioDecoder),
}

pub struct FFmpegPlayer {
    uri: String,
    texture: Arc<ExternalTexture>,
    state: Option<VideoState>,
    threads: Vec<JoinHandle<()>>,
    time: Option<Arc<RwLock<TimeData>>>,
}

pub struct InitResult {
    pub duration: i64,
    pub size: (u32, u32),
}

struct TimeData {
    start_time: Instant,
    paused: Option<Instant>,
    looping: bool,
    duration: i64,
}

impl FFmpegPlayer {
    pub fn new(uri: String, texture: Arc<ExternalTexture>) -> Self {
        Self {
            uri,
            texture,
            state: None,
            threads: Vec::new(),
        }
    }

    pub fn init(&mut self, rt: RuntimeData) -> InitResult {
        let time = Arc::new(RwLock::new(TimeData::new()));
        // First, open the input file. Luckily, FFmpeg supports opening videos from URIs.
        let input = ffmpeg::format::input(&Path::new(&self.uri)).unwrap();
        // Now create the video stream data.
        let video = Arc::new(Mutex::new(VideoStreamData::new(
            StreamData::new(
                &input.streams().base(Type::Video).unwrap(),
                Decoder::new_video,
                Arc::clone(&time),
            ),
            Arc::clone(&self.texture),
        )));
        let weak_video = Arc::downgrade(&video);
        // get the duration
        let duration = video.lock().unwrap().stream.duration;
        // Now create the audio stream data.
        let audio = Arc::new(Mutex::new(AudioStreamData::new(
            StreamData::new(
                &input.streams().base(Type::Audio).unwrap(),
                Decoder::new_audio,
                Arc::clone(&time),
            ),
        )));
        let weak_audio = Arc::downgrade(&audio);
        // Create the state.
        let state = Arc::new(Mutex::new(VideoState {
            input,
            video,
            audio,
            time: Arc::clone(&time),
        }));
        let weak_state = Arc::downgrade(&state);

        // Store the duration and then move the TimeData into self.
        {
            let mut time = time.write().unwrap();
            time.duration = duration;
        }
        self.time.replace(time);

        let own_rt = rt;
        // This RuntimeData will be moved into the new thread, so we clone first.
        let rt = own_rt.clone();
        self.threads.push(thread::spawn(|| {
            run_player_thread(weak_state, enqueue_next_packet, rt)
        }));
        let rt = own_rt.clone();
        let weak_video_2 = Weak::clone(&weak_video);
        self.threads.push(thread::spawn(|| {
            run_player_thread(weak_video, play_video, rt)
        }));
        let rt = own_rt.clone();
        self.threads.push(thread::spawn(|| {
            run_player_thread(weak_audio, play_audio, rt)
        }));

        // Wait until the first frame has been decoded and we know the video size.
        let mut size = None;
        while let Some(video) = weak_video_2.upgrade() {
            let video = video.lock.unwrap();
            if video.width.is_some() && video.height.is_some() {
                size = Some((video.width.unwrap(), video.height.unwrap()));
                break;
            } else {
                thread::sleep(Duration::from_millis(5));
            }
        }

        self.state.replace(state);

        InitResult {
            duration,
            size: size.unwrap(),
        }
    }

    pub fn pause(&self) {
        if let Some(time) = self.time.as_ref() {
            let mut time = time.write().unwrap();
            time.pause();
        }
    }

    pub fn play(&self) {
        if let Some(time) = self.time.as_ref() {
            let mut time = time.write().unwrap();
            time.play();
        }
    }

    pub fn set_looping(&self, looping: bool) {
        if let Some(time) = self.time.as_ref() {
            let mut time = time.write().unwrap();
            time.looping = looping;
        }
    }

    pub fn position(&self) -> i64 {
        if let Some(time) = self.time.as_ref() {
            let time = time.read().unwrap();
            // Respect the pause state if necessary.
            let now = if let Some(paused) = time.paused {
                paused
            } else {
                Instant::now()
            };
            if now <= time.start_time {
                0
            } else {
                // Get only the position in the current loop.
                now.duration_since(time.start_time).as_millis() as i64 % time.duration
            }
        }
    }
}

impl Drop for FFmpegPlayer {
    fn drop(&mut self) {
        // Drop the Arc<VideoState> to signal threads to exit.
        self.state.take();
        // Wait for each thread to exit and print errors.
        while let Some(t) = self.threads.pop() {
            if let Err(err) = t.join() {
                warn!("thread exited with error: {:?}", err);
            }
        }
    }
}

impl TimeData {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            duration: 0,
        }
    }

    fn pause(&mut self) {
        if self.paused.is_none() {
            self.paused = Some(Instant::now());
        }
    }

    fn play(&mut self) {
        if let Some(paused) = self.paused.take() {
            self.start_time += Instant::now() - paused;
        }
    }
}

impl VideoStreamData {
    fn new(stream: StreamData, texture: Arc<ExternalTexture>) -> Self {
        Self {
            stream,
            width: None,
            height: None,
            texture,
        }
    }
}

impl AudioStreamData {
    fn new(stream: StreamData) -> Self {
        Self {
            stream,
            output_stream: None,
            source_frames: VecDeque::new(),
            resampled_frames: VecDeque::new(),
            resampler: None,
            sample_buffer: Arc::new(Mutex::new(None)),
        }
    }
}

impl TryFrom<&Format> for OutputFormat {
    type Error = failure::Error;

    fn try_from(format: &Format) -> Result<Self, Self::Error> {
        // Convert the data type from cpal to ffmpeg.
        let dst_format = match format.data_type {
            CpalSampleFormat::F32 => FFmpegSampleFormat::F32(SampleType::Packed),
            CpalSampleFormat::I16 => FFmpegSampleFormat::I16(SampleType::Packed),
            CpalSampleFormat::U16 => {
                return Err(failure::err_msg("Unsupported sample format U16!"));
            }
        };
        // Convert the cpal channel number to a ffmpeg channel layout.
        let channel_layout = match format.channels {
            1 => channel_layout::FRONT_CENTER,
            2 => channel_layout::FRONT_LEFT | channel_layout::FRONT_RIGHT,
            c => {
                return Err(failure::format_err!(
                    "Unsupported number of channels: {}!",
                    c
                ));
            }
        };

        Ok(Self {
            format: dst_format,
            channel_layout,
            rate: format.sample_rate.0,
        })
    }
}

impl StreamData {
    fn new<D: FnOnce(FFmpegDecoder) -> Decoder>(
        stream: &Stream,
        decoder_fn: D,
        time: Arc<RwLock<TimeData>>,
    ) -> Self {
        // Get the time base of the stream
        let time_base = stream.time_base();
        let time_base = time_base.numerator() as f64 / time_base.denominator() as f64;
        // Calculate duration in seconds.
        let duration = stream.duration() as f64 * time_base;
        // Convert to milliseconds as that's what Flutter expects.
        let duration = (duration * 1000_f64) as i64;

        Self {
            stream_index: stream.index(),
            decoder: decoder_fn(stream.codec().decoder()),
            time_base,
            duration,
            time,
            packet_queue: VecDeque::new(),
        }
    }
}

impl Decoder {
    fn new_video(d: FFmpegDecoder) -> Self {
        Decoder::Video(d.video().unwrap())
    }
    fn as_video(&mut self) -> &mut VideoDecoder {
        if let Decoder::Video(d) = self {
            d
        } else {
            panic!("wrong type")
        }
    }
    fn new_audio(d: FFmpegDecoder) -> Self {
        Decoder::Audio(d.audio().unwrap())
    }
    fn as_audio(&mut self) -> &mut AudioDecoder {
        if let Decoder::Audio(d) = self {
            d
        } else {
            panic!("wrong type")
        }
    }
}

enum LoopState {
    Running,
    Sleep(u64),
    Exit,
}

fn run_player_thread<F, T>(state: Weak<Mutex<T>>, f: F, rt: RuntimeData)
where
    F: Fn(&mut T, &RuntimeData) -> LoopState,
{
    // We have to exit the loop when the state has been lost.
    while let Some(state) = state.upgrade() {
        // Run this in a block to drop the MutexGuard as soon as possible.
        let loop_state = {
            let mut state = state.lock().unwrap();
            f(&mut *state, &rt);
        };

        match loop_state {
            LoopState::Running => (),
            LoopState::Sleep(millis) => thread::sleep(Duration::from_millis(millis)),
            LoopState::Exit => break,
        }
    }
}

fn enqueue_next_packet(state: &mut VideoState, _: &RuntimeData) -> LoopState {
    let video = state.video.lock().unwrap();
    let audio = state.audio.lock().unwrap();
    if video.stream.packet_queue.len() >= VIDEO_PACKET_QUEUE_MAX
        || audio.stream.packet_queue.len() >= AUDIO_PACKET_QUEUE_MAX {
        return LoopState::sleep(QUEUE_FULL_SLEEP);
    }
    // Drop the MutexGuard while we decode the next packet.
    drop(video);
    drop(audio);

    let packet = state.input.packets().next();
    let mut video = state.video.lock().unwrap();
    let mut audio = state.audio.lock().unwrap();
    if let Some((stream, packet)) = packet {
        let idx = stream.index();
        if idx == video.stream.stream_index {
            video.stream.packet_queue.push_back(PacketData::Packet(packet, state.loop_count));
        } else if idx == audio.stream.stream_index {
            audio.stream.packet_queue.push_bacl(PacketData::Packet(packet, state.loop_count));
        }
    } else {
        // EOF reached
        let time = state.time.read().unwrap();
        if !time.looping {
            return LoopState::Sleep(PAUSE_SLEEP);
        }
        // We're looping, so now we need to seek to the beginning of the input video.
        let _ = state.input.seek(0, 0..i64::max_value());
        // Signal the video player to flush its decoder when it reaches this packet.
        video.stream.packet_queue.push_back(PacketData::Flush);
        audio.stream.packet_queue.push_back(PacketData::Flush);
        state.loop_count += 1;
    }

    LoopState::Running
}

fn get_source_frame(video: &mut VideoStreamData) -> Result<(VideoFrame, u32), LoopState> {
    // Get a packet from the packet queue.
    let (packet, loop_count) = if let Some(packet) = video.stream.packet_queue.pop_front() {
        // Check what we found in the packet queue.
        match packet {
            PacketData::Packet(p, l) => (p, l),
            PacketData::Flush => {
                // Flush the decoder and return the next source frame.
                video.stream.decoder.as_video().flush();
                return get_source_frame(video);
            }
        }
    } else {
        return Err(LoopState::Sleep(NO_PACKET_SLEEP));
    };
    // Decode this packet into a frame.
    let decoder = video.stream.decoder.as_video();
    let mut frame = VideoFrame::empty();
    match decoder.decode(&packet, &mut frame) {
        Err(err) => {
            error!("failed to decode video frame: {}", err);
            Err(LoopState::Exit)
        }
        Ok(_) => {
            if frame.format() == PixelFormat::None {
                // Call this function recursively until we have a full frame decoded
                get_source_frame(video)
            } else {
                Ok((frame, loop_count))
            }
        }
    }
}

fn scale_source_frame(
    video: &mut VideoStreamData,
    source_frame: &VideoFrame,
) -> Result<VideoFrame, LoopState> {
    let size = if let Some(size) = video.texture.size() {
        size
    } else {
        // We don't know the target size, so sleep some more.
        return Err(LoopState::Sleep(NO_TEXTURE_SLEEP));
    };

    // Check that neither input dimensions and format nor output dimensions have changed.
    if let Some(scaler) = video.scaler.as_ref() {
        if scaler.context.input().width != source_frame.width()
            || scaler.context.input().height != source_frame.height()
            || scaler.context.input().format != source_frame.format()
            || scaler.context.output().width != size.0
            || scaler.context.output().height != size.1
        {
            video.scaler.take();
        }
    }
    // Create a new scaling context if necessary.
    let scaler = if let Some(scaler) = video.scaler.as_mut() {
        scaler
    } else {
        video.scaler.replace(ScalingContext {
            context: FFmpegScalingContext::get(
                source_frame.format(),
                source_frame.width(),
                source_frame.height(),
                PixelFormat::RGBA,      // Aha! Here's our pixel format we need!
                size.0,
                size.1,
                scaling::flag::BILINEAR,
            )
            .unwrap(),
        });
        video.scaler.as_mut().unwrap()
    };
    // Now create a new video frame and scale the source frame.
    let mut scaled_frame = VideoFrame::empty();
    scaler.context.run(source_frame, &mut scaled_frame).unwrap();
    scaled_frame.set_pts(source_frame.pts());
    Ok(scaled_frame)
}

fn play_video(video: &mut VideoStreamData, rt: &RuntimeData) -> LoopState {
    // Try to get a cached frame first.
    let (scaled_frame, loop_count) = if let Some(frame) = video.scaled_frame.take() {
        frame
    } else {
        // No scaled frame available, calculate a new one.
        let (source_frame, loop_count) = if let Some(frame) = video.source_frame.take() {
            frame
        } else {
            // No source frame available, so decode a new one.
            match get_source_frame(video) {
                Ok(frame) => frame,
                Err(state) => return state,
            }
        };
        // Store the frame size.
        video.width.replace(source_frame.width());
        video.height.replace(source_frame.height());
        // Scale the frame.
        match scale_source_frame(video, &source_frame) {
            Ok(frame) => (frame, loop_count),
            Err(state) => {
                video.source_frame.replace(frame);
                return state;
            }
        }
    };

    let start_time = {
        let time = video.stream.time.read().unwrap();
        // Now check for pause.
        if time.paused.is_some() {
            // Cache the frame.
            video.scaled_frame.replace((scaled_frame, loop_count));
            return LoopState::Sleep(PAUSE_SLEEP);
        }
        time.start_time
    };

    // Calculate display time for frame.
    let display_time = scaled_frame.pts().unwrap() as f64 * video.stream.time_base;
    let display_time =
        (display_time * 1000_f64) as u64 + (video.stream.duration as u64 * loop_count as u64);
    let display_time = start_time.add(Duration::from_millis(display_time));
    let now = Instant::now();
    if display_time > now {
        let diff = display_time.duration_since(now);
        if diff > *FRAME_SLEEP_EPSILON {
            video.scaled_frame.replace((scaled_frame, loop_count));
            return LoopState::Sleep((diff.as_millis() as u64).max(FRAME_TIMEOUT_MAX));
        }
    }

    // Now render the frame!
    let texture = Arc::clone(&video.texture);
    let thread_rt = rt.clone();
    let send_result = rt.post_to_render_thread(move |_| unsafe {
        let texture_name = texture.gl_texture().unwrap();
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture_name);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::TexSubImage2D(
            gl::TEXTURE_2D,
            0,                                         // mipmap level
            0,                                         // x offset
            0,                                         // y offset
            scaled_frame.width().try_into().unwrap(),  // width
            scaled_frame.height().try_into().unwrap(), // height
            gl::RGBA,                                  // format of the pixel data
            gl::UNSIGNED_BYTE,                         // data type of the pixel data
            scaled_frame.data(0).as_ptr() as *const _, // pixel data
        );
        let texture = Arc::clone(&texture);
        let _ = thread_rt.with_window(move |_| {
            texture.mark_frame_available();
        });
    });
    if send_result.is_err() {
        return LoopState::Exit;
    }

    LoopState::Running
}

fn get_audio_source_frames(audio: &mut AudioStreamData) -> Result<Vec<AudioFrame>>, LoopState> {
    // Get a packet from the packet queue.
    let (packet, _loop_count) = if let Some(packet) = audio.stream.packet_queue.pop_front() {
        // Check what we found in the packet queue.
        match packet {
            PacketData::Packet(p, l) => (p, l),
            PacketData::Flush => {
                // Flush the decoder and return the next source frame.
                audio.stream.decoder.as_audio().flush();
                return get_audio_source_frames(audio);
            }
        }
    } else {
        return Err(LoopState::Sleep(NO_PACKET_SLEEP));
    };
    // Decode this packet into one or more frames.

    // First, store the original data pointer and size because we need to change it and later
    // restore it to the original values to avoid memory leaks when dropping.
    let original = unsafe {
        let ptr = packet.as_ptr();
        ((*ptr).size, (*ptr).data)
    };
    let decoder = audio.stream.decoder.as_audio();
    let mut frames = Vec::new();
    loop {
        let mut frame = AudioFrame::empty();
        let decoded = match decoder.decode(&packet, &mut frame) {
            Err(err) => {
                error!("failed to decode audio frame: {}", err);
                return Err(LoopState::Exit);
            }
            Ok((_, decoded)) => {
                if frame.format() != FFmpegSampleFormat::None {
                    frames.push(frame);
                }
                decoded
            }
        };
        // Now increase the data pointer and decrease the size by the number of read bytes.
        unsafe {
            let ptr = packet.as_mut_ptr();
            (*ptr).size -= decoded;
            (*ptr).data = (*ptr).data.offset(decoded as isize);
        }
        if packet.size() == 0 {
            break;
        }
    }

    // Finally restore the packet data before dropping.
    unsafe {
        let ptr = packet.as_mut_ptr();
        (*ptr).size = original.0;
        (*ptr).data = original.1;
    }

    if frames.is_empty() {
        get_audio_source_frames(audio)
    } else {
        Ok(frames)
    }
}

fn resample_source_frame(
    audio: &mut AudioStreamData,
    source_frame: &AudioFrame,
) -> Vec<AudioFrame> {
    // Get the stream's output format.
    let (_stream, format) = audio.output_stream.as_ref().unwrap();
    // Get or create the correct resampler.
    let resampler = if let Some(resampler) = audio.resampler.as_mut() {
        resampler
    } else {
        audio.resampler.replace(ResamplingContext {
            context: FFmpegResamplingContext::get(
                source_frame.format(),
                source_frame.channel_layout(),
                source_frame.rate(),
                format.format,
                format.channel_layout,
                format.rate,
            )
            .unwrap(),
        });
        audio.resampler.as_mut().unwrap()
    };
    // Start resampling.
    let context = &mut resampler.context;
    let mut resampled_frames = Vec::new();

    let mut resampled_frame = AudioFrame::empty();
    let mut delay = context.run(source_frame, &mut resampled_frame).unwrap();
    resampled_frames.push(resampled_frame);
    while let Some(_) = delay {
        let mut resampled_frame = AudioFrame::empty();
        resampled_frame.set_channel_layout(format.channel_layout);
        resampled_frame.set_format(format.format);
        resampled_frame.set_rate(format.rate);
        delay = context.flush(&mut resampled_frame).unwrap();
        resampled_frames.push(resampled_frame);
    }

    resampled_frames
}

fn play_audio(audio: &mut AudioStreamData, _rt: RuntimeData) -> LoopState {
    // First of all, check for pause and pause/play the audio stream.
    {
        let time = audio.stream.time.read().unwrap();
        if time.paused.is_some() {
            if let Some(stream) = audio.output_stream.as_ref() {
                let _ = stream.0.pause();
            }
            return LoopState::Sleep(PAUSE_SLEEP);
        } else if let Some(stream) = audio.output_stream.as_ref() {
            let _ = stream.0.play();
        }
    }

    // Create a new audio stream if we don't have one.
    if audio.output_stream.is_none() {
        // Clone the sample buffer Arc so we can pass it to the callback.
        let sample_buffer = Arc::clone(&audio.sample_buffer);
        let output_stream = audio::AUDIO
            .create_output_stream(move |stream_data| buffer_callback(stream_data, &sample_buffer))
            .unwrap();
        // Convert the stream format from cpal to ffmpeg.
        let format = match (&output_stream.format).try_into() {
            Ok(format) => format,
            Err(e) => {
                error!("{}", e);
                return LoopState::Exit;
            }
        };
        // Create the sample buffer.
        let buffer = match output_stream.format.data_type {
            CpalSampleFormat::I16 => SampleBuffer::I16 {
                buffer: VecDeque::new(),
            },
            CpalSampleFormat::F32 => SampleBuffer::F32 {
                buffer: VecDeque::new(),
            },
            CpalSampleFormat::U16 => unreachable!(),
        };
        // Store stream and buffer.
        audio.output_stream.replace((output_stream, format));
        audio.sample_buffer.lock().unwrap().replace(buffer);
    }

    // Try to get a cached frame first.
    let resampled_frame = if let Some(frame) = audio.resampled_frames.pop_front() {
        frame
    } else {
        // No resampled frame available, calculate a new one.
        let source_frame = if let Some(frame) = audio.source_frames.pop_front() {
            frame
        } else {
            // No source frame available, so decode a new one.
            let frames = match get_audio_source_frames(audio) {
                Ok(frames) => frames,
                Err(state) => return state,
            };
            // Store the frames.
            audio.source_frames.extend(frames);
            audio.source_frames.pop_front().unwrap()
        };
        // Resample the frame.
        let mut resampled_frames = resample_source_frame(audio, &source_frame).into();
        audio.resampled_frames.append(&mut resampled_frames);
        audio.resampled_frames.pop_front().unwrap()
    };

    // Get the sample buffer.
    let mut buffer = audio.sample_buffer.lock().unwrap();
    let buffer = buffer.as_mut().unwrap();
    // Check for the sample data type.
    match buffer {
        SampleBuffer::F32 { buffer } => {
            // Check that we don't store too many samples.
            if buffer.len() >= AUDIO_BUFFER_MAX {
                audio.resampled_frames.push_front(resampled_frame);
                return LoopState::Sleep(QUEUE_FULL_SLEEP);
            }
            // Get frame data in the correct type.
            let frame_data = resampled_frame.data(0);
            let frame_data = unsafe {
                // FFmpeg internally allocates the data pointers, they're definitely aligned.
                #[allow(clippy::cast_ptr_alignment)]
                std::slice::from_raw_parts(
                    frame_data.as_ptr() as *const f32,
                    frame_data.len() / 4,
                )
            };
            // Store frame data in the sample buffer.
            buffer.extend(frame_data);
        }
        SampleBuffer::I16 { buffer } => {
            // Check that we don't store too many samples.
            if buffer.len() >= AUDIO_BUFFER_MAX {
                audio.resampled_frames.push_front(resampled_frame);
                return LoopState::Sleep(QUEUE_FULL_SLEEP);
            }
            // Get frame data in the correct type.
            let frame_data = resampled_frame.data(0);
            let frame_data = unsafe {
                // FFmpeg internally allocates the data pointers, they're definitely aligned.
                #[allow(clippy::cast_ptr_alignment)]
                std::slice::from_raw_parts(
                    frame_data.as_ptr() as *const i16,
                    frame_data.len() / 2,
                )
            };
            // Store frame data in the sample buffer.
            buffer.extend(frame_data);
        }
    }

    LoopState::Running
}

fn buffer_callback(stream_data: CpalStreamData, sample_buffer: &Arc<Mutex<Option<SampleBuffer>>>) {
    // Get the sample buffer.
    let mut sample_buffer = sample_buffer.lock().unwrap();
    if let Some(sample_buffer) = sample_buffer.as_mut() {
        // Check that data types match.
        match (stream_data, sample_buffer) {
            (
                CpalStreamData::Output {
                    buffer: UnknownTypeOutputBuffer::F32(ref mut stream_buffer),
                },
                SampleBuffer::F32 {
                    buffer: sample_buffer,
                },
            ) => {
                // Copy samples from one buffer to the other.
                copy_buffers(stream_buffer, sample_buffer, 0.0);
            }
            (
                CpalStreamData::Output {
                    buffer: UnknownTypeOutputBuffer::I16(ref mut stream_buffer),
                },
                SampleBuffer::I16 {
                    buffer: sample_buffer,
                },
            ) => {
                // Copy samples from one buffer to the other.
                copy_buffers(stream_buffer, sample_buffer, 0);
            }
            _ => (),
        }
    }
}

fn copy_buffers<T: Copy>(
    stream_buffer: &mut [T],
    sample_buffer: &mut VecDeque<T>,
    zero: T,
) -> usize {
    // Check that we don't access anything beyond buffer lengths.
    let len = stream_buffer.len().min(sample_buffer.len());
    let (front, back) = sample_buffer.as_slices();
    if front.len() >= len {
        // Just copy from the first slice, it's enough.
        (&mut stream_buffer[0..len]).copy_from_slice(&front[0..len]);
    } else {
        // Copy from both slices of the VecDeque.
        let front_len = front.len();
        (&mut stream_buffer[0..front_len]).copy_from_slice(&front[0..front_len]);
        (&mut stream_buffer[front_len..len]).copy_from_slice(&back[0..len - front_len]);
    }
    // Remove copied samples from our sample buffer.
    sample_buffer.rotate_left(len);
    sample_buffer.truncate(sample_buffer.len() - len);
    // Fill remaining stream buffer with silence.
    if len < stream_buffer.len() {
        warn!("Not enough samples to fill stream buffer!");
        for s in stream_buffer[len..].iter_mut() {
            *s = zero;
        }
    }
    len
}
