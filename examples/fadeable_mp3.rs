use std::io::BufReader;
use rodio::Source;
use rodio::OutputStreamTrait;

use rodio::source::FadeDirection;
use std::sync::mpsc::channel;
use std::sync::Arc;

fn main() {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let file = std::fs::File::open("assets/music.mp3").unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();


    let s = source.fadeable(std::time::Duration::from_secs(1));
    let fader = s.1;
    sink.append(s.0.repeat_infinite().amplify(1.0));

    std::thread::sleep(std::time::Duration::from_secs(2));
    fader.change_direction(FadeDirection::Out);

    std::thread::sleep(std::time::Duration::from_secs(10));
    fader.change_direction(FadeDirection::In);
    sink.sleep_until_end();
}
