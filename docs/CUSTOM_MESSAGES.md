# Custom Messages Guide

This guide explains how to create and use custom ROS2 messages in the ROS bridge project.

## 📁 **Custom Message Package Structure**

The project includes a local ROS2 message package at `custom_msgs/` that contains:

```
custom_msgs/
├── msg/                    # Message definitions
│   ├── CustomAudio.msg    # Custom audio message
│   └── RobotStatus.msg    # Robot status message
├── srv/                    # Service definitions
│   └── ProcessAudio.srv    # Audio processing service
├── action/                 # Action definitions
│   └── AudioProcessing.action  # Audio processing action
├── package.xml             # ROS2 package metadata
└── CMakeLists.txt          # Build configuration
```

## 🚀 **Quick Start**

### **1. Build Custom Messages**

```bash
# Build the custom message package
npm run build:custom-msgs

# Build ROS2 nodes with custom messages
npm run build:ros2:with-custom

# Test custom message compilation
npm run test:custom-msgs
```

### **2. Run Custom Message Test**

```bash
# Start the custom message test system
npm run start:custom

# In another terminal, monitor the topics
ros2 topic list
ros2 topic echo /custom/audio
ros2 topic echo /custom/robot_status
```

## 📝 **Message Definitions**

### **CustomAudio.msg**
```msg
# Custom audio message for the ROS bridge project
std_msgs/Header header
uint8[] audio_data
int32 sample_rate
int32 channels
string format
int32 chunk_size
```

### **RobotStatus.msg**
```msg
# Robot status message
std_msgs/Header header
string robot_id
int32 battery_level
bool is_connected
string[] active_sensors
```

### **ProcessAudio.srv**
```srv
# Audio processing service request
uint8[] audio_data
int32 sample_rate
string format
---
# Audio processing service response
bool success
string message
int32 processed_samples
```

### **AudioProcessing.action**
```action
# Audio processing action goal
uint8[] audio_data
int32 sample_rate
string format
---
# Audio processing action result
int32 total_samples_processed
float32 processing_time
string status_message
---
# Audio processing action feedback
int32 samples_processed_so_far
float32 current_processing_time
string current_status
```

## 🔧 **Using Custom Messages in Rust Code**

### **Import Custom Messages**

```rust
use dora_ros2_bridge::{
    messages::custom_msgs::msg::{CustomAudio, RobotStatus},
    messages::std_msgs::msg::Header,
    // ... other imports
};
```

### **Create and Publish Messages**

```rust
// Create a custom audio message
let custom_audio = CustomAudio {
    header: Header {
        stamp: dora_ros2_bridge::messages::builtin_interfaces::msg::Time {
            sec: 0,
            nanosec: 0,
        },
        frame_id: "custom_audio".to_string(),
    },
    audio_data: vec![1, 2, 3, 4], // Your audio data
    sample_rate: 48000,
    channels: 2,
    format: "S16LE".to_string(),
    chunk_size: 1024,
};

// Publish to ROS2
publisher.publish(custom_audio)?;
```

## 🛠️ **Adding New Custom Messages**

### **1. Create Message File**

Add a new `.msg` file to `custom_msgs/msg/`:

```msg
# custom_msgs/msg/MyNewMessage.msg
std_msgs/Header header
string data
int32 value
float64 timestamp
```

### **2. Update CMakeLists.txt**

Add the new message to `custom_msgs/CMakeLists.txt`:

```cmake
rosidl_generate_interfaces(${PROJECT_NAME}
  "msg/CustomAudio.msg"
  "msg/RobotStatus.msg"
  "msg/MyNewMessage.msg"  # Add your new message
  "srv/ProcessAudio.srv"
  "action/AudioProcessing.action"
  DEPENDENCIES std_msgs
)
```

### **3. Rebuild Messages**

```bash
npm run build:custom-msgs
npm run build:ros2:with-custom
```

### **4. Use in Your Code**

```rust
use dora_ros2_bridge::messages::custom_msgs::msg::MyNewMessage;

let my_message = MyNewMessage {
    header: Header { /* ... */ },
    data: "Hello World".to_string(),
    value: 42,
    timestamp: 123.456,
};
```

## 🔍 **Debugging Custom Messages**

### **Check Message Discovery**

```bash
# List all available interfaces
ros2 interface list | grep custom_msgs

# Show message details
ros2 interface show custom_msgs/msg/CustomAudio
```

### **Monitor Topics**

```bash
# List topics
ros2 topic list

# Echo topic data
ros2 topic echo /custom/audio

# Show topic info
ros2 topic info /custom/audio
```

### **Check Build Output**

```bash
# Check if messages are being generated
find target -name "messages.rs" -exec grep -l "custom_msgs" {} \;

# View generated message code
cat target/debug/build/dora-ros2-bridge-*/out/messages.rs | grep -A 10 "custom_msgs"
```

## 📋 **Available Commands**

| Command | Description |
|---------|-------------|
| `npm run build:custom-msgs` | Build the custom message package |
| `npm run build:ros2:with-custom` | Build ROS2 nodes with custom messages |
| `npm run test:custom-msgs` | Test custom message compilation |
| `npm run start:custom` | Start custom message test system |
| `npm run build:custom-test` | Build the custom message test node |

## 🎯 **Integration with Existing Systems**

The custom messages are automatically integrated with the `dora-ros2-bridge` and can be used alongside existing messages like `audio_common_msgs::AudioStamped`.

### **Example: Using Custom Messages with Audio System**

```rust
use dora_ros2_bridge::{
    messages::audio_common_msgs::msg::AudioStamped,
    messages::custom_msgs::msg::CustomAudio,
    // ... other imports
};

// You can use both standard and custom messages
let audio_stamped = AudioStamped { /* ... */ };
let custom_audio = CustomAudio { /* ... */ };
```

## 🔧 **Troubleshooting**

### **Common Issues**

1. **Messages not found**: Ensure `AMENT_PREFIX_PATH` includes the custom messages install directory
2. **Build errors**: Check that all dependencies are properly declared in `CMakeLists.txt`
3. **Runtime errors**: Verify that the message package is built and installed correctly

### **Debug Commands**

```bash
# Check AMENT_PREFIX_PATH
echo $AMENT_PREFIX_PATH

# Verify message package installation
ls -la custom_msgs/install/

# Test message discovery
ros2 interface list | grep custom_msgs
```

## 📚 **Next Steps**

- Add more custom messages as needed for your specific use case
- Create services and actions for more complex interactions
- Integrate custom messages with your existing Dora dataflows
- Consider creating message packages for different domains (sensors, control, etc.)
