#!/bin/bash

# Deploy and Run Audio Sender on Remote Robot
# This script sets up and runs the GStreamer audio sender on a remote robot

set -e

# Configuration - EDIT THESE VALUES
REMOTE_USER="pal"
REMOTE_HOST="tiago-119c"
REMOTE_DIR="/home/$REMOTE_USER/audio_sender"
LOCAL_IP="192.168.30.110"  # IP address of the local machine receiving audio
UDP_PORT="5004"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[$(date '+%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1"
}

warning() {
    echo -e "${YELLOW}[$(date '+%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1"
}

info() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')] INFO:${NC} $1"
}

# Check if remote connection details are configured
check_config() {
    if [[ "$REMOTE_USER" == "your_username" ]] || [[ "$REMOTE_HOST" == "your_remote_host" ]]; then
        error "Please edit this script and set REMOTE_USER and REMOTE_HOST variables"
        error "Current values:"
        error "  REMOTE_USER=$REMOTE_USER"
        error "  REMOTE_HOST=$REMOTE_HOST"
        exit 1
    fi
    
    if [[ "$LOCAL_IP" == "192.168.30.110" ]]; then
        warning "Please verify LOCAL_IP is correct for your network setup"
        warning "Current LOCAL_IP: $LOCAL_IP"
    fi
}

# Test SSH connection
test_connection() {
    log "Testing SSH connection to $REMOTE_USER@$REMOTE_HOST..."
    if ! ssh -o ConnectTimeout=10 -o BatchMode=yes "$REMOTE_USER@$REMOTE_HOST" "echo 'SSH connection successful'" 2>/dev/null; then
        error "Cannot connect to $REMOTE_USER@$REMOTE_HOST"
        error "Please check your SSH configuration and ensure key-based authentication is set up"
        exit 1
    fi
    log "SSH connection successful"
}

# Check if GStreamer is available on remote system
check_gstreamer() {
    log "Checking GStreamer availability on remote system..."
    if ! ssh "$REMOTE_USER@$REMOTE_HOST" "which gst-launch-1.0" 2>/dev/null; then
        error "GStreamer is not installed on the remote system"
        error "Please install GStreamer and its development packages"
        error "On Ubuntu/Debian: sudo apt-get install gstreamer1.0-tools gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly"
        exit 1
    fi
    log "GStreamer is available on remote system"
}

# Check if PulseAudio is available on remote system
check_pulseaudio() {
    log "Checking PulseAudio availability on remote system..."
    if ! ssh "$REMOTE_USER@$REMOTE_HOST" "which pulseaudio" 2>/dev/null; then
        error "PulseAudio is not installed on the remote system"
        error "Please install PulseAudio: sudo apt-get install pulseaudio"
        exit 1
    fi
    log "PulseAudio is available on remote system"
}

# Create audio sender script on remote system
create_audio_sender_script() {
    log "Creating audio sender script on remote system..."
    
    # Create remote directory
    ssh "$REMOTE_USER@$REMOTE_HOST" "mkdir -p $REMOTE_DIR"
    
    # Create the audio sender script
    cat << EOF | ssh "$REMOTE_USER@$REMOTE_HOST" "cat > $REMOTE_DIR/audio_sender.sh"
#!/bin/bash

# Audio Sender Script for Robot
# This script captures audio from the robot's microphone and streams it via UDP

set -e

# Configuration
TARGET_IP="$LOCAL_IP"
UDP_PORT="$UDP_PORT"
SAMPLE_RATE="48000"
CHANNELS="1"
FORMAT="S16LE"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "\${GREEN}[\$(date '+%Y-%m-%d %H:%M:%S')]\${NC} \$1"
}

error() {
    echo -e "\${RED}[\$(date '+%Y-%m-%d %H:%M:%S')] ERROR:\${NC} \$1"
}

warning() {
    echo -e "\${YELLOW}[\$(date '+%Y-%m-%d %H:%M:%S')] WARNING:\${NC} \$1"
}

# Check if target IP is reachable
check_connectivity() {
    log "Testing connectivity to \$TARGET_IP:\$UDP_PORT..."
    if ! timeout 5 bash -c "</dev/tcp/\$TARGET_IP/\$UDP_PORT" 2>/dev/null; then
        warning "Cannot reach \$TARGET_IP:\$UDP_PORT"
        warning "This might be normal if no receiver is listening yet"
    else
        log "Connectivity test successful"
    fi
}

