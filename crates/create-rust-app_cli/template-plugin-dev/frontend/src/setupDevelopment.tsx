///////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////// DEV BOX
///////////////////////////////////////////////////////////////////////////////

/* require() this file in development mode to enable development hints */

import { useCallback, useEffect, useRef, useState } from 'react'
import ReactDOM from 'react-dom/client'
import ReconnectingWebsocket from 'reconnecting-websocket'
import Ansi from 'ansi-to-react'

interface Migration {
  name: string
  status: 'Applied' | 'AppliedButMissingLocally' | 'Pending' | 'Unknown'
  version: string
}

const useWebsocketConnection = () => {
  const [features, setFeaturesList] = useState<Set<String>>(new Set())
  const [backendCompileState, setBackendCompileState] = useState(true)
  const [backendCompilingState, setBackendCompilingState] = useState(false)
  const [backendRestartingState, setBackendRestartingState] = useState(false)
  const [backendCompilerMessages, setBackendCompilerMessages] = useState<CompilerMessage[]>([])
  const [backendOnlineState, setBackendOnlineState] = useState(true)
  const [wsConnected, setWsConnected] = useState(false)
  const [viteStatus, setViteStatus] = useState(true)
  const [migrationsPending, setMigrationPending] = useState(false)
  const [migrating, setMigrating] = useState(false)
  const [migrationSuccess, setMigrationSuccess] = useState(true)
  const [migrationError, setMigrationError] = useState()
  const [migrations, setMigrations] = useState<Migration[]>([])

  const wsRef = useRef<ReconnectingWebsocket>()

  useEffect(() => {
    const connect = () => {
      if (wsRef.current) {
        wsRef.current.close()
      }

      const ws = new ReconnectingWebsocket(`ws://localhost:${import.meta.env.DEV_SERVER_PORT}/ws`);

      wsRef.current = ws;

      ws.onopen = () => { setWsConnected(true) }
      ws.onclose = () => { setWsConnected(false) }
      ws.onerror = (err) => {
        console.error('create-rust-app:plugin_dev: Websocket error', err)
        ws.close()
      }
      ws.onmessage = (payload) => {
        const data = JSON.parse(payload.data)

        // console.log("DEV MESSAGE", data)

        if (data.type === 'featuresList') {
          setFeaturesList(new Set(data.features))
        } else if (data.type === 'backendCompiling') {
          setBackendCompilingState(data.compiling)
        } else if (data.type === 'compileStatus') {
          setBackendCompileState(data.compiled)
        } else if (data.type === 'backendStatus') {
          setBackendOnlineState(data.status)
        } else if (data.type === 'compilerMessages') {
          setBackendCompilerMessages(data.messages)
        } else if (data.type === 'viteStatus') {
          setViteStatus(data.status)
        } else if (data.type === 'backendRestarting') {
          setBackendRestartingState(data.status)
        } else if (data.type === 'migrationsPending') {
          setMigrationPending(data.status)
          setMigrations(data.migrations)
        } else if (data.type === 'migrateResponse') {
          setMigrationSuccess(data.status)
          setMigrationError(data.error)
          setMigrating(false)
        }
      }
    }

    connect();

    return () => {
      if (wsRef.current) {
        wsRef.current.close()
      }
    }
  }, [])

  interface RequestBase {
    type: string
  }

  interface OpenRequest extends RequestBase { type: 'open'; file: string }
  interface MigrateRequest extends RequestBase { type: 'migrate' }

  type WebsocketRequest =
    | MigrateRequest
    | OpenRequest

  const send = useCallback((message: WebsocketRequest) => {
    let msg = ''

    if (message.type === 'open') {
      msg = `open:${message.file}`
    } else if (message.type === 'migrate') {
      msg = `migrate`
      setMigrating(true)
    }

    if (msg.length === 0) return

    wsRef.current?.send(msg)
  }, [wsRef.current]);

  return {
    features,
    backendCompileState,
    backendCompilingState,
    backendRestartingState,
    backendCompilerMessages,
    backendOnlineState,
    wsConnected,
    viteStatus,
    migrationsPending,
    migrationSuccess,
    migrationError,
    migrating,
    migrations,
    send
  }
}


