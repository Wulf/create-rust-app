import React, { useState } from 'react'
import ReactDOM from 'react-dom'
import { QueryClient, QueryClientProvider, useQuery } from 'react-query'

const fetchQuery = (query) => fetch('/api/development/db/query', { method: 'POST', body: JSON.stringify({query: query}), headers: { 'Content-Type': 'application/json' } }).then(r => r.json())

const useTableCount = (tableName: string) => {
  return useQuery(`table-${tableName}-count`, () => fetchQuery(`SELECT COUNT(*) FROM ${tableName}`).then(r => r[0]['count']))
}

const TableLink = (props: {name: string, onClick: () => void}) => {
  const countQuery = useTableCount(props.name)
  
  return <>
    <button onClick={props.onClick} className="flex-1 truncate text-left hover:underline text-blue-500 hover:text-blue-700">{props.name}</button> ({typeof countQuery.data === 'number' ? countQuery.data : countQuery.isFetching ? '…' : '-'})
  </>
}

interface TableColumn {
  column_name: string,
  data_type: string
}

const RowView = (props: {columns: TableColumn[], table: string }) => {
  const rowsQuery = useQuery<Record<string, any>>(`table-${props.table}-rows`, () => fetchQuery(`SELECT * FROM ${props.table} LIMIT 50`))
  
  return rowsQuery.data && rowsQuery.data.map(row => <tr className="odd:bg-grey-400">
    {props.columns.map(column => <td className="max-w-xs truncate">{JSON.stringify(row[column.column_name])}</td>)}
  </tr>) || null
}

const TableView = (props: {name: string}) => {
  const tableCountQuery = useTableCount(props.name)
  const viewQuery = useQuery<TableColumn[]>(`table-${props.name}-columns`, () => fetchQuery(`
    SELECT *
    FROM information_schema.columns
    WHERE table_name   = '${props.name}'
  `))

  return <div>
    <h1 className="font-bold text-xl">{props.name} {viewQuery.isFetching && <span className="text-grey-500 text-xs">(Loading...)</span>}</h1>
    <table className="table-auto w-full border-grey-500 border-2">
      <thead>
        <tr className="text-left align-top border-b-2">
          {viewQuery.data && viewQuery.data.map(col => 
            <th className="p-2">{col.column_name}<br/><span className="text-xs">({col.data_type})</span></th>
          )}
        </tr>
      </thead>
      <tbody>
        {viewQuery.data && <RowView columns={viewQuery.data || []} table={props.name} />}
      </tbody>
    </table>
    <div className="flex">
      <div className="flex-1"></div>
      <div>1 - {Math.min(typeof tableCountQuery.data === 'number' ? tableCountQuery.data : 0, 50)} of {typeof tableCountQuery.data === 'number' ? tableCountQuery.data : (tableCountQuery.isFetching ? '…' : '?')}</div>
    </div>
  </div>
}

const AdminPage = () => {
  /*
    SELECT tablename AS name, (SELECT COUNT(*) FROM `tablename`) AS count FROM (SELECT * FROM pg_catalog.pg_tables WHERE schemaname != 'pg_catalog' AND schemaname != 'information_schema')
  */

  const tableQuery = useQuery<{name: string}[]>('tables', () => fetchQuery(`SELECT tablename AS name FROM pg_catalog.pg_tables WHERE schemaname != 'pg_catalog' AND schemaname != 'information_schema'`))

  const [selectedTable, setSelectedTable] = useState<string | undefined>(undefined)
  
  return (
    <div className="flex h-full flex flex-col">
      <a href="admin" className="p-4 text-blue-500 hover:underline hover:text-blue-700">Admin Portal <span className="text-xs">Create Rust App</span></a>
      <div className="flex-1 flex">
        <div className="p-4 w-50 border-r-2 border-grey-50">
          <h2 className="text-xs">tables {tableQuery.isFetching && <span className="text-gray-500">(Loading...)</span>}</h2>
          <ul className="flex-col">
            {tableQuery.data && tableQuery.data.map(table =>
              <li className="flex">
                <TableLink name={table.name} onClick={() => setSelectedTable(table.name)} />
              </li>
            )}
            
          </ul>
        </div>
        <div className="p-4 flex-1">
          {!selectedTable && <div className="text-gray-500">
            No table selected.
          </div>}
          {selectedTable && <TableView name={selectedTable}/>}
        </div>
      </div>
    </div>
  )
}

const queryClient = new QueryClient()
ReactDOM.render(<QueryClientProvider client={queryClient}>
  <AdminPage />
</QueryClientProvider>, document.getElementById('app'))
