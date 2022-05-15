import React, { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'

export const RecoveryPage = () => {
  const auth = useAuth()
  const navigate = useNavigate()
  const [email, setEmail] = useState<string>('')
  const [processing, setProcessing] = useState<boolean>(false)

  const recover = async () => {
    setProcessing(true)
    const response = await (
      await fetch('/api/auth/forgot', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email }),
      })
    ).json()
    console.log(response)
    setProcessing(false)
    setEmail('')
  }

  if (auth.isAuthenticated) {
    navigate('/')
    return <div>Already logged in. Redirecting you to the home page...</div>
  }

  return (
    <div className="Form" style={{ textAlign: 'left' }}>
      <h1>Account Recovery</h1>
      <br />
      <div style={{ display: 'flex', flexFlow: 'column' }}>
        <label>Email</label>
        <input value={email} onChange={(e) => setEmail(e.target.value)} />
      </div>
      <div style={{ display: 'flex', flexFlow: 'column' }}>
        <button disabled={processing} onClick={recover}>
          Recover
        </button>
      </div>
    </div>
  )
}
