use std::{sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender}, thread::{self, JoinHandle}, time::Duration};

use ffmpeg::{format::{input, Pixel}, media, software::scaling::{Context, Flags}, util::frame::Video};
use image::{DynamicImage, ImageBuffer};

pub struct Loader {
    handle: JoinHandle<()>,
    kill_tx: Sender<()>
}

impl Loader {
    pub fn init() -> (Loader, LoaderPlaybackHandle) {
        let (kill_tx, kill_rx) = channel();
        let (playback_tx, playback_rx) = channel();
        let loader_playback_handle = LoaderPlaybackHandle {
            rx: playback_rx
        };

        let handle = Loader::init_thread(kill_rx, playback_tx);

        let loader = Loader {
            handle,
            kill_tx
        };

        (loader, loader_playback_handle)
    }

    fn init_thread(
        kill_rx: Receiver<()>,
        loader_playback_tx: Sender<LoaderPlaybackMessage>
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            ffmpeg::init().unwrap();

            let mut ictx = input(&"tminus.mp4").unwrap();

            let input = ictx.streams()
                .best(media::Type::Video)
                .ok_or(ffmpeg::Error::StreamNotFound)
                .unwrap();

            let video_stream_index = input.index();
            let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters()).unwrap();
            let mut decoder = context_decoder.decoder().video().unwrap();

            let mut scaler = Context::get(
                decoder.format(),
                decoder.width(),
                decoder.height(),
                Pixel::RGBA,
                decoder.width(),
                decoder.height(),
                Flags::BILINEAR,
            ).unwrap();

            let mut frame_index = 0;

            for (stream, packet) in ictx.packets() {
                if stream.index() == video_stream_index {
                    decoder.send_packet(&packet).unwrap();
                    let mut decoded = Video::empty();
                    while decoder.receive_frame(&mut decoded).is_ok() {
                        let mut rgb_frame = Video::empty();
                        scaler.run(&decoded, &mut rgb_frame).unwrap();
                        let frame = Box::new(DynamicImage::ImageRgba8(ImageBuffer::from_raw(rgb_frame.width(), rgb_frame.height(), rgb_frame.data(0).into_iter().map(|x| *x).collect()).unwrap()));
                        loader_playback_tx.send(LoaderPlaybackMessage::NewFrame { index: frame_index, frame }).unwrap()
                    }
                }
            }
        })
    }
}

pub struct LoaderPlaybackHandle {
    rx: Receiver<LoaderPlaybackMessage>
}

impl LoaderPlaybackHandle {
    pub fn next_message(&mut self) -> Option<LoaderPlaybackMessage> {
        match self.rx.recv_timeout(Duration::from_nanos(1)) {
            Ok(msg) => Some(msg),
            Err(RecvTimeoutError::Timeout) => None,
            _ => panic!("hung up")
        }
    }
}

pub enum LoaderPlaybackMessage {
    NewFrame {
        index: usize,
        frame: Box<DynamicImage>,
    }
}
