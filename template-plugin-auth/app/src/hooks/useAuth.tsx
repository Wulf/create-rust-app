import React, { createContext, useCallback, useContext, useEffect, useState } from 'react'

interface AccessToken {
  exp: number // DateTime in UTC seconds
  sub: number // subject's ID
  token_type: "access_token"
}

const MILLISECONDS_UNTIL_EXPIRY_CHECK = 10 * 1000 // check expiry every 10 seconds
const REMAINING_TOKEN_EXPIRY_TIME_ALLOWED = 60 * 1000 // 1 minute before token should be refreshed

interface AuthContext {
  accessToken: string | undefined
  parsedToken: AccessToken | undefined
  setAccessToken: (accessToken: string | undefined) => void
  setParsedToken: (parsedToken: AccessToken | undefined) => void

  isCheckingAuth: boolean
  setCheckingAuth: (checking: boolean) => void
}

interface AuthWrapperProps {
  children: React.ReactNode
}

const Context = createContext<AuthContext>(undefined as any)

export const AuthProvider = (props: AuthWrapperProps) => {
  const [accessToken, setAccessToken] = useState<string | undefined>()
  const [parsedToken, setParsedToken] = useState<AccessToken | undefined>()
  const [isCheckingAuth, setCheckingAuth] = useState<boolean>(false)
  
  return <Context.Provider value={{
    accessToken,
    parsedToken,
    setAccessToken,
    setParsedToken,
    isCheckingAuth,
    setCheckingAuth
  }}>
    {props.children}
  </Context.Provider>
}

export const useAuth = () => {
  const context = useContext(Context)

  const login = async (email: string, password: string): Promise<boolean> => {
    const response = await fetch('/api/auth/login', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ email, password })
    })

    if (response.ok) {
      const responseJson = await response.json()

      context.setAccessToken(responseJson.access_token)
      context.setParsedToken(parseJwt(responseJson.access_token))
      return true
    } else {
      context.setAccessToken(undefined)
      context.setParsedToken(undefined)
      return false
    }
  }

  const logout = async (): Promise<boolean> => {
    const response = await fetch('/api/auth/logout', {
      method: 'POST'
    })

    if (response.ok) {
      context.setAccessToken(undefined)
      context.setParsedToken(undefined)
      return true
    } else {
      return false
    }
  }
  
  return {
    accessToken: context.accessToken,
    parsedToken: context.parsedToken,
    isCheckingAuth: context.isCheckingAuth,
    isAuthenticated: !!context.accessToken,
    login,
    logout
  }
}

export const useAuthCheck = () => {
  const context = useContext(Context)

  const refreshIfNecessary = useCallback(async () => {
    context.setCheckingAuth(true)
    
    const isExpiringSoon = () => {
      if (context.parsedToken?.exp) {
        const expireTimeMS = context.parsedToken.exp * 1000
        const currentTimeMS = Date.now()
  
        return (expireTimeMS - currentTimeMS) <= REMAINING_TOKEN_EXPIRY_TIME_ALLOWED
      }
  
      return true
    }
    
    if (!context.accessToken || isExpiringSoon()) {
      console.log('Restoring session')
      const response = await fetch('/api/auth/refresh', {
        method: 'POST'
      })

      if (response.ok) {
        const responseJson = await response.json()
  
        context.setAccessToken(responseJson.access_token)
        context.setParsedToken(parseJwt(responseJson.access_token))
      } else {
        context.setAccessToken(undefined)
        context.setParsedToken(undefined)
      }
    } else {
      console.log(`${context.accessToken ? 'access token' : ''} ${isExpiringSoon() ? ' is not expiring' : ''}`)
    }

    context.setCheckingAuth(false)
  }, [context.accessToken, context.parsedToken])

  useEffect(() => {
    refreshIfNecessary()
    let intervalId: NodeJS.Timeout | undefined = undefined

    if (context.accessToken) {
      // if the access token is set, we want to check its expiry on some interval
      intervalId = setInterval(() => {
        refreshIfNecessary()
      }, MILLISECONDS_UNTIL_EXPIRY_CHECK)
    }
    return () => { if (intervalId) clearInterval(intervalId) }
  }, [refreshIfNecessary])
}

// https://stackoverflow.com/a/38552302
const parseJwt = (token: string) => {
  var base64Url = token.split('.')[1]
  var base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/')
  var jsonPayload = decodeURIComponent(atob(base64).split('').map(function(c) {
      return '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2)
  }).join(''))

  return JSON.parse(jsonPayload)
}
