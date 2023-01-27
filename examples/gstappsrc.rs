use std::io::BufReader;
use gst::prelude::*;

use rodio::OutputStreamTrait;

fn main() {

    let _ = gst::init();

    let pipeline = gst::Pipeline::new(Some("pipe"));

    let maincaps = gst::Caps::builder("audio/x-raw")
        .field("format", &"F32LE")
        .field("rate", &44100i32)
        .field("channels", &2i32)
        .field("layout", &"interleaved")
        .build();
        
    let src = gst::ElementFactory::make("appsrc", None).unwrap();
    src.set_property("is-live", &true);
    src.set_property("block", &false);
    src.set_property("format", &gst::Format::Time);
    src.set_property("caps", &maincaps);

    let audioconvert = gst::ElementFactory::make("audioconvert", None).unwrap();
    let audiosink = gst::ElementFactory::make("autoaudiosink", None).unwrap();
    let queue = gst::ElementFactory::make("queue2", None).unwrap();
    queue.set_property("max-size-time", &32000u64);

    pipeline.add_many(&[&src, &audioconvert, &audiosink]).unwrap();
    gst::Element::link_many(&[&src, &audioconvert, &audiosink]).unwrap();
    //let output= OutputStream::try_default().unwrap();

    let appsrc = src
        .dynamic_cast::<gst_app::AppSrc>()
        .expect("Source element is expected to be an appsrc!");

    let (_stream, handle) = rodio::GstOutputStream::try_from_device(&appsrc).unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let file = std::fs::File::open("assets/music.mp3").unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());

    pipeline.set_state(gst::State::Playing).unwrap();
    sink.sleep_until_end();
    pipeline.set_state(gst::State::Null).unwrap();
}