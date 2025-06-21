import React from 'react'
import ReactDOM from 'react-dom/client'
import App from '../src/App'
import reportWebVitals from '../src/reportWebVitals'
import {BrowserRouter} from 'react-router-dom'

ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
        {/* CRA: Wrap */}
        <BrowserRouter>
            <App/>
        </BrowserRouter>
        {/* CRA: Unwrap */}
    </React.StrictMode>
)

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals()
