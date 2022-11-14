import { Link } from "react-router-dom";
import { AiOutlineHome } from "react-icons/ai";
import { IoAnalytics } from "react-icons/io5";

const sidebar = () => {

    return (
        <div className="h-screen w-48  bg-gray-700 container">
            
            <div>
                <h1 className="text-white text-4xl p-6">IoT Sound</h1>
            </div>
            <div className="flex flex-col pl-8 text-2xl  text-white ">

                <Link draggable="false" className="buttonGradiant mt-1 p-2 pr-0 hover:translate-x-1 bg-gray-600  hover:text-red-400 nodrag" to="/"><AiOutlineHome /> Sensors</Link>
                <Link draggable="false" className="buttonGradiant mt-1 p-2 pr-0 hover:translate-x-1 bg-gray-600  hover:text-red-400 nodrag" to="/socials"><IoAnalytics /> Data</Link>
                </div>

            <div className="absolute bottom-0 left-0">

</div>
        </div>
    )

}

export default sidebar;