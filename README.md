# IOT-Sound
## Overview
IOT Sound sensor that is used to measure how loud the environment is.  

Possible uses are
- measure busyness (how busy it is) of environment
- determine if the environment is too noisy to work comfortably
- determine if the environment is loud enough to damage hearing

## Design and methods
Solution consists of two parts:
- sensor node
- visualization node

Visualization and sensor node(s) communicate via MQTT broker server set up by our lecturer.

##### Sensor Node
The responsibility of the sensor node is to gather data and send it to the MQTT broker.  
For this project, the sensor node measures the noise level (in dB) in the environment and sends measured data along with the timestamp of the measurement to the MQTT server.  
Equipment used:
- ESP32 microcontroller  
- ~~a microphone (RS PRO Omnidirectional Microphone Condenser)~~  
Due to lack of equipment, data from the sensor is simulated. Simulation is either way run on an ESP32.  

##### Visualization Node
The responsibility of the visualization node is retrieving data from the MQTT broker, processing it and visualizing it in a meaningful way to the end-user.  
Our solution for that side of the solution is a bit more elaborate. It consists of multiple smaller programs that have to be run simultaneously. These programs are: *iot_sound_backend*, *iot_sound_api* and *iot_sound_frontend*. In addition, a running [PostgreSQL](https://www.postgresql.org/) database is required. 

## Technologies used
ESP32 microcontroller, microphones  
[Rust](https://www.rust-lang.org/) programming language: for sensor node and backend of the visualisation node.
