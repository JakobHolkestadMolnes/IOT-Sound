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
                            <div className='esp32bg'>

                            <h3 className="text-2xl p-2 mb-40 font-bold"><span className='bg-g9 p-2 rounded-xl'>{sensor.id}</span></h3>
                            </div>
                            <div>
                            <p className="text-xl py-4 font-bold">Location: {sensor.location}</p>
                            <p className="text-xl py-4 font-bold">Type: {sensor.type_}</p>
                        </div>
                        </div>
                    )
                })
                }
          </div>
        </div>

    )
}

export default sensors;