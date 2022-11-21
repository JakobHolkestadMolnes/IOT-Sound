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

### Data simulation

### Data processing

### Visualization