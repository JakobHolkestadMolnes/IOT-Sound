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

### Sensor Node
The responsibility of the sensor node is to gather data and send it to the MQTT broker.  
For this project, the sensor node measures the noise level (in dB) in the environment and sends measured data along with the timestamp of the measurement to the MQTT server.  
Equipment used:
- ESP32 microcontroller  
- ~~a microphone (RS PRO Omnidirectional Microphone Condenser)~~  

Due to lack of equipment, data from the sensor is simulated. Simulation is either way run on an ESP32.  

### Visualization Node
The responsibility of the visualization node is retrieving data from the MQTT broker, processing it and visualizing it in a meaningful way to the end-user.  
Our solution for that side of the solution is a bit more elaborate. It consists of multiple smaller programs that have to be run simultaneously. These programs are: *iot_sound_backend*, *iot_sound_api* and *iot_sound_frontend*. In addition, a running [PostgreSQL](https://www.postgresql.org/) database is required for api and backend to function.  
iot_sound_backend: Retrieves data from the MQTT broker, processes it and saves in the database.  
iot_sound_frontend: web application that visualizes data from the database.
iot_sound_api: acts as a link between the database and frontend application.  
Due to the division in different components, the frontend is independent off the other components and can run on a separate machine. In addition, all gathered data is saved in a database.

## Technologies used
Physical sensor node: ESP32 microcontroller, ~~a microphone~~.  
To program everything except for frontend, we used [Rust](https://www.rust-lang.org/) programming language. Frontend uses [React](https://reactjs.org/) with [typescript](https://www.typescriptlang.org/).  
Why Rust? Rust gives its programmers low level control without giving up features from higher level languages. It can be used to program microcontrollers and is memory safe, which is the main reason for why we chose to learn it.  All components other than sensor node (and frontend) use Rust for the sake of sticking to the same technology over the majority of the project.  

Frontend is written in React because one of our team members has prior experience in it. Additionally, frontend libraries generally have a steep learning curve, and we did not feel the need to invest time in learning a new technology for that. It would be outside the scope of this project.  

### Network protocols used
Backend gathers data from sensor via MQTT broker, which uses the MQTT protocol. This protocol is lightweight and rather simple, which makes it a perfect fit for an IoT project. These MQTT packets are transported with TCP, the reliable transport layer protocol.  
IP protocol is used to address the MQTT broker and the database. PostgreSQL database itself also uses TCP for communication.  
The visualization frontend is a web application, which means it uses HTTP (again using TCP and IP (localhost) under the hood).  

It's also important to note that there are multiple underlying protocols on the link and the physical layers, but we don't interact with them directly.

### Data simulation
We did not get the microphone that could be used as a sensor, so we had to resort to simulating the data. Simulation can be run on an ESP32 microcontroller that is connected to the internet, additionally it can simply be run on a computer.  

**TODO insight into the simulation here**

### Data processing
Sensor node packages measured data with the timestamp of the measurement in a CSV (comma-separated values) format. Like so: `30.205029,1669026612`, the first value is the loudness level in dB, the second one is a timestamp in Unix time.  

Data in this format is then captured by backend. It is there split by the comma, error checked, and both values are parsed. If everything is okay, the data is sent to the database.

### Visualization

### Results and potential future work