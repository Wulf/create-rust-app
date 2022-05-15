import React, { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'

export const AccountPage = () => {
  const auth = useAuth()
  const navigate = useNavigate()

  const [processing, setProcessing] = useState<boolean>(false)
  const [originalPassword, setOriginalPassword] = useState<string>('')
  const [password, setPassword] = useState<string>('')

  const [page, setPage] = useState<number>(0)
  const [pageSize, setPageSize] = useState<number>(10)

  const [isFetchingSessions, setFetchingSessions] = useState<boolean>(false)
  const [sessions, setSessions] = useState<UserSessionResponse>({
    sessions: [],
    num_pages: 1,
  })

  const [isDeleting, setDeleting] = useState<boolean>(false)
  const deleteSession = async (id: number) => {
    setDeleting(true)

    const response = await fetch(`/api/auth/sessions/${id}`, {
      method: 'DELETE',
      headers: {
        Authorization: `${auth.accessToken}`,
      },
    })

    if (response.ok) {
      if (sessions.sessions.length === 1 && page !== 0) {
        setPage(page - 1)
      }
      await fetchSessions()
    }

    setDeleting(false)
  }

  const deleteAllSessions = async () => {
    setDeleting(true)

    const response = await fetch(`/api/auth/sessions`, {
      method: 'DELETE',
      headers: {
        Authorization: `${auth.accessToken}`,
      },
    })

    if (response.ok) {
      setPage(0)
      await fetchSessions()
    }

    setDeleting(false)
  }

  const fetchSessions = async () => {
    setFetchingSessions(true)

    if (!auth.isAuthenticated) {
      setSessions({ sessions: [], num_pages: 1 })
      setFetchingSessions(false)
      return
    }

    const sessions = await (
      await fetch(`/api/auth/sessions?page=${page}&page_size=${pageSize}`, {
        method: 'GET',
        headers: {
          Authorization: `${auth.accessToken}`,
        },
      })
    ).json()

    setSessions(sessions)
    setFetchingSessions(false)
  }

  useEffect(() => {
    fetchSessions()
  }, [auth.isAuthenticated, page, pageSize])

  const changePassword = async () => {
    setProcessing(true)
    const response = await (
      await fetch('/api/auth/change', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `${auth.accessToken}`,
        },
        body: JSON.stringify({
          old_password: originalPassword,
          new_password: password,
        }),
      })
    ).json()
    console.log(response)
    setOriginalPassword('')
    setPassword('')
    setProcessing(false)
  }

  return (
    <div style={{ textAlign: 'left' }}>
      <h1>Account</h1>
      <br />
      {auth.isAuthenticated && (
        <div>
          User # {auth.session?.userId}
          <div className="Form" style={{ textAlign: 'left' }}>
            <h1>Permissions</h1>
            <pre>
              {!auth.session && (
                <div>Error: No auth session present.</div>
              )}
              {auth.session?.permissions?.map((perm) => {
                return <div>{JSON.stringify(perm)}</div>
              })}
              {auth.session?.permissions?.length === 0 && (
                <div>No permissions granted.</div>
              )}
            </pre>
          </div>
          <div className="Form" style={{ textAlign: 'left' }}>
            <h1>Change password</h1>
            <br />
            <div style={{ display: 'flex', flexFlow: 'column' }}>
              <label>Original Password</label>
              <input
                type="password"
                value={originalPassword}
                onChange={(e) => setOriginalPassword(e.target.value)}
              />
            </div>
            <div style={{ display: 'flex', flexFlow: 'column' }}>
              <label>New Password</label>
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
            </div>
            <div style={{ display: 'flex', flexFlow: 'column' }}>
              <button disabled={processing} onClick={changePassword}>
                Change Password
              </button>
            </div>
          </div>
          <div>
            <h1>Sessions</h1>
            <button disabled={isDeleting} onClick={() => deleteAllSessions()}>
              Delete All
            </button>
            {sessions.sessions.map((session) => (
              <div>
                {JSON.stringify(session, null, 2)}
                <button
                  disabled={isDeleting}
                  onClick={() => deleteSession(session.id)}
                >
                  Delete
                </button>
              </div>
            ))}
            {isFetchingSessions && <div>Fetching sessions...</div>}
            <div>
              <button
                disabled={page <= 0}
                onClick={() => setPage(page - 1)}
              >{`<<`}</button>
              <span>
                {page + 1} / {sessions.num_pages}
              </span>
              <button
                disabled={page + 1 >= sessions.num_pages}
                onClick={() => setPage(page + 1)}
              >{`>>`}</button>
            </div>
          </div>
        </div>
      )}
      {!auth.isAuthenticated && (
        <div>
          <a href="#" onClick={() => navigate('/login')}>
            Login to view your account detials
          </a>
        </div>
      )}
    </div>
  )
}
