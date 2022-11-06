import { useState } from 'react'
import logo from './logo.svg'
import './App.css'
import Sidebar from './components/sidebar'
import { BrowserRouter, Route, Routes } from 'react-router-dom'
import Content from './components/content'
import Home from './components/home'


function App() {

  return (
    <div className="App bg-gray-800 flex Gradiant">
      <BrowserRouter>

        <Sidebar />



        <div className='p-6 container'>
        <Routes>

          <Route path="/" element={<Home/>} />
          <Route path="/socials" element={<Content />} />

        </Routes>
        </div>

      </BrowserRouter>

    </div>
  )
}

export default App