# Start audio streaming
start_audio_stream() {
    log "Starting audio stream to \$TARGET_IP:\$UDP_PORT"
    log "Audio format: \${SAMPLE_RATE}Hz, \${CHANNELS} channel(s), \${FORMAT}"
    log "Press Ctrl+C to stop streaming"
    
    # GStreamer pipeline for audio capture and streaming
    gst-launch-1.0 -v \\
        pulsesrc do-timestamp=true buffer-time=10000 latency-time=5000 \\
        ! audio/x-raw,format=\${FORMAT},rate=\${SAMPLE_RATE},channels=\${CHANNELS} \\
        ! audioconvert \\
        ! rtpL16pay pt=96 \\
        ! udpsink host=\$TARGET_IP port=\$UDP_PORT
}

# Cleanup function
cleanup() {
    log "Stopping audio stream..."
    pkill -f "gst-launch-1.0.*udpsink" 2>/dev/null || true
    log "Audio stream stopped"
    exit 0
}

# Main execution
main() {
    log "Audio Sender for Robot"
    log "Target: \$TARGET_IP:\$UDP_PORT"
    
    # Check connectivity
    check_connectivity
    
    # Set up signal handler for Ctrl+C
    trap cleanup SIGINT SIGTERM
    
    # Start audio streaming
    start_audio_stream
}

# Show usage if help requested
if [[ "\$1" == "-h" ]] || [[ "\$1" == "--help" ]]; then
    echo "Usage: \$0"
    echo ""
    echo "This script captures audio from the robot's microphone and streams it via UDP."
    echo ""
    echo "Configuration:"
    echo "  TARGET_IP: \$TARGET_IP"
    echo "  UDP_PORT: \$UDP_PORT"
    echo "  SAMPLE_RATE: \$SAMPLE_RATE Hz"
    echo "  CHANNELS: \$CHANNELS"
    echo "  FORMAT: \$FORMAT"
    echo ""
    echo "The script will:"
    echo "1. Test connectivity to the target"
    echo "2. Start audio streaming using GStreamer"
    echo "3. Keep running until you press Ctrl+C"
    exit 0
fi

# Run main function
main "\$@"
EOF

    # Make the script executable
    ssh "$REMOTE_USER@$REMOTE_HOST" "chmod +x $REMOTE_DIR/audio_sender.sh"
    
    log "Audio sender script created at $REMOTE_DIR/audio_sender.sh"
}

# Run the audio sender on remote system
run_remote_audio_sender() {
    log "Starting audio sender on remote system..."
    log "Streaming audio to $LOCAL_IP:$UDP_PORT"
    log "Press Ctrl+C to stop the remote process"
    
    # Run the audio sender on remote system
    ssh "$REMOTE_USER@$REMOTE_HOST" "cd $REMOTE_DIR && ./audio_sender.sh"
}

# Cleanup function
cleanup() {
    log "Stopping remote audio sender..."
    ssh "$REMOTE_USER@$REMOTE_HOST" "pkill -f 'gst-launch-1.0.*udpsink'" 2>/dev/null || true
    log "Remote audio sender stopped"
    exit 0
}

# Main execution
main() {
    log "Audio Sender Remote Deploy and Run Script"
    log "Target: $REMOTE_USER@$REMOTE_HOST:$REMOTE_DIR"
    log "Streaming to: $LOCAL_IP:$UDP_PORT"
    
    # Check configuration
    check_config
    
    # Test connection
    test_connection
    
    # Check dependencies
    check_gstreamer
    check_pulseaudio
    
    # Create audio sender script
    create_audio_sender_script
    
    # Set up signal handler for Ctrl+C
    trap cleanup SIGINT SIGTERM
    
    # Run the audio sender
    run_remote_audio_sender
}

# Show usage if no arguments or help requested
if [[ "$1" == "-h" ]] || [[ "$1" == "--help" ]]; then
    echo "Usage: $0"
    echo ""
    echo "This script deploys and runs an audio sender on a remote robot."
    echo ""
    echo "Before running:"
    echo "1. Edit this script and set REMOTE_USER, REMOTE_HOST, and LOCAL_IP variables"
    echo "2. Ensure SSH key-based authentication is set up"
    echo "3. Make sure GStreamer and PulseAudio are installed on the remote system"
    echo ""
    echo "The script will:"
    echo "1. Create an audio sender script on the remote system"
    echo "2. Start audio streaming from the robot's microphone"
    echo "3. Stream audio to LOCAL_IP:UDP_PORT via UDP RTP"
    echo "4. Keep running until you press Ctrl+C"
    echo ""
    echo "Files will be created at: \$REMOTE_DIR on the remote system"
    echo ""
    echo "Audio format: 48kHz, 1 channel, S16LE, RTP L16 payload"
    exit 0
fi

# Run main function
main "$@"
