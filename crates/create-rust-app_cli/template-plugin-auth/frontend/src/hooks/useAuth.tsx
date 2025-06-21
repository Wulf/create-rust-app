import React, { createContext, MutableRefObject, useCallback, useContext, useEffect, useRef, useState } from 'react'

const MILLISECONDS_UNTIL_EXPIRY_CHECK = 10 * 1000 // check expiry every 10 seconds
const REMAINING_TOKEN_EXPIRY_TIME_ALLOWED = 60 * 1000 // 1 minute before token should be refreshed

class Permissions {
  private readonly rolesSet: Set<string>
  private readonly rolesArray: string[]

  private readonly permissionsSet: Set<string>
  private readonly permissionsArray: string[]

  constructor(roles: string[], perms: Permission[]) {
    this.rolesArray = roles
    this.permissionsArray = perms.map(p => p.permission)

    this.rolesSet = new Set(this.rolesArray)
    this.permissionsSet = new Set(this.permissionsArray)
  }

  public get roles(): string[] {
    return this.rolesArray
  }

  public get permissions(): string[] {
    return this.permissionsArray
  }

  public hasRole = (role: string): boolean => {
    return this.rolesSet.has(role)
  }

  public hasPermission = (permission: string): boolean => {
    return this.permissionsSet.has(permission)
  }
}

interface Session {
  expiresOnUTC: number
  userId: ID
  roles: string[]
  permissions: string[]
  hasRole(role: string): boolean
  hasPermission(permission: string): boolean
}

interface AuthContext {
  accessToken: string | undefined
  session: Session | undefined
  setAccessToken: (accessToken: string | undefined) => void
  setSession: (session: Session | undefined) => void
  isCheckingAuth: MutableRefObject<boolean>
}

interface AuthWrapperProps {
  children: React.ReactNode
}

const Context = createContext<AuthContext>(undefined as any)

export const AuthProvider = (props: AuthWrapperProps) => {
  const [accessToken, setAccessToken] = useState<string | undefined>()
  const [session, setSession] = useState<Session | undefined>()
  const isCheckingAuth = useRef<boolean>(false)

  return (
      <Context.Provider
          value={{
            accessToken,
            session,
            setAccessToken,
            setSession,
            isCheckingAuth,
          }}
      >
        {props.children}
      </Context.Provider>
  )
}

export const useAuth = () => {
  const context = useContext(Context)

  const login = async (email: string, password: string): Promise<boolean> => {
    const response = await fetch('/api/auth/login', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ email, password }),
    })

    if (response.ok) {
      const responseJson = await response.json()
      const parsedToken = parseJwt(responseJson.access_token) as AccessTokenClaims
      const permissions = new Permissions(parsedToken.roles, parsedToken.permissions)
      context.setAccessToken(responseJson.access_token)
      context.setSession({
        userId: parsedToken.sub,
        expiresOnUTC: parsedToken.exp,
        roles: permissions.roles,
        permissions: permissions.permissions,
        hasPermission: permissions.hasPermission,
        hasRole: permissions.hasRole,
      })
      return true
    } else {
      context.setAccessToken(undefined)
      context.setSession(undefined)
      return false
    }
  }

  const logout = async (): Promise<boolean> => {
    const response = await fetch('/api/auth/logout', {
      method: 'POST',
    })

    if (response.ok) {
      context.setAccessToken(undefined)
      context.setSession(undefined)
      return true
    } else {
      return false
    }
  }

  return {
    accessToken: context.accessToken,
    session: context.session,
    isCheckingAuth: context.isCheckingAuth,
    isAuthenticated: !!context.accessToken,
    login,
    logout,
  }
}

export const useAuthCheck = () => {
  const context = useContext(Context)
  const { isCheckingAuth } = context

  const refreshIfNecessary = useCallback(async () => {
    if (isCheckingAuth.current) {
      return
    }
    isCheckingAuth.current = true

    const isExpiringSoon = () => {
      if (context.session?.expiresOnUTC) {
        const expireTimeMS = context.session.expiresOnUTC * 1000
        const currentTimeMS = Date.now()

        return expireTimeMS - currentTimeMS <= REMAINING_TOKEN_EXPIRY_TIME_ALLOWED
      }

      return true
    }

    if (!context.accessToken || isExpiringSoon()) {
      // console.log('Restoring session')
      const response = await fetch('/api/auth/refresh', {
        method: 'POST',
      })

      if (response.ok) {
        const responseJson = await response.json()
        const parsedToken = parseJwt(responseJson.access_token) as AccessTokenClaims
        const permissions = new Permissions(parsedToken.roles, parsedToken.permissions)

        context.setAccessToken(responseJson.access_token)
        context.setSession({
          userId: parsedToken.sub,
          expiresOnUTC: parsedToken.exp,
          roles: permissions.roles,
          permissions: permissions.permissions,
          hasRole: permissions.hasRole,
          hasPermission: permissions.hasPermission,
        })
      } else {
        context.setAccessToken(undefined)
        context.setSession(undefined)
      }
    } else {
      // console.log(`${context.accessToken ? 'access token' : ''} ${isExpiringSoon() ? ' is not expiring' : ''}`)
    }

    isCheckingAuth.current = false
  }, [context.accessToken, context.session])

  useEffect(() => {
    refreshIfNecessary()
    let intervalId: NodeJS.Timeout | undefined = undefined

    if (context.accessToken) {
      // if the access token is set, we want to check its expiry on some interval
      intervalId = setInterval(() => {
        refreshIfNecessary()
      }, MILLISECONDS_UNTIL_EXPIRY_CHECK)
    }
    return () => {
      if (intervalId) clearInterval(intervalId)
    }
  }, [refreshIfNecessary])
}

// https://stackoverflow.com/a/38552302
const parseJwt = (token: string) => {
  const base64Url = token.split('.')[1]
  const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/')
  const jsonPayload = decodeURIComponent(
      atob(base64)
          .split('')
          .map(function (c) {
            return '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2)
          })
          .join('')
  )

  return JSON.parse(jsonPayload)
}
