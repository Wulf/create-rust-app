import React from 'react'
import './App.css'
import { Home } from './containers/Home'
import { Todos } from './containers/Todo'
import { Route, useNavigate, Routes } from 'react-router-dom'

const App = () => {
  const navigate = useNavigate()
  /* CRA: app hooks */
  
  // @ts-ignore
    return (
    <div className="App">
      <div className="App-nav-header">
        <div style={{ display: 'flex', flex: 1 }}>
          <a className="NavButton" onClick={() => navigate('/')}>Home</a>
          <a className="NavButton" onClick={() => navigate('/todos')}>Todos</a>
          {/* CRA: left-aligned nav buttons */}
        </div>
        <div>
          {/* CRA: right-aligned nav buttons */}
        </div>
      </div>
      <div style={{ margin: '0 auto', maxWidth: '800px' }}>
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/todos" element={<Todos />} />
            {/* CRA: routes */}
          </Routes>
      </div>
    </div>
  )
}

export default App
