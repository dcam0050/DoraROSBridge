# Python Helpers

This directory contains Python helper applications for ROS 1.

## TTS Bridge (`tts_bridge.py`)

A ROS 1 Python application that bridges between a simple text topic and the PAL Robotics TTS action interface.

### Functionality

- **Subscribes to**: `/tts/say` (std_msgs/String)
- **Publishes to**: `/tts/goal` (pal_interaction_msgs/TtsActionGoal)

The application receives text messages on `/tts/say` and converts them into properly formatted `TtsActionGoal` messages for the PAL Robotics TTS system.

### Message Structure

The application creates `TtsActionGoal` messages with the following structure:
- `rawtext.text`: The text received from `/tts/say`
- `rawtext.lang_id`: Set to 'en' (English)
- `speakerName`: Empty string
- `wait_before_speaking`: 0.0 seconds

### Dependencies

- ROS 1 (tested with Melodic)
- `pal_interaction_msgs` package
- Python 2.7 (for ROS Melodic)

## Remote Deployment (`deploy_and_run_remote.sh`)

A script to automatically deploy and run the TTS bridge on a remote system with ROS Melodic.

### Prerequisites

- SSH key-based authentication set up for the remote system
- ROS Melodic installed on the remote system
- PAL workspace with `pal_interaction_msgs` package available

### Configuration

Edit the script and set these variables:
```bash
REMOTE_USER="pal"
REMOTE_HOST="tiago-119c"
REMOTE_DIR="/home/$REMOTE_USER/tts_bridge"
```

### Usage

1. Configure the remote connection details in the script
2. Run the deployment script:
   ```bash
   ./deploy_and_run_remote.sh
   ```
3. The script will:
   - Copy the TTS bridge files to the remote system
   - Start the TTS bridge on the remote system with proper environment sourcing
   - Keep running until you press Ctrl+C
4. To stop the remote process, press Ctrl+C

### Environment Setup

The script automatically sources the required environment:
- `/opt/ros/melodic/setup.bash` - ROS Melodic
- `/home/pal/catkin_ws/devel/setup.bash` - PAL workspace
- `init_pal_env.sh` - PAL environment

### Example

```bash
# Edit the script first to set REMOTE_USER and REMOTE_HOST
# Then run:
./deploy_and_run_remote.sh

# The script will copy files and start the bridge on the remote system
# Press Ctrl+C to stop the remote process

# In another terminal, send text to be spoken:
rostopic pub /tts/say std_msgs/String "data: 'Hello robot, please speak this text'"
```


