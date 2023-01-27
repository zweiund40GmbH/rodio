use std::io::{Read, Seek};
use std::marker::Sync;
use std::sync::{Arc, Weak};
use std::{error, fmt};
use crate::dynamic_mixer::{self, DynamicMixerController};

use crate::stream::{OutputStreamHandle, StreamError, OutputStreamTrait};
use gst::prelude::*;
use gst_app::prelude::*;
use byte_slice_cast::AsMutSliceOf;

/// `cpal::Stream` container. Also see the more useful `OutputStreamHandle`.
///
/// If this is dropped playback will end & attached `OutputStreamHandle`s will no longer work.
pub struct GstOutputStream {
    mixer: Arc<DynamicMixerController<f32>>,
    //_stream: cpal::Stream,
}

impl OutputStreamTrait for GstOutputStream {
    type SelfHandle = Self;
    type OutputDevice = gst_app::AppSrc;

    fn try_default() -> Result<(Self::SelfHandle, OutputStreamHandle), StreamError> {
        Err(StreamError::NoDevice)
    } 

    fn try_from_device(
        device: &Self::OutputDevice,
    ) -> Result<(Self, OutputStreamHandle), StreamError> {
        let caps = device.caps().unwrap();
        let new_pad_struct = caps.structure(0).expect("Failed to get first structure of caps");

        let rate = new_pad_struct.get::<i32>("rate").unwrap();
        let channels = new_pad_struct.get::<i32>("channels").unwrap();
        
        let (mixer_tx, mut mixer_rx) =
            dynamic_mixer::mixer::<f32>(channels as _ , rate as _ );

        device.set_callbacks(
            gst_app::AppSrcCallbacks::builder()
                .need_data(move |appsrc, length| {

                    let mut buffer = gst::Buffer::with_size(length as _).unwrap();
                    {
                        let ref_buf = buffer.make_mut();
                        let mut buf_map = ref_buf.map_writable().unwrap();
                        let buf_slice = buf_map.as_mut_slice_of::<f32>().unwrap();

                        for sample in buf_slice {
                            let a = mixer_rx.next().unwrap_or(0.0);
                            *sample = a;
                        }
                    }

                    appsrc.push_buffer(buffer).unwrap();
                })
                .build(),
        );

        let out = Self { mixer: mixer_tx };
        let handle = OutputStreamHandle {
            mixer: Arc::downgrade(&out.mixer),
        };

        Ok((out, handle))
    }
}

