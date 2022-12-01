import axios from 'axios';
import { useState, useEffect, useMemo } from 'react';

function formatDate(seconds: number) {
    const regex = /\([^\)]*\)/;
    let milliseconds = seconds * 1000;
    let dateObject = new Date(milliseconds);
    let dateString = dateObject.toString();
    return dateString.replace(regex, "").trim();
}

const logs = () => {

    type Log = {
        id: number,
        message: string,
        time : {
            secs_since_epoch: number,
            nanos_since_epoch: number,
        }
    }

    const [logs, setLogs] = useState([]);
    const [err, setErr] = useState(false);

    useEffect(() => {
        axios.get('http://localhost:8081/logs/limit?limit_amount=50')
            .then(res => {
                setLogs(res.data);
            })
            .catch(err => {
                console.log(err);
                setErr(true);
            })
    }, []);

    return (
        <div className="content min-h-full h-screen">
            <table className="text-white">
                <thead>
                    <th>ID</th>
                    <th>Message</th>
                    <th>Time</th>
                </thead>
                <tbody>
                    {
                        logs.map((log:Log, index) => {
                            return (
                                <tr key="{index}">
                                    <td>{log.id.toString()}</td>
                                    <td>{log.message}</td>
                                    <td>{formatDate(log.time.secs_since_epoch)}</td>
                                </tr>
                            )
                        })
                    }
                </tbody>
            </table>
        </div>
    );
}

export default logs;
