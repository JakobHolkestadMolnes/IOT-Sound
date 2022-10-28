# IOT-Sound
## Overview
IOT Sound sensor that is used to measure how loud the environment is.  

Possible uses are
- measure busyness (how busy it is) of environment
- determine if the environment is too noisy to work comfortably
- determine if the environment is loud enough to damage hearing

Visualisation and sensor node(s) communicate via MQTT broker server set up by our lecturer.

## Design and methods
Solution consists of two parts:
- sensor node
- visualisation node

##### Sensor Node
ESP32 microcontroller with a microphone.  
TODO: what does it do and how and whatnot

##### Visualisation Node
Application that recieves data from the sensor node(s), processes it and visualises it to the user.

## Technologies used
ESP32 microcontroller, microphones  
[Rust](https://www.rust-lang.org/) programming language: for sensor node and backend of the visualisation node.
