# IOT-Sound
This repository is a part of a school project for IDATA2304 – Computer communication and network programming course at NTNU.  
The following text is the project report.  

## Abstract
Picture this: You're at your university, it is lunchtime, and you're facing a dilemma; go to the school cafeteria or go to the nearby supermarket to get food? What is the most optimal choice. This is highly dependent on how busy the cafeteria is, it may be very busy, and you may not be able to get food in time before your next lecture!  
We have created a system that can help you make this decision. Introducing IOT-Sound, a system that can tell you how busy your school cafeteria is, or any other environment for that matter. (multiple environments can be tracked at the same time)  
It is based on an ESP32 microcontroller (or a Raspberry Pi) that gathers data from a microphone and sends it via our system to the frontend web application. There the measured data is displayed in a graph and the user can see how noisy, thus busy the environment is.
As of now, sensor node is only simulated due to lack of necessary equipment, but simulation itself can also run on a microcontroller. We went through many iterations of the system, and we have learned a lot about making such systems as robust as possible.  
Possible future work includes sending mobile notifications to the user when the environment is getting *too busy* – according to the user's preferences.

## Introduction

We propose a solution to a problem that exists at universities today. Currently, students cannot know whether or not a room is in use or if it's very busy. Is there a lecture there? Is the cafeteria full? This can cause students to waste time, perhaps by visiting a room to check if it is empty to use for group work, when there is in fact a lecture occurring. Our IoT system will allow students to check the loudness of the room before wasting time and energy traveling there.  
Our system is not limited to universities, it can in fact function in any public space that would have a need for it.  

