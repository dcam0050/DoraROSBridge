use dora_node_api::{self, DoraNode, Event, IntoArrow, MetadataParameters, Parameter, dora_core::config::DataId};
use dora_ros2_bridge::{
    messages::std_msgs::msg::String as Ros2String,
    ros2_client::{self, NodeOptions, ros2},
    rustdds::{self, policy},
};
use eyre::{Context, eyre};
use futures::{StreamExt, task::SpawnExt};
use std::sync::{Arc, Mutex};

fn main() -> eyre::Result<()> {
    println!("starting ROS2 TTS source node");

    // Dora output id for text
    let output = DataId::from("text".to_owned());

    // Get text topic from environment variable
    let text_topic = std::env::var("ROS2_TEXT_TOPIC").unwrap_or_else(|_| "/robot/say".to_string());
    println!("Subscribing to text topic: {}", text_topic);

    // --- ROS 2 setup: node + subscriber + spinner --------------------------------------------
    let mut ros_node = init_ros_node()?;
    let text_subscription = create_text_subscriber(&mut ros_node, &text_topic)?;

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

    // Store latest text message
    let latest_text: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let latest_text_clone = Arc::clone(&latest_text);

    // Spawn task to handle ROS2 messages
    pool.spawn(async move {
        loop {
            match text_subscription.take() {
                Ok(Some((text_msg, _info))) => {
                    let text = text_msg.data;
                    println!("Received text from ROS2: '{}'", text);
                    if let Ok(mut guard) = latest_text_clone.lock() {
                        *guard = Some(text);
                    }
                }
                Ok(None) => {
                    // No message available, continue polling
                }
                Err(e) => {
                    eprintln!("Error reading from subscription: {:?}", e);
                }
            }
                         // Small delay to avoid busy waiting
             futures_timer::Delay::new(std::time::Duration::from_millis(10)).await;
        }
    })
        .context("failed to spawn text handler")?;

    // --- Dora: init and process events ------------------------------------------------------
    let (mut node, dora_events) = DoraNode::init_from_env()?;
    let mut events = futures::executor::block_on_stream(dora_events);

    // Forward latest text to Dora on each tick
    while let Some(event) = events.next() {
        match event {
            Event::Input { id, metadata, data: _ } => match id.as_str() {
                "tick" => {
                    let maybe_text = {
                        // acquire briefly
                        let guard = latest_text.lock().ok();
                        guard.and_then(|mut g| g.take())
                    };

                    if let Some(text) = maybe_text {
                        let mut params: MetadataParameters = metadata.parameters;
                        params.insert("length".into(), Parameter::Integer(text.len() as i64));
                        params.insert("topic".into(), Parameter::String(text_topic.clone()));

                        // Send text as Arrow StringArray
                        let text_bytes = text.as_bytes().to_vec();
                        println!("sending text: '{}' ({} bytes)", text, text_bytes.len());
                        node.send_output(output.clone(), params, text_bytes.into_arrow())?;
                    } else {
                        // No text received yet; ignore this tick
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

fn init_ros_node() -> eyre::Result<ros2_client::Node> {
    let ros_context = dora_ros2_bridge::create_ros2_context().unwrap();

    ros_context
        .new_node(
            ros2_client::NodeName::new("/dora", "ros2_tts_source")
                .map_err(|e| eyre!("failed to create ROS2 node name: {e}"))?,
            NodeOptions::new().enable_rosout(true),
        )
        .map_err(|e| eyre::eyre!("failed to create ros2 node: {e:?}"))
}

fn create_text_subscriber(
    ros_node: &mut ros2_client::Node,
    topic_name: &str,
) -> eyre::Result<ros2_client::Subscription<Ros2String>> {
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

    // Parse topic name (e.g., "/robot/say" -> namespace: "/robot", name: "say")
    let (namespace, name) = if topic_name.starts_with('/') {
        let parts: Vec<&str> = topic_name.split('/').filter(|s| !s.is_empty()).collect();
        if parts.len() >= 2 {
            (format!("/{}", parts[0]), parts[1..].join("/"))
        } else {
            ("/".to_string(), "say".to_string())
        }
    } else {
        ("/".to_string(), topic_name.to_string())
    };

    let text_topic = ros_node
        .create_topic(
            &ros2_client::Name::new(&namespace, &name)
                .map_err(|e| eyre!("failed to create ROS2 name: {e}"))?,
            ros2_client::MessageTypeName::new("std_msgs", "String"),
            &topic_qos,
        )
        .context("failed to create topic")?;

    let text_subscriber = ros_node
        .create_subscription::<Ros2String>(&text_topic, None)
        .context("failed to create subscriber")?;
    Ok(text_subscriber)
}
