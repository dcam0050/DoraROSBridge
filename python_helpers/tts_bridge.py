#!/usr/bin/env python

import rospy
from std_msgs.msg import String
from pal_interaction_msgs.msg import TtsActionGoal
from actionlib_msgs.msg import GoalID
from std_msgs.msg import Header
import time

class TtsBridgeDirect:
    def __init__(self):
        rospy.init_node('tts_bridge_direct', anonymous=True)
        
        # Subscribe to the /tts/say topic
        self.subscriber = rospy.Subscriber('/tts/say', String, self.callback)
        
        # Publisher for the /tts/goal action topic
        self.publisher = rospy.Publisher('/tts/goal', TtsActionGoal, queue_size=10)
        
        rospy.loginfo("TTS Bridge Direct initialized. Subscribing to /tts/say and publishing to /tts/goal")
    
    def callback(self, msg):
        """Callback function that receives text from /tts/say and publishes to /tts/goal"""
        text = msg.data
        rospy.loginfo("Received text: {}".format(text))
        
        # Create the TtsActionGoal message using the imported message type
        action_goal = TtsActionGoal()
        
        # Set header
        action_goal.header = Header()
        action_goal.header.stamp = rospy.Time.now()
        action_goal.header.frame_id = ''
        
        # Set goal_id
        action_goal.goal_id = GoalID()
        action_goal.goal_id.stamp = rospy.Time.now()
        action_goal.goal_id.id = "tts_goal_{}".format(int(time.time()))
        
        # Set the goal content
        # Fill in the rawtext with the received string and 'en' language
        action_goal.goal.rawtext.text = text
        action_goal.goal.rawtext.lang_id = 'en'
        
        # Set other required fields to empty/default values
        action_goal.goal.speakerName = ''
        action_goal.goal.wait_before_speaking = 0.0
        
        # Publish the message
        self.publisher.publish(action_goal)
        rospy.loginfo("Published TTS goal for text: '{}'".format(text))
    
    def run(self):
        """Main run loop"""
        rospy.spin()

if __name__ == '__main__':
    try:
        bridge = TtsBridgeDirect()
        bridge.run()
    except rospy.ROSInterruptException:
        pass
