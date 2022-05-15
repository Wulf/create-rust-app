import React, { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'
import { useQueryParam } from '../hooks/useQueryParam'

export const ResetPage = () => {
  const auth = useAuth()
  const navigate = useNavigate()
  const resetToken = useQueryParam('token')
  const [newPassword, setNewPassword] = useState<string>('')
  const [newPasswordConfirmation, setNewPasswordConfirmation] =
    useState<string>('')
  const [processing, setProcessing] = useState<boolean>(false)

  const reset = async () => {
    setProcessing(true)
    try {
      const response = await (
        await fetch('/api/auth/reset', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            reset_token: resetToken,
            new_password: newPassword,
          }),
        })
      ).json()
      console.log(response)
      navigate('/login')
      setNewPassword('')
      setNewPasswordConfirmation('')
    } finally {
      setProcessing(false)
    }
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
        <label>New Password</label>
        <input
          type="password"
          value={newPassword}
          onChange={(e) => setNewPassword(e.target.value)}
        />
      </div>
      <div style={{ display: 'flex', flexFlow: 'column' }}>
        <label>Confirm New Password</label>
        <input
          type="password"
          value={newPasswordConfirmation}
          onChange={(e) => setNewPasswordConfirmation(e.target.value)}
        />
      </div>
      <div style={{ display: 'flex', flexFlow: 'column' }}>
        <button disabled={processing} onClick={reset}>
          Recover
        </button>
      </div>
    </div>
  )
}
