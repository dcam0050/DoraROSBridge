use dora_node_api::{self, DoraNode, Event, Parameter};
use dora_ros2_bridge::{
    messages::sensor_msgs::msg::Image as Ros2Image,
    messages::builtin_interfaces::msg::Time,
    ros2_client::{self, NodeOptions, ros2},
    rustdds::{self, policy},
};
use eyre::{Context, eyre};
use futures::task::SpawnExt;

fn main() -> eyre::Result<()> {
    println!("starting ROS2 image sink node");

    // Get ROS2 topic from environment variable, default to /camera/image_raw
    let ros2_topic = std::env::var("ROS2_TOPIC").unwrap_or_else(|_| "/camera/image_raw".to_string());
    println!("ROS2 topic: {}", ros2_topic);

    // --- ROS 2 setup: node + publisher + spinner --------------------------------------------
    let mut ros_node = init_ros_node()?;
    let image_publisher = create_image_publisher(&mut ros_node, &ros2_topic)?;

    // background spinner (service discovery, executor, etc.)
    let pool = futures::executor::ThreadPool::new()?;
    let spinner = ros_node
        .spinner()
        .map_err(|e| eyre::eyre!("failed to create spinner: {e:?}"))?;
    pool.spawn(async {
        if let Err(err) = spinner.spin().await {
            eprintln!("ros2 spinner failed: {err:?}");
        }
    })
        .context("failed to spawn ros2 spinner")?;

    // --- Dora: init and process events ------------------------------------------------------
    let (_node, dora_events) = DoraNode::init_from_env()?;
    let mut events = futures::executor::block_on_stream(dora_events);

    while let Some(event) = events.next() {
        match event {
            Event::Input { id, metadata, data } => match id.as_str() {
                "image" => {
                    // Extract image metadata from Dora parameters
                    let width = metadata.parameters.get("width")
                        .and_then(|p| match p {
                            Parameter::Integer(w) => Some(*w as u32),
                            _ => None
                        })
                        .unwrap_or(0);
                    
                    let height = metadata.parameters.get("height")
                        .and_then(|p| match p {
                            Parameter::Integer(h) => Some(*h as u32),
                            _ => None
                        })
                        .unwrap_or(0);
                    
                    let encoding = metadata.parameters.get("encoding")
                        .and_then(|p| match p {
                            Parameter::String(e) => Some(e.clone()),
                            _ => None
                        })
                        .unwrap_or_else(|| "rgb8".to_string());
                    
                    let step = metadata.parameters.get("step")
                        .and_then(|p| match p {
                            Parameter::Integer(s) => Some(*s as u32),
                            _ => None
                        })
                        .unwrap_or(width * 3); // Default to width * 3 for RGB
                    
                    let is_bigendian = metadata.parameters.get("is_bigendian")
                        .and_then(|p| match p {
                            Parameter::Integer(b) => Some(*b != 0),
                            _ => None
                        })
                        .unwrap_or(false);

                    // Convert Arrow data to Vec<u8> using TryFrom
                    let image_data: Vec<u8> = (&data).try_into()
                        .context("failed to convert image data to bytes")?;

                    // Get current time for timestamp
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default();

                    // Create ROS2 Image message
                    let ros2_image = Ros2Image {
                        header: dora_ros2_bridge::messages::std_msgs::msg::Header {
                            stamp: Time {
                                sec: now.as_secs() as i32,
                                nanosec: now.subsec_nanos(),
                            },
                            frame_id: "camera_frame".to_string(),
                        },
                        height,
                        width,
                        encoding: encoding.clone(),
                        is_bigendian: if is_bigendian { 1 } else { 0 },
                        step,
                        data: image_data,
                    };

                    println!("publishing ROS2 image: {} bytes, {}x{}, encoding: {}", 
                             ros2_image.data.len(), width, height, encoding);
                    
                    image_publisher.publish(ros2_image)
                        .map_err(|e| eyre::eyre!("failed to publish image: {e:?}"))?;
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

fn init_ros_node() -> eyre::Result<ros2_client::Node> {
    let ros_context = dora_ros2_bridge::create_ros2_context().unwrap();

    ros_context
        .new_node(
            ros2_client::NodeName::new("/dora", "ros2_image_sink")
                .map_err(|e| eyre!("failed to create ROS2 node name: {e}"))?,
            NodeOptions::new().enable_rosout(true),
        )
        .map_err(|e| eyre::eyre!("failed to create ros2 node: {e:?}"))
}

fn create_image_publisher(
    ros_node: &mut ros2_client::Node,
    topic_name: &str,
) -> eyre::Result<ros2_client::Publisher<Ros2Image>> {
    let topic_qos: rustdds::QosPolicies = {
        rustdds::QosPolicyBuilder::new()
            .durability(policy::Durability::Volatile)
            .liveliness(policy::Liveliness::Automatic {
                lease_duration: ros2::Duration::INFINITE,
            })
            .reliability(policy::Reliability::Reliable {
                max_blocking_time: ros2::Duration::from_millis(100),
            })
            .history(policy::History::KeepLast { depth: 1 })
            .build()
    };

    // Parse topic name (e.g., "/camera/image_raw" -> namespace: "/camera", name: "image_raw")
    let (namespace, name) = if topic_name.starts_with('/') {
        let parts: Vec<&str> = topic_name.split('/').filter(|s| !s.is_empty()).collect();
        if parts.len() >= 2 {
            (format!("/{}", parts[0]), parts[1..].join("/"))
        } else {
            ("/".to_string(), "image_raw".to_string())
        }
    } else {
        ("/".to_string(), topic_name.to_string())
    };

    let image_topic = ros_node
        .create_topic(
            &ros2_client::Name::new(&namespace, &name)
                .map_err(|e| eyre!("failed to create ROS2 name: {e}"))?,
            ros2_client::MessageTypeName::new("sensor_msgs", "Image"),
            &topic_qos,
        )
        .context("failed to create topic")?;

    let image_publisher = ros_node
        .create_publisher::<Ros2Image>(&image_topic, None)
        .context("failed to create publisher")?;
    Ok(image_publisher)
}
