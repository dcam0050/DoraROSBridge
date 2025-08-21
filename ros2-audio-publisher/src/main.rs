use dora_node_api::{self, DoraNode, Event, MetadataParameters, Parameter};
use dora_ros2_bridge::{
    messages::audio_common_msgs::msg::AudioStamped as Ros2AudioStamped,
    messages::audio_common_msgs::msg::{Audio as Ros2Audio, AudioData as Ros2AudioData, AudioInfo as Ros2AudioInfo},
    messages::std_msgs::msg::Header as Ros2Header,
    ros2_client::{self, NodeOptions, ros2},
    rustdds::{self, policy},
};
use eyre::{Context, eyre};
use futures::{StreamExt, task::SpawnExt};

fn main() -> eyre::Result<()> {
    println!("starting ROS2 audio publisher node with audio_common_msgs/AudioStamped");

    // Get audio topic from environment variable
    let audio_topic = std::env::var("ROS2_AUDIO_TOPIC").unwrap_or_else(|_| "/robot/audio".to_string());
    println!("Publishing to audio topic: {}", audio_topic);

    // --- ROS 2 setup: node + publisher --------------------------------------------
    let mut ros_node = init_ros_node()?;
    let audio_publisher = create_audio_publisher(&mut ros_node, &audio_topic)?;

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
    let (mut node, dora_events) = DoraNode::init_from_env()?;
    let mut events = futures::executor::block_on_stream(dora_events);

    // Process audio data from Dora and publish to ROS2
    while let Some(event) = events.next() {
        match event {
            Event::Input { id, metadata, data } => match id.as_str() {
                "audio" => {
                    // Extract audio data from Arrow format
                    let audio_data: Vec<u8> = (&data).try_into()
                        .map_err(|e| eyre::eyre!("Failed to convert data to bytes: {e}"))?;
                    
                    let audio_len = audio_data.len();
                    
                    // Extract metadata from Dora parameters
                    let sample_rate = metadata.parameters.get("sample_rate")
                        .and_then(|p| match p {
                            Parameter::String(sr) => sr.parse::<i32>().ok(),
                            Parameter::Integer(sr) => Some(*sr as i32),
                            _ => None
                        })
                        .unwrap_or(48000);
                    
                    let channels = metadata.parameters.get("channels")
                        .and_then(|p| match p {
                            Parameter::String(ch) => ch.parse::<i32>().ok(),
                            Parameter::Integer(ch) => Some(*ch as i32),
                            _ => None
                        })
                        .unwrap_or(1);
                    
                    let format_str = metadata.parameters.get("format")
                        .and_then(|p| match p {
                            Parameter::String(fmt) => Some(fmt.clone()),
                            _ => None
                        })
                        .unwrap_or_else(|| "S16LE".to_string());
                    
                    // Convert format string to format code (matching PortAudio format codes)
                    let format_code = match format_str.as_str() {
                        "S16LE" => 8,  // paInt16 = 0x00000008
                        "S32LE" => 2,  // paInt32  
                        "F32LE" => 1,  // paFloat32 = 0x00000001
                        "S8" => 16,    // paInt8 = 0x00000010
                        "U8" => 32,    // paUInt8 = 0x00000020
                        _ => 8,        // default to paInt16
                    };
                    
                    // Calculate chunk size (samples per chunk)
                    let bytes_per_sample = match format_code {
                        1 => 2,  // paInt16
                        2 => 4,  // paInt32
                        3 => 4,  // paFloat32
                        4 => 1,  // paInt8
                        5 => 1,  // paUInt8
                        _ => 2,  // default
                    };
                                               // Calculate chunk size (samples per chunk)
                           // The audio player expects chunk * channels samples
                           let chunk = audio_len as i32 / (bytes_per_sample * channels);
                    
                    // Create audio_common_msgs AudioStamped message (matching audio_capturer_node)
                    let audio_info = Ros2AudioInfo {
                        format: 8,
                        channels,
                        rate: sample_rate,
                        chunk,
                    };
                    
                    // Create audio data based on format (matching audio_capturer_node)
                    let audio_data_msg = match format_code {
                        8 => { // paInt16 = 0x00000008
                            let int16_data: Vec<i16> = audio_data.chunks(2)
                                .map(|chunk| {
                                    if chunk.len() == 2 {
                                        i16::from_le_bytes([chunk[0], chunk[1]])
                                    } else {
                                        0
                                    }
                                })
                                .collect();
                            Ros2AudioData {
                                int16_data,
                                ..Default::default()
                            }
                        }
                        2 => { // paInt32
                            let int32_data: Vec<i32> = audio_data.chunks(4)
                                .map(|chunk| {
                                    if chunk.len() == 4 {
                                        i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                                    } else {
                                        0
                                    }
                                })
                                .collect();
                            Ros2AudioData {
                                int32_data,
                                ..Default::default()
                            }
                        }
                        1 => { // paFloat32 = 0x00000001
                            let float32_data: Vec<f32> = audio_data.chunks(4)
                                .map(|chunk| {
                                    if chunk.len() == 4 {
                                        f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                                    } else {
                                        0.0
                                    }
                                })
                                .collect();
                            Ros2AudioData {
                                float32_data,
                                ..Default::default()
                            }
                        }
                        16 => { // paInt8 = 0x00000010
                            let int8_data: Vec<i8> = audio_data.iter()
                                .map(|&b| b as i8)
                                .collect();
                            Ros2AudioData {
                                int8_data,
                                ..Default::default()
                            }
                        }
                        32 => { // paUInt8 = 0x00000020
                            Ros2AudioData {
                                uint8_data: audio_data,
                                ..Default::default()
                            }
                        }
                        _ => { // default to int16
                            let int16_data: Vec<i16> = audio_data.chunks(2)
                                .map(|chunk| {
                                    if chunk.len() == 2 {
                                        i16::from_le_bytes([chunk[0], chunk[1]])
                                    } else {
                                        0
                                    }
                                })
                                .collect();
                            Ros2AudioData {
                                int16_data,
                                ..Default::default()
                            }
                        }
                    };
                    
                    let audio_msg = Ros2Audio {
                        audio_data: audio_data_msg,
                        info: audio_info,
                    };
                    
                    // Create header with current timestamp (matching audio_capturer_node)
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default();
                    
                    let header = Ros2Header {
                        stamp: dora_ros2_bridge::messages::builtin_interfaces::msg::Time {
                            sec: now.as_secs() as i32,
                            nanosec: now.subsec_nanos(),
                        },
                        frame_id: "robot_microphone".to_string(),
                    };
                    
                    let audio_stamped_msg = Ros2AudioStamped {
                        header,
                        audio: audio_msg,
                    };
                    
                    println!("Audio metadata - Rate: {}Hz, Channels: {}, Format: {} ({}), Chunk: {}, Length: {} bytes", 
                             sample_rate, channels, format_str, format_code, chunk, audio_len);

                    // Publish to ROS2
                    println!("Publishing audio_common_msgs/AudioStamped: {} bytes to topic {}", audio_len, audio_topic);
                    audio_publisher
                        .publish(audio_stamped_msg)
                        .map_err(|e| eyre::eyre!("failed to publish audio: {e:?}"))?;
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
    let ros_context = ros2_client::Context::new().unwrap();

    ros_context
        .new_node(
            ros2_client::NodeName::new("/dora", "ros2_audio_publisher")
                .map_err(|e| eyre!("failed to create ROS2 node name: {e}"))?,
            NodeOptions::new().enable_rosout(true),
        )
        .map_err(|e| eyre::eyre!("failed to create ros2 node: {e:?}"))
}

fn create_audio_publisher(
    ros_node: &mut ros2_client::Node,
    topic_name: &str,
) -> eyre::Result<ros2_client::Publisher<Ros2AudioStamped>> {
    let topic_qos: rustdds::QosPolicies = {
        rustdds::QosPolicyBuilder::new()
            .durability(policy::Durability::Volatile)
            .liveliness(policy::Liveliness::Automatic {
                lease_duration: ros2::Duration::INFINITE,
            })
            .reliability(policy::Reliability::Reliable {
                max_blocking_time: ros2::Duration::from_millis(100),
            })
            .history(policy::History::KeepLast { depth: 10 })
            .build()
    };

    // Parse topic name (e.g., "/robot/audio" -> namespace: "/robot", name: "audio")
    let (namespace, name) = if topic_name.starts_with('/') {
        let parts: Vec<&str> = topic_name.split('/').filter(|s| !s.is_empty()).collect();
        if parts.len() >= 2 {
            (format!("/{}", parts[0]), parts[1..].join("/"))
        } else {
            ("/".to_string(), "audio".to_string())
        }
    } else {
        ("/".to_string(), topic_name.to_string())
    };

    let audio_topic = ros_node
        .create_topic(
            &ros2_client::Name::new(&namespace, &name)
                .map_err(|e| eyre!("failed to create ROS2 name: {e}"))?,
            ros2_client::MessageTypeName::new("audio_common_msgs", "AudioStamped"),
            &topic_qos,
        )
        .context("failed to create topic")?;

    let audio_publisher = ros_node
        .create_publisher::<Ros2AudioStamped>(&audio_topic, None)
        .context("failed to create publisher")?;
    Ok(audio_publisher)
}
