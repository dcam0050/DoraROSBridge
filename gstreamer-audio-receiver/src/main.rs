use dora_node_api::{self, DoraNode, Event, IntoArrow, MetadataParameters, Parameter, dora_core::config::DataId};
use eyre::{Context, eyre};
use futures::{StreamExt, task::SpawnExt};
use gstreamer as gst;
use gstreamer::prelude::{ElementExt, Cast, GstBinExt};
use gstreamer_app as gst_app;
use std::sync::{Arc, Mutex};

fn main() -> eyre::Result<()> {
    println!("starting GStreamer audio receiver node");

    // Initialize GStreamer
    gst::init()?;

    // Dora output id for audio data
    let output = DataId::from("audio".to_owned());

    // Get configuration from environment variables
    let udp_port = std::env::var("AUDIO_UDP_PORT").unwrap_or_else(|_| "5004".to_string());
    let sample_rate = std::env::var("AUDIO_SAMPLE_RATE").unwrap_or_else(|_| "48000".to_string());
    let channels = std::env::var("AUDIO_CHANNELS").unwrap_or_else(|_| "1".to_string());
    
    println!("Listening on UDP port: {}", udp_port);
    println!("Audio format: {}Hz, {} channels", sample_rate, channels);

    // Store latest audio data
    let latest_audio: Arc<Mutex<Option<Vec<u8>>>> = Arc::new(Mutex::new(None));
    let latest_audio_clone = Arc::clone(&latest_audio);

    // Create GStreamer pipeline for receiving audio
    let pipeline_str = format!(
        "udpsrc port={} caps=\"application/x-rtp,media=audio,encoding-name=L16,clock-rate={},channels={},payload=96\" ! \
         rtpjitterbuffer latency=10 ! \
         rtpL16depay ! \
         audioconvert ! \
         volume volume=0.5 ! \
         appsink name=appsink sync=false",
        udp_port, sample_rate, channels
    );

    println!("Creating GStreamer pipeline: {}", pipeline_str);

    let pipeline = gst::parse_launch(&pipeline_str)
        .context("Failed to create GStreamer pipeline")?;

    // Get the appsink element from the pipeline
    let pipeline_bin = pipeline
        .clone()
        .dynamic_cast::<gst::Bin>()
        .unwrap();
    
    let appsink = pipeline_bin
        .by_name("appsink")
        .expect("Failed to get appsink element");

    // Set up appsink callbacks
    let appsink = appsink.dynamic_cast::<gst_app::AppSink>().unwrap();
    appsink.set_callbacks(
        gst_app::AppSinkCallbacks::builder()
            .new_sample(move |appsink| {
                let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Error)?;
                let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                
                let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;
                let audio_data = map.as_slice().to_vec();
                let audio_len = audio_data.len();
                
                // Store the audio data
                if let Ok(mut guard) = latest_audio_clone.lock() {
                    *guard = Some(audio_data);
                }
                
                println!("Received audio data: {} bytes", audio_len);
                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    // Start the pipeline
    pipeline.set_state(gst::State::Playing)
        .context("Failed to start GStreamer pipeline")?;

    // Background task to handle GStreamer messages
    let pipeline_clone = pipeline.clone();
    let pool = futures::executor::ThreadPool::new()?;
    pool.spawn(async move {
        let bus = pipeline_clone.bus().unwrap();
        for msg in bus.iter_timed(gst::ClockTime::NONE) {
            match msg.view() {
                gst::MessageView::Error(err) => {
                    eprintln!("GStreamer error: {:?}", err);
                    break;
                }
                gst::MessageView::Eos(..) => {
                    println!("GStreamer pipeline reached end of stream");
                    break;
                }
                gst::MessageView::StateChanged(state) => {
                    println!("GStreamer state changed: {:?}", state);
                }
                _ => {}
            }
        }
    }).context("failed to spawn GStreamer message handler")?;

    // --- Dora: init and process events ------------------------------------------------------
    let (mut node, dora_events) = DoraNode::init_from_env()?;
    let mut events = futures::executor::block_on_stream(dora_events);

    // Forward latest audio to Dora on each tick
    while let Some(event) = events.next() {
        match event {
            Event::Input { id, metadata, data: _ } => match id.as_str() {
                "tick" => {
                    let maybe_audio = {
                        // acquire briefly
                        let guard = latest_audio.lock().ok();
                        guard.and_then(|mut g| g.take())
                    };

                    if let Some(audio_data) = maybe_audio {
                        let mut params: MetadataParameters = metadata.parameters;
                        params.insert("length".into(), Parameter::Integer(audio_data.len() as i64));
                        params.insert("sample_rate".into(), Parameter::String(sample_rate.clone()));
                        params.insert("channels".into(), Parameter::String(channels.clone()));
                        params.insert("format".into(), Parameter::String("S16LE".to_string()));

                        // Send audio data as Arrow BinaryArray
                        println!("sending audio data: {} bytes", audio_data.len());
                        node.send_output(output.clone(), params, audio_data.into_arrow())?;
                    } else {
                        // No audio received yet; ignore this tick
                    }
                }
                other => eprintln!("Ignoring unexpected input `{other}`"),
            },
            Event::Stop(_) => {
                println!("Received stop");
                break;
            }
            other => eprintln!("Received unexpected input: {other:?}"),
        }
    }

    // Cleanup
    pipeline.set_state(gst::State::Null)
        .context("Failed to stop GStreamer pipeline")?;

    Ok(())
}
