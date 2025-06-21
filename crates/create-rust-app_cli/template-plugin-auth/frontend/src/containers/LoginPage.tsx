import React, { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'

export const LoginPage = () => {
  const auth = useAuth()
  const navigate = useNavigate()
  const [email, setEmail] = useState<string>('')
  const [password, setPassword] = useState<string>('')
  const [processing, setProcessing] = useState<boolean>(false)

  const login = async () => {
    setProcessing(true)
    await auth.login(email, password)
    setProcessing(false)
  }

  if (auth.isAuthenticated) {
    navigate('/')
    return <div>Already logged in. Redirecting you to the home page...</div>
  }

  return (
    <div className="Form" style={{ textAlign: 'left' }}>
      <h1>Login</h1>
      <br />
      <div style={{ display: 'flex', flexFlow: 'column' }}>
        <label>Email</label>
        <input value={email} onChange={(e) => setEmail(e.target.value)} />
      </div>
      <div style={{ display: 'flex', flexFlow: 'column' }}>
        <label>Password</label>
        <input
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
      </div>
      <div style={{ display: 'flex', flexFlow: 'column' }}>
        <button disabled={processing} onClick={login}>
          Login
        </button>
      </div>
      <a
        style={{ marginTop: '30px' }}
        href="#"
        onClick={() => navigate('/register')}
      >
        Don't have an account? Click here to register.
      </a>
      <a
        style={{ marginTop: '30px' }}
        href="#"
        onClick={() => navigate('/recovery')}
      >
        Forgot your password? Click here to recover your account.
      </a>
    </div>
  )
}