const DevBox = () => {
  const state = useWebsocketConnection()
  const [isWeirdMigrationState, setWeirdMigrationState] = useState(false)
  const send = state.send

  const shouldDisplay = state.backendCompileState !== true
    || state.backendRestartingState === true
    || state.backendOnlineState !== true
    || state.wsConnected === false
    || state.backendCompilingState === true
    || state.viteStatus === false
    || state.migrationsPending === true
    || state.migrationSuccess === false
    || state.migrating === true
    || isWeirdMigrationState

  useEffect(() => {
    if (state.migrations?.find(m => m.status === 'AppliedButMissingLocally')) {
      setWeirdMigrationState(true)
    }
  }, [state.migrations])
    
  const compilerErrors = state.backendCompilerMessages
    ?.filter(m => m?.message?.level === "error")

  return (
    <div
      style={{
        display: shouldDisplay ? 'block' : 'none',
        margin: '1px',
        padding: '4px',
        backgroundColor: 'black',
        color: 'white',
        overflow: 'scroll',
        maxHeight: 'calc(100vh - 200px)',
        zIndex: 9999999999999999
      }}
    >
      {/* 
        DEV SERVER CONNECTION
      */}
      {state.wsConnected === false && <div>Connecting to development server...</div>}
      {state.viteStatus === false && <div>Vite dev server is down... <a href="#" onClick={(e) => {
        e.preventDefault()
        window.location.reload()
      }}>refresh?</a></div>}
      
      {/* 
        MIGRATIONS
      */}
      {isWeirdMigrationState && <div><span style={{ color: 'tomato' }}>Migrations are in a strange state...</span> <a href="#" onClick={() => setWeirdMigrationState(false)}>ok, thanks!</a></div>}
      {state.migrationSuccess === false && <div>⚠️ Migration failed ({state.migrationError}).</div>}
      {state.migrationsPending === true &&
        <div>
          <span style={{ color: 'goldenrod' }}>Migrations are pending...</span> <a href="#" onClick={e => {
            e.preventDefault()
            if (!state.migrating) {
              send({ type: 'migrate' })
            }
          }}>{state.migrating === false ? 'migrate?' : 'migrating...'}</a>
        </div>}

        {(state.migrationsPending === true || isWeirdMigrationState) && <div>{state.migrations?.sort((a, b) => b.version.localeCompare(a.version)).map(m => (
          <div style={{ display: 'flex', color: m.status === 'Applied' ? 'white' : (m.status === 'Pending' ? 'goldenrod' : 'tomato') }}>
            <span>
              {m.name}
            </span>
            <span style={{ flex: 1, border: '0px dotted white', borderBottomWidth: '1px' }} />
            <span>
              {m.status}
            </span>
          </div>
        ))}
      </div>}
      
      {/*
        COMPILATION
      */}
      {state.backendCompileState === false && <div>The last compilation failed.</div>}
      {state.backendOnlineState === false && <div>The backend is offline!</div>}
      {state.backendCompilingState && <div>🔨 Compiling backend...</div>}
      {state.backendRestartingState === true && <div>♻️ Restarting backend...</div>}
      {compilerErrors.length > 0 &&
        <pre>
          {compilerErrors.map(m => (
            <CompilerMessage
              onOpenFile={(path) => {
                send({ type: 'open', file: path })
              }}
              m={m}
            />
          ))}
        </pre>
      }
    </div>
  )
}

interface CompilerMessage {
  package_id: {
    repr: string
  },
  target: {
    name: string,
    kind: Array<"bin" | "example" | "test" | "bench" | "lib" | "custom-build">,
    crate_types: Array<string>,
    required_features: Array<string>,
    src_path: string,
    edition: string,
    doctest: boolean,
    test: boolean,
    doc: boolean,
  },
  message: {
    message: string,
    code?: {
      code?: string
      explanation?: string
    },
    /// "error: internal compiler error", "error", "warning", "note", "help"
    level: "error: internal compiler error" | "error" | "warning" | "note" | "help" | "failure-note",
    /// A list of source code spans this diagnostic is associated with.
    spans: Array<{
      file_name: string,
      byte_start: number,
      byte_end: number,
      line_start: number,
      line_end: number,
      column_start: number,
      column_end: number,
      is_primary: boolean,
      text: Array<{
        text: string,
        highlight_start: number,
        highlight_end: number,
      }>,
      label?: string,
      suggested_replacement?: string,
      suggestion_applicability?: string,
      expansion?: any,
    }>,
    children: Array<any>,
    rendered?: string, // The message as rustc would render it 
  },
}

interface CompilerMessageProps {
  m: CompilerMessage
  onOpenFile: (f: string) => void
}
const CompilerMessage = ({ m, onOpenFile }: CompilerMessageProps) => {
  const primarySpan = m.message.spans.filter(s => s.is_primary)[0];

  if (!primarySpan) return <></> // <div>no primary span, <pre>{JSON.stringify(m)}</pre></div>

  return <pre
    style={{ overflow: 'hidden' }}
  >
    <div>
      <strong><span className={m.message.level}>{m.message.level}</span>: {m.message.message}</strong>
    </div>
    <div>
      <span className="guide">&nbsp;&nbsp;--&gt;&nbsp;</span>
      <span className="file" onClick={(e) => {
        e.preventDefault()
        onOpenFile(`${primarySpan.file_name}`)
      }}
      >
        {primarySpan.file_name} {/* the following svg was taken from https://iconmonstr.com/cursor-2-svg/ and does not belong to the create-rust-app project */}<svg xmlns="http://www.w3.org/2000/svg" fill="white" width="12" height="12" viewBox="0 0 24 24" fill-rule="evenodd" clip-rule="evenodd"><path d="M14 4h-13v18h20v-11h1v12h-22v-20h14v1zm10 5h-1v-6.293l-11.646 11.647-.708-.708 11.647-11.646h-6.293v-1h8v8z" /></svg>
      </span>
    </div>
    <div style={{ whiteSpace: 'pre' }}>
      <Ansi>{m.message.rendered?.split('\n').slice(2).join('\n')}</Ansi>
    </div>
  </pre>
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
  bottom: 0;
  font-size: 12px;
}
.${DEVBOX_ID} > div {
  position: absolute;
  bottom: 0;
  width: 100vw;
  transition-property: all;
	transition-duration: .5s;
  font-family: monospace;
	transition-timing-function: cubic-bezier(1, 1, 1, 1);
  color: black;
  background: rgb(255,69,69);
  background: linear-gradient(0deg, rgba(0,0,0,1) 0%, rgba(255,255,255,1) 100%);
  box-shadow: 0 0px 8px #00000088;
}

.${DEVBOX_ID} a {
  color: skyblue;
}

.file:hover {
  text-decoration: underline;
  cursor: pointer;
  background-color: gray;
}

.guide {
  color: rgb(106,112,246);
}

.warning {
  color: yellow;
}

.error {
  color: rgb(237,118,108);
}

pre {
  white-space: pre-line;
  font-family: monospace;
}
`
devBox.appendChild(devBoxStyle)
devBox.appendChild(document.createElement('div'))
document.body.appendChild(devBox)

ReactDOM.createRoot(devBox.children[1]).render(<DevBox />)
