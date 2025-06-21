import React, { useState } from 'react'
import { useNavigate } from 'react-router'
import { useAuth } from '../hooks/useAuth'
import { useQueryParam } from '../hooks/useQueryParam'

export const ActivationPage = () => {
  const auth = useAuth()
  const navigate = useNavigate()
  const token = useQueryParam('token') || '';
  const [activationToken, setActivationToken] = useState<string>(token)
  const [processing, setProcessing] = useState<boolean>(false)

  const activate = async () => {
    setProcessing(true)
    const response = await fetch(
      `/api/auth/activate?activation_token=${activationToken}`,
      {
        headers: {
          'Content-Type': 'application/json',
        },
      }
    )
    if (response.ok) {
      navigate('/login')
    }
    setProcessing(false)
  }

  return (
    <div className="Form" style={{ textAlign: 'left' }}>
      <h1>Activate</h1>
      <br />
      <div style={{ display: 'flex', flexFlow: 'column' }}>
        <label>Activation Token</label>
        <input
          type="password"
          value={activationToken}
          onChange={(e) => setActivationToken(e.target.value)}
        />
      </div>
      <div style={{ display: 'flex', flexFlow: 'column' }}>
        <button disabled={processing} onClick={activate}>
          Activate
        </button>
      </div>
    </div>
  )
}
