import React, { useState } from 'react'
import { useAuth } from '../hooks/useAuth'

export const ActivationPage = () => {
  const auth = useAuth()
  const [activationToken, setActivationToken] = useState<string>('')
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
    console.log(response)
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
