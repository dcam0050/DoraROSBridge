use dora_node_api::{self, DoraNode, Event};
use eyre::eyre;

fn main() -> eyre::Result<()> {
    println!("starting ROS1 TTS sink node");

    // Initialize Dora
    let (mut node, mut events) = DoraNode::init_from_env()?;

    // Initialize ROS1
    rosrust::init("dora_ros1_tts_node");
    
    // Get TTS topic from environment variable
    let tts_topic = std::env::var("ROS_TTS_TOPIC").unwrap_or_else(|_| "/tts/say".to_string());
    
    println!("TTS topic: {}", tts_topic);

    // Create a publisher for the TTS topic
    let tts_publisher = rosrust::publish::<rosrust_msg::std_msgs::String>(&tts_topic, 1)
        .map_err(|e| eyre::eyre!("Failed to create TTS publisher: {e}"))?;



    // Process incoming text from Dora
    while let Some(event) = events.recv() {
        match event {
            Event::Input { id, metadata: _, data } => match id.as_str() {
                "text" => {
                    // Convert data to string (assuming it's bytes)
                    let text_bytes: Vec<u8> = (&data).try_into()
                        .map_err(|e| eyre::eyre!("Failed to convert data to bytes: {e}"))?;
                    
                    let text = String::from_utf8(text_bytes)
                        .map_err(|e| eyre::eyre!("Failed to convert bytes to string: {e}"))?;
                    
                    println!("Received text to speak: '{}'", text);
                    
                    // Create TTS goal message
                    let tts_msg = rosrust_msg::std_msgs::String {
                        data: text.clone(),
                    };
                    
                    // Send to TTS topic
                    tts_publisher.send(tts_msg)
                        .map_err(|e| eyre::eyre!("Failed to send TTS message: {e}"))?;
                    
                    println!("Sent text to TTS topic: '{}'", text);
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
