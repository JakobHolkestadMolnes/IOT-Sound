import { Github, Linkedin } from '@icons-pack/react-simple-icons';
import { useState } from 'react';
import { CartesianGrid, Line, LineChart, Tooltip, XAxis } from 'recharts';


const content =   () => {

    const [data, setData] = useState([]);
    
    const getSensorData = async () => {
        const response = await fetch('http://localhost:8080/sound');
        const data = await response.json();
        setData(data);
    };
    getSensorData();
    return (
        <div className="content">

            <LineChart
            width={500}
            height={300}
            data={data}
            margin={{ top: 5, right: 30, left: 20, bottom: 5 }}
            >
                <XAxis dataKey="time" />
                <Tooltip />

                <Line type="monotone" dataKey="sound" stroke="#8884d8" />

                <Line type="monotone" dataKey="sound" stroke="#ff7300" yAxisId={0} />

            </LineChart>
          
        </div>

    )
}

export default  content;