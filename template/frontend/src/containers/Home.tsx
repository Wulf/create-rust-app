import React from 'react'
import reactLogo from '../images/logo.svg'
import rustLogo from '../images/logo2.svg'
import plus from '../images/plus.svg'

export const Home = () => {
  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'center' }}>
        <img src={rustLogo} className="App-logo" alt="rust-logo" />
        <img src={plus} alt="plus" />
        <img src={reactLogo} className="App-logo" alt="react-logo" />
      </div>
      <p>
        Edit <code>app/src/App.tsx</code> and save to reload.
      </p>

      <div style={{ display: 'flex', justifyContent: 'center' }}>
        <a
          className="App-link"
          href="https://create-rust-app.dev"
          target="_blank"
          rel="noopener noreferrer"
        >
          Docs
        </a>
        &nbsp;
        <a
          className="App-link"
          href="https://github.com/Wulf/create-rust-app"
          target="_blank"
          rel="noopener noreferrer"
        >
          Repo
        </a>
      </div>
    </div>
  )
}