The remainder of this report goes into more detail on some specific topics. Under [Theory and technology](#Theory-and-technology), we describe how the two main parts of the system work: [Sensor node](#Sensor-node) and [Visualization node](#Visualization-node). There are more details about our [programming](#Programming) and [protocols used](#Used-protocols) in the corresponding sections. Later we describe how we worked, under [Methodology](#Methodology). Under [Results](#Results), you can find details about the [structure](#Structure) of the system, and a description of our [user interface](#User-interface). Near the end, there is a [discussion](#Discussion) section where we discuss security, among other things. Lastly, there is [conclusion and future work](#Conclusion-and-future-work) section.

## Theory and technology
### Sensor node
In general, our solution consists of two parts: sensor node and visualization node. Sensor node is simpler, so we will start with that one.  
The sensor node is responsible for gathering data from the environment and sending it to the MQTT broker set up by our lecturer (more on MQTT under [Used protocols](#Used-protocols) section).
The initial idea was to use a ESP32 microcontroller with a physical microphone. Soon we learned that getting necessary hardware may be a problem, so we had to resort to simulating the data. Simulation can nevertheless run on a microcontroller.  
#### Simulation
In order to have a range of data that somewhat accurately mimics the real world, the simulation has two states: quiet and noisy (night and day in the code). In the quiet state there is less variation and a lower decibel cap (50 dB), whereas in the noisy state there is a louder range (40 to 100 dB) and a greater variation. This simulates times when the classroom has students who are for example working on group projects, and when the classroom is mostly quiet because there is a lecture.

### Visualization node
The responsibility of the visualization node is retrieving data from the MQTT broker, processing it and visualizing it in a meaningful way to the end-user.  
Our solution for that side of the project is a bit more elaborate. It consists of multiple smaller programs that have to be run simultaneously. These programs are:  
- iot_sound_backend: Retrieves data from the MQTT broker, processes it and saves in the database.  
- iot_sound_frontend: web application that visualizes data from the database.
- iot_sound_api: acts as a link between the database and frontend application.  
- In addition, a running [PostgreSQL](https://www.postgresql.org/) database is required for API and backend to function.  

Due to the division in different components, the frontend is independent of the other components and can run on a separate machine. In addition, all gathered data is saved in a database.  

### Programming
To program everything except for frontend, we used [Rust](https://www.rust-lang.org/) programming language. Frontend uses [React](https://reactjs.org/) with [typescript](https://www.typescriptlang.org/).  
We decided to use Rust as our programming language because it is a common choice for microcontrollers. Rust gives its programmers low level control without giving up features from higher level languages. It can be used to program microcontrollers and is memory safe, which is the main reason for why we chose to learn it. Apart from the sensor node, all components could have used other languages. We used Rust for the sake of sticking to the same technology for the majority of the project.  

Frontend is written in React because one of our team members has prior experience in it. Additionally, frontend libraries generally have a steep learning curve, and we did not feel the need to invest time in learning a new technology for that. It would be outside the scope of this project.  

### Used protocols
In this section we will describe the protocols we used and how are they used in our solution.
#### MQTT
A lightweight subscribe/publish messaging application layer protocol.  
In our case, the sensor publishes data to the broker, and iot_sound_backend subscribes to the broker and processes the data. Data is sent in CSV (comma-separated values) format. Like so: `30.205029,1669026612`, the first value is the loudness level in dB, the second one is a timestamp in Unix time. Data is validated by backend before being saved in the database. Sensor ID is grabbed from the topic the data was published to.
#### HTTP
Hypertext Transfer Protocol, also an application layer protocol.

HTTP in this project is used between the frontend and the API server to communicate.
This happens using REST (Representational State Transfer) which is an architectural style for providing standards between different computer systems.
That means the API has different endpoints to hit for the data it wants, and it doesn't need to get all the data at once. That allows us to have a separation of concern when it comes to querying data from the database and processing it, and rendering it on the frontend.
Some example of the endpoints are:  
- http://example.com/sound  
- http://example.com/sound/sorted  
- http://example.com/sound/sorted/limit?limit_amount=10  
- http://example.com/sensors  
- http://example.com/logs  

#### TCP
[TCP](https://no.wikipedia.org/w/index.php?title=TCP&oldid=20556710) or **Transmission Control Protocol** is a network protocol for connection oriented, reliable and error checked transmission of data.   
It is a **transport layer** protocol that works under the hood.
This project uses the TCP protocol as it is important for this project because the nature of TCP is that it is lossless, which means that the validity of the data is ensured since they are not just pushed like a stream like UDP. 

#### IP
[IP](https://en.wikipedia.org/w/index.php?title=IPv4&oldid=1124299621) or **Internet Protocol** is the network layer communications protocol.
It is the basis of the internet. It uses an addressing system (example: 192.168.1.1) and performs routing between source and the next router which is one hop closer to the intended destination host on another network.

In this project we use the addressing system to connect the different parts of the project, e.g. the sensor and the MQTT broker. It is also the base of all the other protocols used in this project.

#### Ethernet or wireless
This project uses a mix of wired and wireless.  
It isn't restricted to a specific version of the wireless or wired protocols.  

The sensor is mostly meant to be working wirelessly and be connected to the MQTT broker that way. The backend and API is ideally using a wired connection running on a server connected to the network for best possible connection and throughput speed.  
The frontend can either be hosted locally or served along with the API from a server but it is very flexible in how the end computer can connect.

## Methodology

Our group always met physically at campus in order to work together. We tried to meet at least two times a week, our course schedule permitting. When it came to the programming itself, we worked fast and tried to focus on adding new features rather than never breaking anything. This way we could see what was useful and what wasn't, and could instead go back and fix bugs or refactor.

## Results
### Structure
Our system consists of smaller components that communicate with each other and are mostly independent of each other. As mentioned and explained in more detail above ([Visualization node](#Visualization-node) under [Theory and technology](#Theory-and-technology)), visualization part consists of 3 components (+database). The sensor is more self-contained, easy to deploy on a microcontroller or a Raspberry Pi.  

Below you can find a simple diagram that depicts how data flows through our system, with description underneath.  
![dataflow](imgs/dataflow.png)  
Data is first generated in the sensor node, from there it is sent to the MQTT Broker set up by our lecturer. Backend subscribes to a specific topic on the MQTT Broker and thus receives the measurement forwarded by the broker. Then backend processes received data and saves it in a database. The frontend web application sends requests to the REST API, in turn it sends a request to the database. The database then sends requested data to the API, and it forwards it back to the frontend, where it is visualized.  

### User interface
The frontend web application is divided into four pages:
- Sensors – all sensors that are registered in the database
- Recent data – recent loudness represented in a graph form
- Historical data – all loudness represented in a graph form
- Logs – information about any issues that might have occurred in the backend

We opted for minimalistic design to show only details that are needed, and dark theme to reduce eye fatigue.  

Recent data page:  
![frontendscreenshot](imgs/frontendscreenshot.png)  

## Discussion
talk about lack of security?  
logs  
no sensor - simulation  

## Conclusion and future work

future work:
earlier mentioned: "Possible future work includes sending mobile notifications to the user when the environment is getting *too busy* – according to the user's preferences."

Multiple sensors could be used in one room in an attempt to make a map of the sound levels throughout the room. Reports could be generated about the average noise levels during a day, at specific times throughout a week, month, or year. The sensor data could be encrypted in order to protect the privacy of the location.

For the rest of the project, the logs could be filtered in order to not display the same error message hundreds of times.

## References
- Wikipedia contributors. (2022, November 28). *Transmission Control Protocol*. In Wikipedia, The Free Encyclopedia. Retrieved 11:10, November 28, 2022, from https://en.wikipedia.org/w/index.php?title=Transmission_Control_Protocol&oldid=1124312705  
- Wikipedia contributors. (2022, November 28). *IPv4*. In _Wikipedia, The Free Encyclopedia_. Retrieved 10:24, November 28, 2022, from https://en.wikipedia.org/w/index.php?title=IPv4&oldid=1124299621  
- The Rust open source community. (regularly updated) *The Rust Book* Retrieved multiple times October and November 2022, from https://doc.rust-lang.org/book/  
