///////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////// DEV BOX
///////////////////////////////////////////////////////////////////////////////

/* require() this file in development mode to enable development hints */

import React, { useEffect, useRef, useState } from 'react'
import ReactDOM from 'react-dom'
import {
  QueryClient,
  QueryClientProvider,
  useMutation,
  useQuery,
} from 'react-query'

const warning = (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    fill="currentColor"
    height="24px"
    width="24px"
    viewBox="0 0 24 24"
  >
    <path d="M12 2.829l9.172 9.171-9.172 9.171-9.172-9.171 9.172-9.171zm0-2.829l-12 12 12 12 12-12-12-12zm-1 7h2v6h-2v-6zm1 10.25c-.69 0-1.25-.56-1.25-1.25s.56-1.25 1.25-1.25 1.25.56 1.25 1.25-.56 1.25-1.25 1.25z" />
  </svg>
)
const close = (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    fill="currentColor"
    width="24"
    height="24"
    viewBox="0 0 24 24"
  >
    <path d="M12 0c-6.627 0-12 5.373-12 12s5.373 12 12 12 12-5.373 12-12-5.373-12-12-12zm4.151 17.943l-4.143-4.102-4.117 4.159-1.833-1.833 4.104-4.157-4.162-4.119 1.833-1.833 4.155 4.102 4.106-4.16 1.849 1.849-4.1 4.141 4.157 4.104-1.849 1.849z" />
  </svg>
)

interface Action {
  label: string
  fn: () => void
}

const DevBoxItem = (props: {
  actions?: Action[]
  children: React.ReactNode
}) => {
  return (
    <div style={{ display: 'flex', flexFlow: 'column nowrap' }}>
      <div>{props.children}</div>
      <div style={{ display: 'flex' }}>
        {props.actions?.map((action) => (
          <button onClick={() => action.fn()}>{action.label}</button>
        ))}
      </div>
    </div>
  )
}

const DevBox = () => {
  const [display, setDisplay] = useState<boolean>(true)

  const healthQuery = useQuery(
    'health',
    () => fetch('/api/development/health').then((r) => r.json()),
    {
      onSuccess: () => queryClient.invalidateQueries('migrations'),
    }
  )
  const dbMigrationQuery = useQuery('migrations', () =>
    fetch('/api/development/db/needs-migration').then((r) => r.json())
  )
  const dbMigrateMutation = useMutation(
    () => fetch('/api/development/db/migrate').then((r) => r.json()),
    {
      onSuccess: () => queryClient.invalidateQueries('migrations'),
    }
  )
  // const hasSystemRoleQuery = useQuery('role-check', () => fetch('/api/development/auth/has-system-role', { headers: { Authorization: `${auth.accessToken}` } }).then(r => r.json()))
  // const addSystemRoleMutation = useMutation(() => fetch('/api/development/auth/add-system-role', { headers: { Authorization: `${auth.accessToken}` } }).then(r => r.json()), {
  //   onSuccess: () => queryClient.invalidateQueries('role-check')
  // })

  const isFetching =
    healthQuery.isFetching ||
    dbMigrationQuery.isFetching /*|| hasSystemRoleQuery.isFetching*/
  const shouldDisplay = useRef<boolean>(true)

  shouldDisplay.current = !!(
    (
      healthQuery.isError ||
      dbMigrationQuery.isError ||
      dbMigrationQuery.data
    ) /*|| hasSystemRoleQuery.data === false*/
  )

  // useEffect(() => {
  //   hasSystemRoleQuery.refetch()
  // }, [auth.isAuthenticated, auth.accessToken])

  useEffect(() => {
    setDisplay(shouldDisplay.current)
  }, [
    shouldDisplay,
    healthQuery.isError,
    healthQuery.isFetching,
    healthQuery.data,
    dbMigrationQuery.isError,
    dbMigrationQuery.data,
    // hasSystemRoleQuery.data
  ])

  return (
    <div
      style={{
        display: display && shouldDisplay.current ? 'block' : 'none',
        margin: '12px',
        padding: '12px',
        backgroundColor: 'white',
      }}
    >
      <div style={{ position: 'absolute', height: '0px', width: '0px' }}>
        <div style={{ position: 'relative', left: '-36px', top: '-36px' }}>
          <a
            style={{
              textDecoration: 'none',
              boxShadow: '0 4px 4px #00000066',
              color: '#fe4646',
              backgroundColor: '#FFF',
              borderRadius: '50%',
              display: 'inline-flex',
            }}
            href="/"
            onClick={(e) => {
              e.preventDefault()
              e.stopPropagation()
              setDisplay(false)
            }}
          >
            {close}
          </a>
        </div>
      </div>
      <div
        style={{
          display: 'flex',
          marginBottom: '12px',
          flexFlow: 'row nowrap',
        }}
      >
        <span
          style={{
            fontSize: '24px',
            fontWeight: 'bolder',
            flex: 1,
            color: 'rgba(207,144,58,1)',
            display: 'flex',
            overflow: 'hidden',
            height: '30px',
          }}
        >
          <div style={{ flex: 1 }}>DEVBOX</div>{' '}
          {isFetching && <div className={`${DEVBOX_ID}-loader`}></div>}
        </span>
        {!isFetching && (
          <span style={{ color: 'rgba(207,144,58,1)' }}>{warning}</span>
        )}
      </div>
      {(healthQuery.isError || healthQuery.isFetching) && (
        <DevBoxItem
          actions={[
            {
              label: 'Retry',
              fn: () => queryClient.invalidateQueries('health'),
            },
          ]}
        >
          <span style={{ color: 'hotpink' }}>WARN!</span>&nbsp;
          {healthQuery.isError && 'The backend is not reachable.'}
          {healthQuery.isFetching && 'Connecting to backend...'}
        </DevBoxItem>
      )}
      {!healthQuery.isError && (
        <>
          {!dbMigrationQuery.isError && dbMigrationQuery.data && (
            <DevBoxItem
              actions={[
                {
                  label: dbMigrateMutation.isLoading
                    ? 'running...'
                    : 'Run Migrations',
                  fn: () => {
                    if (dbMigrateMutation.isLoading) return
                    dbMigrateMutation.mutate()
                  },
                },
              ]}
            >
              <span style={{ color: 'hotpink' }}>WARN!</span>&nbsp;Some
              migrations are pending!
            </DevBoxItem>
          )}
          {/* {hasSystemRoleQuery.data === false && <DevBoxItem actions={[{
          label: addSystemRoleMutation.isLoading ? 'running...' : 'Add `developer` role to current user',
          fn: () => {
            if (addSystemRoleMutation.isLoading) return
            addSystemRoleMutation.mutate()
          }
        }]}>
          <span style={{color: 'deepskyblue'}}>INFO!</span>&nbsp;The logged in user doesn't have the developer role. You may not have access to administrative developer tasks.
        </DevBoxItem>} */}
        </>
      )}
    </div>
  )
}

