import React, { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'

export const RegistrationPage = () => {
  const auth = useAuth()
  const navigate = useNavigate()
  const [email, setEmail] = useState<string>('')
  const [password, setPassword] = useState<string>('')
  const [processing, setProcessing] = useState<boolean>(false)

  const register = async () => {
    setProcessing(true)
    const response = (
      await fetch('/api/auth/register', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email, password }),
      })
    ).json()
    console.log(response)
    setProcessing(false)
    navigate('/activate')
  }

  return (
    <div className="Form" style={{ textAlign: 'left' }}>
      <h1>Registration</h1>
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
        <button disabled={processing} onClick={register}>
          Register
        </button>
      </div>
      <a
        style={{ marginTop: '30px' }}
        href="#"
        onClick={() => navigate('/login')}
      >
        Already have an account? Click here to login.
      </a>
      <a
        style={{ marginTop: '30px' }}
        href="#"
        onClick={() => navigate('/activate')}
      >
        Need to activate your account? Click here.
      </a>
    </div>
  )
}
