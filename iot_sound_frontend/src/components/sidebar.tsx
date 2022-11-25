import { Link } from "react-router-dom";
import { AiOutlineHome } from "react-icons/ai";
import { IoAnalytics } from "react-icons/io5";
import { BsClockHistory } from "react-icons/bs";
import { SlNotebook } from "react-icons/sl";

const sidebar = () => {

    return (
        <div className="min-h-screen h-screen w-48  bg-gray-700 container fixed">
            
            <div>
                <h1 className="text-white text-4xl p-6">IoT Sound</h1>
            </div>
            <div className="flex flex-col pl-8 text-2xl  text-white ">

                <Link draggable="false" className="buttonGradiant mt-1 p-2 pr-0 hover:translate-x-1 bg-gray-600  hover:text-red-400 nodrag" to="/"><AiOutlineHome /> Sensors</Link>
                <Link draggable="false" className="buttonGradiant mt-1 p-2 pr-0 hover:translate-x-1 bg-gray-600  hover:text-red-400 nodrag" to="/recentdata"><IoAnalytics /> Recent data</Link>
                <Link draggable="false" className="buttonGradiant mt-1 p-2 pr-0 hover:translate-x-1 bg-gray-600  hover:text-red-400 nodrag" to="/historicaldata"><BsClockHistory /> Historical data</Link>
                <Link draggable="false" className="buttonGradiant mt-1 p-2 pr-0 hover:translate-x-1 bg-gray-600  hover:text-red-400 nodrag" to="/logs"><SlNotebook /> Logs</Link>

                </div>

            <div className="absolute bottom-0 left-0">

</div>
        </div>
    )

}

export default sidebar;
