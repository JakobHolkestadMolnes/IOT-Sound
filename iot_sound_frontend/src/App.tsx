import { useState } from 'react'
import logo from './logo.svg'
import './App.css'
import Sidebar from './components/sidebar'
import { BrowserRouter, Route, Routes } from 'react-router-dom'
import Recentdata from './components/recentdata'
import Home from './components/home'
import Historicaldata from './components/historicaldata'


function App() {

  return (
    <div className="App flex h-screen">
      <BrowserRouter>

        <Sidebar />



        <div className='p-6 container ml-60'>
        <Routes>

          <Route path="/" element={<Home/>} />
          <Route path="/recentdata" element={<Recentdata />} />
          <Route path="/historicaldata" element={<Historicaldata />} />

        </Routes>
        </div>

      </BrowserRouter>

    </div>
  )
}

export default App
