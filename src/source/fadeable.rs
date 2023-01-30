use std::sync::atomic::AtomicU8;
use std::time::Duration;
use std::sync::mpsc::{Receiver};
use crate::{Sample, Source};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub enum FadeDirection {
    In,
    Out,
    Nothing,
}



unsafe impl Send for FadeDirection {}
unsafe impl Sync for FadeDirection {}

#[derive(Clone, Debug)]
pub struct AtomicFadeDirection(Arc<AtomicU8>);
impl AtomicFadeDirection {
    pub fn change_direction(&self, direction: FadeDirection) {
        self.0.store(direction as u8, std::sync::atomic::Ordering::Relaxed);
    }
}




/// Internal function that builds a `Fadeable` object.
pub fn fadeable<I>(input: I, duration: Duration, ) -> (Fadeable<I>, AtomicFadeDirection)
where
    I: Source,
    I::Item: Sample,
{
    let duration = duration.as_secs() * 1000000000 + duration.subsec_nanos() as u64;

    let direction = Arc::new(AtomicU8::new(FadeDirection::Nothing as u8));
    let s = Fadeable {
        input,
        remaining_ns: duration as f32,
        total_ns: duration as f32,
        f: 1.0,
        direction: direction.clone(),
        current_direction: FadeDirection::Nothing as u8,
    };
    (s, AtomicFadeDirection(direction.clone()))
}

/// Filter that modifies raises the volume from silence over a time period.
#[derive(Clone, Debug)]
pub struct Fadeable<I> {
    input: I,
    remaining_ns: f32,
    total_ns: f32,
    f: f32,
    direction: Arc<AtomicU8>,
    current_direction: u8,
    
}

impl<I> Fadeable<I>
where
    I: Source,
    I::Item: Sample,
{
    /// Returns a reference to the inner source.
    #[inline]
    pub fn inner(&self) -> &I {
        &self.input
    }

    /// Returns a mutable reference to the inner source.
    #[inline]
    pub fn inner_mut(&mut self) -> &mut I {
        &mut self.input
    }

    /// Returns the inner source.
    #[inline]
    pub fn into_inner(self) -> I {
        self.input
    }
}

impl<I> Iterator for Fadeable<I>
where
    I: Source,
    I::Item: Sample,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {

        if self.direction.load(std::sync::atomic::Ordering::SeqCst) != self.current_direction {
            self.remaining_ns = self.total_ns;
            self.current_direction = self.direction.load(std::sync::atomic::Ordering::SeqCst);
        } 

        if self.remaining_ns <= 0.0 && (self.f >= 1.0 || self.f <= 0.0) {
            return self.input.next().map(|value| value.amplify(self.f)) 
        }

        // default is going lowwer
        let factor = if self.current_direction == FadeDirection::Out as u8 {
            self.remaining_ns / self.total_ns
        } else {
            1.0 - self.remaining_ns / self.total_ns
        };

        if factor < 0.0 {
            self.f = 0.0;
        }
        if factor > 1.0 {
            self.f = 1.0;
        }

        self.remaining_ns -=
            1000000000.0 / (self.input.sample_rate() as f32 * self.channels() as f32);
        self.input.next().map(|value| value.amplify(self.f))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.input.size_hint()
    }
}

impl<I> ExactSizeIterator for Fadeable<I>
where
    I: Source + ExactSizeIterator,
    I::Item: Sample,
{
}

impl<I> Source for Fadeable<I>
where
    I: Source,
    I::Item: Sample,
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.input.channels()
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }
}
