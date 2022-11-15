import { Github, Linkedin } from '@icons-pack/react-simple-icons';
import { useState, useEffect } from 'react';
import { CartesianGrid, Legend, Line, LineChart, Tooltip, XAxis, YAxis } from 'recharts';

import axios from 'axios';

const content =   () => {

    type Sensor = {
        id: String,
        location: String,
        type_ : String,
    }


 type Root = data[][]

 interface data {
  id: number
  sensor_name: string
  sound: string
  time: Time
}

 interface Time {
  secs_since_epoch: number
  nanos_since_epoch: number
  dateT: string
}

    const [data, setData] = useState([]);

    useEffect(() => {
        axios.get('http://localhost:8081/sound/sorted')
            .then(res => {
                setData(res.data);
            })
            .catch(err => {
                console.log(err);
            })
    }, []);

    const secs_since_epoch_To_Date_string = (secs_since_epoch: number) => {
        var d = new Date(0);
        d.setUTCSeconds(secs_since_epoch);
        return d.toLocaleString();
    }



    return (
        <div className="content">
            <div className=" grid  place-items-center ">
{

    

                
                    /* create a chart for all the data based on sensor name */
                    data.map((sensor:Root, index) => {
                        return (
                            <div className="text-black grid bg-g5 p-4 m-2 rounded-xl " key={sensor[0].sensor_name}>
                                <h3 className="text-2xl font-bold">{sensor[0].sensor_name}</h3>
                                <LineChart width={600} height={300} data={sensor} >
                                    <YAxis type='category' domain={['auto','auto']} allowDataOverflow={false}  stroke='#ffffff'/>
                                    <Line type="monotone" dataKey="sound" stroke="#dd23ea" />
                                    <CartesianGrid stroke="#eee" strokeDasharray="5 5"/>
                                    <XAxis dataKey="time_string" stroke='#ffffff' />
                                    <Tooltip />
                                    <Legend  />
                                </LineChart>
                            </div>
                        )
                    })
                    }
            </div>
        </div>
    )
}

export default  content;