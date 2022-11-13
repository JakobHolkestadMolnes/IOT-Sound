import Sensors from "./sensors";

const home = () => {
    return (
        <div className="text-white grid  place-items-center ">
<div>
        <h1 className="text-5xl font-bold">Sound sensors</h1>    
        <Sensors/>

</div>
        </div>
    );
}
export default home;