use dora_node_api::{self, DoraNode, Event};
use dora_ros2_bridge::{
    messages::custom_msgs::msg::{CustomAudio, RobotStatus},
    messages::std_msgs::msg::Header,
    ros2_client::{self, NodeOptions},
    rustdds::{self, policy},
};
use eyre::Result;
use futures::StreamExt;

fn main() -> Result<()> {
    println!("Starting custom message test node");

    // Initialize ROS2 node
    let ros_context = dora_ros2_bridge::create_ros2_context()?;
    let mut ros_node = ros_context
        .new_node(
            ros2_client::NodeName::new("/dora", "custom_message_test")?,
            NodeOptions::new().enable_rosout(true),
        )?;

    // Create publishers for custom messages
    let custom_audio_publisher = create_custom_audio_publisher(&mut ros_node)?;
    let robot_status_publisher = create_robot_status_publisher(&mut ros_node)?;

    // Initialize Dora node
    let (mut node, dora_events) = DoraNode::init_from_env()?;
    let mut events = futures::executor::block_on_stream(dora_events);

    // Process events
    while let Some(event) = events.next() {
        match event {
            Event::Input { id, metadata, data } => match id.as_str() {
                "custom_audio" => {
                    // Create custom audio message
                    let audio_data: Vec<u8> = (&data).try_into()?;
                    
                    let custom_audio = CustomAudio {
                        header: Header {
                            stamp: dora_ros2_bridge::messages::builtin_interfaces::msg::Time {
                                sec: 0,
                                nanosec: 0,
                            },
                            frame_id: "custom_audio".to_string(),
                        },
                        audio_data,
                        sample_rate: 48000,
                        channels: 2,
                        format: "S16LE".to_string(),
                        chunk_size: 1024,
                    };

                    println!("Publishing CustomAudio message");
                    custom_audio_publisher.publish(custom_audio)?;
                }
                "robot_status" => {
                    // Create robot status message
                    let robot_status = RobotStatus {
                        header: Header {
                            stamp: dora_ros2_bridge::messages::builtin_interfaces::msg::Time {
                                sec: 0,
                                nanosec: 0,
                            },
                            frame_id: "robot_status".to_string(),
                        },
                        robot_id: "test_robot".to_string(),
                        battery_level: 85,
                        is_connected: true,
                        active_sensors: vec!["camera".to_string(), "microphone".to_string()],
                    };

                    println!("Publishing RobotStatus message");
                    robot_status_publisher.publish(robot_status)?;
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

fn create_custom_audio_publisher(
    ros_node: &mut ros2_client::Node,
) -> Result<ros2_client::Publisher<CustomAudio>> {
    let topic_qos: rustdds::QosPolicies = {
        rustdds::QosPolicyBuilder::new()
            .durability(policy::Durability::Volatile)
            .liveliness(policy::Liveliness::Automatic {
                lease_duration: dora_ros2_bridge::ros2_client::ros2::Duration::INFINITE,
            })
            .reliability(policy::Reliability::Reliable {
                max_blocking_time: dora_ros2_bridge::ros2_client::ros2::Duration::from_millis(100),
            })
            .history(policy::History::KeepLast { depth: 10 })
            .build()
    };

    let audio_topic = ros_node
        .create_topic(
            &ros2_client::Name::new("/custom", "audio")?,
            ros2_client::MessageTypeName::new("custom_msgs", "CustomAudio"),
            &topic_qos,
        )?;

    let audio_publisher = ros_node
        .create_publisher::<CustomAudio>(&audio_topic, None)?;
    Ok(audio_publisher)
}

fn create_robot_status_publisher(
    ros_node: &mut ros2_client::Node,
) -> Result<ros2_client::Publisher<RobotStatus>> {
    let topic_qos: rustdds::QosPolicies = {
        rustdds::QosPolicyBuilder::new()
            .durability(policy::Durability::Volatile)
            .liveliness(policy::Liveliness::Automatic {
                lease_duration: dora_ros2_bridge::ros2_client::ros2::Duration::INFINITE,
            })
            .reliability(policy::Reliability::Reliable {
                max_blocking_time: dora_ros2_bridge::ros2_client::ros2::Duration::from_millis(100),
            })
            .history(policy::History::KeepLast { depth: 10 })
            .build()
    };

    let status_topic = ros_node
        .create_topic(
            &ros2_client::Name::new("/custom", "robot_status")?,
            ros2_client::MessageTypeName::new("custom_msgs", "RobotStatus"),
            &topic_qos,
        )?;

    let status_publisher = ros_node
        .create_publisher::<RobotStatus>(&status_topic, None)?;
    Ok(status_publisher)
}
