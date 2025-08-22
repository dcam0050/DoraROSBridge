#![allow(clippy::missing_safety_doc)]

pub use flume;
pub use futures;
pub use futures_timer;
pub use ros2_client;
pub use rustdds;
pub use tracing;
pub use eyre;

#[cfg(feature = "generate-messages")]
pub mod messages {
    include!(env!("MESSAGES_PATH"));
}

pub mod _core;

/// Create a ROS2 context with proper domain ID support.
/// 
/// This function reads the ROS_DOMAIN_ID environment variable and creates
/// a context with the specified domain ID. If ROS_DOMAIN_ID is not set,
/// it defaults to domain 0.
pub fn create_ros2_context() -> eyre::Result<ros2_client::Context> {
    let domain_id = std::env::var("ROS_DOMAIN_ID")
        .unwrap_or_else(|_| "0".to_string())
        .parse::<u16>()
        .unwrap_or(0);
    
    let context_options = ros2_client::ContextOptions::new().domain_id(domain_id);
    Ok(ros2_client::Context::with_options(context_options)?)
}
