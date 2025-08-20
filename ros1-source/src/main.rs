use dora_node_api::{self, DoraNode, Event, IntoArrow, MetadataParameters, Parameter, dora_core::config::DataId};
use std::sync::{Arc, Mutex};

use rosrust_msg::sensor_msgs::Image as RosImage;

fn main() -> eyre::Result<()> {
    println!("starting ROS1 image bridge node");

    // Dora output id for images
    let output = DataId::from("image".to_owned());

    // Initialize Dora
    let (mut node, mut events) = DoraNode::init_from_env()?;

    // Initialize ROS1 and subscribe to sensor_msgs/Image
    rosrust::init("dora_ros1_image_node");
    let image_topic = std::env::var("ROS_IMAGE_TOPIC").unwrap_or_else(|_| "/camera/image_raw".to_string());

    let latest_image: Arc<Mutex<Option<RosImage>>> = Arc::new(Mutex::new(None));
    let latest_image_cb = Arc::clone(&latest_image);
    let _sub = rosrust::subscribe(&image_topic, 1, move |msg: RosImage| {
        if let Ok(mut slot) = latest_image_cb.lock() {
            *slot = Some(msg);
        }
    }).map_err(|e| eyre::eyre!("ros subscribe error: {e}"))?;

    // Forward latest ROS image to Dora on each tick
    while let Some(event) = events.recv() {
        match event {
            Event::Input { id, metadata, data: _ } => match id.as_str() {
                "tick" => {
                    let maybe_image = {
                        // acquire briefly
                        let guard = latest_image.lock().ok();
                        guard.and_then(|mut g| g.take())
                    };

                    if let Some(img) = maybe_image {
                        let mut params: MetadataParameters = metadata.parameters;
                        params.insert("width".into(), Parameter::Integer(img.width as i64));
                        params.insert("height".into(), Parameter::Integer(img.height as i64));
                        params.insert("encoding".into(), Parameter::String(img.encoding.clone()));
                        params.insert("step".into(), Parameter::Integer(img.step as i64));
                        params.insert("is_bigendian".into(), Parameter::Integer(img.is_bigendian as i64));

                        // Send raw image bytes as Arrow UInt8Array
                        let len = img.data.len();
                        println!("sending image: {len} bytes, {}x{}, {}", img.width, img.height, img.encoding);
                        node.send_output(output.clone(), params, img.data.into_arrow())?;
                    } else {
                        // No image received yet; ignore this tick
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

    Ok(())
}
