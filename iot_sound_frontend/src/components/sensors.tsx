import { Github, Linkedin } from '@icons-pack/react-simple-icons';
import { useState, useEffect } from 'react';


import axios from 'axios';

const sensors =   () => {

    type Sensor = {
        id: String,
        location: String,
        type_ : String,
    }

    const [sensors, setSensors] = useState([]);
    
    useEffect(() => {
        axios.get('http://localhost:8081/sensors')
            .then(res => {
                setSensors(res.data);
            })
            .catch(err => {
                console.log(err);
            })
    }, []);

    




    return (
        <div className="content">
            <div className="text-black flex ">
            {/* return divs containing sensor names*/
                sensors.map((sensor:Sensor) => {
                    return (
                        <div className="text-black grid bg-g7 p-4 m-2 rounded-xl ">

                            <h3 className="text-2xl font-bold">{sensor.id}</h3>

                            <p className="text-xl font-bold">Location: {sensor.location}</p>
                            <p className="text-xl font-bold">Type: {sensor.type_}</p>
                        </div>
                    )
                })
                }
          </div>
        </div>

    )
}

export default sensors;