import React, { useState } from 'react'
import ReactDOM from 'react-dom/client'

import './index.css'

import { DatabasePage } from './DatabasePage'
import { HomePage } from './HomePage'
import { MailPage } from './MailPage'
import { QueryClient, QueryClientProvider } from 'react-query'

export type Pages = 'database' | 'email' | 'home'
const Index = () => {
  const [page, setPage] = useState<Pages>('email')
  return (
    <div className="flex h-full w-full flex-col text-xs">
      <nav className="select-none shadow-md">
        <span className="border-r py-2 px-8 text-slate-500">Create Rust App</span>
        <button className={`border-r p-2 ${page === 'home' && 'bg-slate-200'}`} onClick={() => setPage('home')}>
          home
        </button>
        <button className={`border-r p-2 ${page === 'database' && 'bg-slate-200'}`} onClick={() => setPage('database')}>
          database (alpha release)
        </button>
        <button className={`border-r p-2 ${page === 'email' && 'bg-slate-200'}`} onClick={() => setPage('email')}>
          email (alpha release)
        </button>
      </nav>
      <div className={'flex-1'}>
        {page === 'home' && <HomePage setPage={setPage} />}
        {page === 'database' && <DatabasePage />}
        {page === 'email' && <MailPage />}
      </div>
    </div>
  )
}

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
    },
  },
})

const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement)
root.render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <Index />
    </QueryClientProvider>
  </React.StrictMode>
)