const DEVBOX_ID = 'create-rust-app-devbox'
const existingDevBoxes = document.getElementsByClassName(DEVBOX_ID)
if (existingDevBoxes.length > 0) {
  for (let i = 0; i < existingDevBoxes.length; i++) {
    const el = existingDevBoxes[i]
    document.body.removeChild(el)
  }
}

const devBox = document.createElement('div')
devBox.id = DEVBOX_ID
devBox.className = DEVBOX_ID
const devBoxStyle = document.createElement('style')
devBoxStyle.innerHTML = `
.${DEVBOX_ID} {
  position: fixed;
  height: 0px;
  width: 0px;
  right: 12px;
  bottom: 12px;
  font-size: 16px;
}
.${DEVBOX_ID} > div {
  position: absolute;
  bottom: 0;
  right: 0;
  transition-property: all;
	transition-duration: .5s;
  font-family: monospace;
	transition-timing-function: cubic-bezier(1, 1, 1, 1);
  color: black;
  background: rgb(255,69,69);
  background: linear-gradient(345deg, rgba(255,69,69,1) 0%, rgba(207,144,58,1) 100%);
  min-width: 400px;
  box-shadow: 0 4px 4px #00000044;
}

.${DEVBOX_ID}-loader,
.${DEVBOX_ID}-loader:before,
.${DEVBOX_ID}-loader:after {
  background: #e86840;
  -webkit-animation: ${DEVBOX_ID}-loader-animation 1s infinite ease-in-out;
  animation: ${DEVBOX_ID}-loader-animation 1s infinite ease-in-out;
  width: 1em;
  height: 1em;
}
.${DEVBOX_ID}-loader {
  color: #e86840;
  text-indent: -9999em;
  margin: 21px 30px;
  position: relative;
  font-size: 11px;
  -webkit-transform: translateZ(0);
  -ms-transform: translateZ(0);
  transform: translateZ(0);
  -webkit-animation-delay: -0.16s;
  animation-delay: -0.16s;
}
.${DEVBOX_ID}-loader:before,
.${DEVBOX_ID}-loader:after {
  position: absolute;
  top: 0;
  content: '';
}
.${DEVBOX_ID}-loader:before {
  left: -1.5em;
  -webkit-animation-delay: -0.32s;
  animation-delay: -0.32s;
}
.${DEVBOX_ID}-loader:after {
  left: 1.5em;
}
@-webkit-keyframes ${DEVBOX_ID}-loader-animation {
  0%, 80%, 100% {
    box-shadow: 0 0;
    height: 4em;
  }
  40% {
    box-shadow: 0 -2em;
    height: 5em;
  }
}
@keyframes ${DEVBOX_ID}-loader-animation {
  0%, 80%, 100% {
    box-shadow: 0 0;
    height: 4em;
  }
  40% {
    box-shadow: 0 -2em;
    height: 5em;
  }
}
`
devBox.appendChild(devBoxStyle)
devBox.appendChild(document.createElement('div'))
document.body.appendChild(devBox)

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
})

ReactDOM.render(
  <>
    <QueryClientProvider client={queryClient}>
      <DevBox />
    </QueryClientProvider>
  </>,
  devBox.children[1]
)

export {}
