import { Github, Linkedin } from '@icons-pack/react-simple-icons';
import { useState, useEffect } from 'react';
import { CartesianGrid, Legend, Area, AreaChart, Tooltip, XAxis, YAxis } from 'recharts';

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
                            <div className="text-black grid bg-g10 p-4 m-2 rounded-xl " key={sensor[0].sensor_name}>
                                <h3 className="text-2xl text-white font-bold">{sensor[0].sensor_name}</h3>
                                <AreaChart width={600} height={300} data={sensor} margin={{bottom: 50}}>
                                <defs>
          <linearGradient id="color" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#EC407A" stopOpacity={0.4} />
            <stop offset="75%" stopColor="#EC407A" stopOpacity={0.05} />
          </linearGradient>
        </defs>
                                    <YAxis type='number' domain={[0,120]} allowDataOverflow={true}  stroke='#ffffff'/>
                                    <Area type="monotone" dataKey="sound" stroke="#EC407A" fill="url(#color)"/>
                                    <CartesianGrid stroke="#eee" strokeDasharray="5 5" opacity={0.2}/>
                                    <XAxis interval={10} angle={90} textAnchor="start"  dataKey="time_string" stroke='#ffffff' hide={true}/>
                                    <Tooltip />
                                    <Legend verticalAlign="top" align="right" />
                                </AreaChart>
                            </div>
                        )
                    })
                    }
            </div>
        </div>
    )
}

export default  content;