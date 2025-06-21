import React, { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'
import { useQueryParam } from "../hooks/useQueryParam"

export const OauthLoginResultPage = () => {
  const auth = useAuth()
  const navigate = useNavigate()
  const errorMessage = useQueryParam('message')

  useEffect(() => {
    if (auth.completeOIDCLogin()) {
      navigate('/')
    }
  }, []);

  if (errorMessage) {
    return <div>Error: {errorMessage}</div>
  }

  return <div>Logging in...</div>
}