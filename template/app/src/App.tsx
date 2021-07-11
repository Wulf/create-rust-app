import React from 'react'
import './App.css'
import { Home } from './containers/Home'
import { Todos } from './containers/Todo'
import { Route, useHistory } from 'react-router-dom'

const App = () => {
  const history = useHistory()
  /* CRA: app hooks */
  
  return (
    <div className="App">
      <div className="App-nav-header">
        <div style={{ display: 'flex', flex: 1 }}>
          <a className="NavButton" onClick={() => history.push('/')}>Home</a>
          <a className="NavButton" onClick={() => history.push('/todos')}>Todos</a>
          {/* CRA: left-aligned nav buttons */}
        </div>
        <div>
          {/* CRA: right-aligned nav buttons */}
        </div>
      </div>
      <div style={{ margin: '0 auto', maxWidth: '800px' }}>
        <Route path="/" exact><Home /></Route>
        <Route path="/todos"><Todos /></Route>
        {/* CRA: routes */}
      </div>
    </div>
  )
}

export default App
