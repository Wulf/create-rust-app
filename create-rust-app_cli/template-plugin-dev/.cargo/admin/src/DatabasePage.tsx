import React, {useState} from 'react'
import {useQuery, useQueryClient} from 'react-query'

const fetchQuery = (query, raw = false) => fetch('/api/development/db/query', {
  method: 'POST',
  body: JSON.stringify({query: query, raw}),
  headers: {'Content-Type': 'application/json'}
}).then(r => r.json())


const useTableCount = (tableName: string) => {
  return useQuery(
    ['table', tableName, 'count'],
    () => fetchQuery(`
        SELECT COUNT(*)
        FROM ${tableName}
    `).then(r => r[0]['count']))
}

const TableLink = (props: { name: string, onClick: () => void }) => {
  const countQuery = useTableCount(props.name)

  return <>
    <button onClick={props.onClick}
            className="flex-1 truncate text-left hover:underline text-blue-500 hover:text-blue-700">{props.name}</button>
    ({typeof countQuery.data === 'number' ? countQuery.data : countQuery.isFetching ? '…' : '-'})
  </>
}

interface TableColumn {
  table_name: string
  column_name: string
  data_type: string
  column_default: string
  is_nullable: "NO" | "YES"
  ordinal_position: number
  udt_name: string // "timestamptz"
  is_identity: "NO" | "YES"
  is_generated: "ALWAYS" | "NEVER"
  is_updatable: "NO" | "YES"
}

const prepareDbColumn = (dbColumn: string) => `"${dbColumn}"`
const prepareDbValue = (dbInput: any): string => {
  if (typeof dbInput === 'string') {
    return `'${dbInput.replace(/'/g, "''")}'`
  } else if(typeof dbInput === 'number') {
    return `${dbInput}`
  } else if(typeof dbInput === 'boolean') {
    return dbInput ? 'true' : 'false'
  } else {
    return dbInput
  }
}

/**
 * Confirms whether a row exists and that there is only one copy of the row
 */
const confirmTargetRow = async (table: string, columns: TableColumn[], row: any): Promise<boolean> => {
  const sql = `SELECT COUNT(*) as count ` +
    `FROM ${table} ` +
    `WHERE ${columns.map(col => `${prepareDbColumn(col.column_name)} = ${prepareDbValue(row[col.column_name])}`).join(' AND ')} `
  console.log('Getting target row', sql)
  const affectedRows = await fetchQuery(sql)
  const affectedRowsCount = affectedRows[0].count
  const confirmed = affectedRowsCount === 1
  console.log(confirmed ? '(success) Found' : (affectedRowsCount > 1 ? '(error) found too many!' : '(error) not found'))
  return confirmed
}

const TableRows = (props: { columns: TableColumn[], offset: number, table: string, limit: number }) => {
  const queryClient = useQueryClient()
  const [selectedRowForEditing, setRowForEditing] = useState<any>(null)
  const [updateValues, setUpdateValues] = useState<any>({})

  const rowsQuery = useQuery<Record<string, any>>(
    ['table', props.table, 'rows', {offset: props.offset, limit: props.limit}],
    () => fetchQuery(`
        SELECT *
        FROM ${props.table} LIMIT ${props.limit}
        OFFSET ${props.offset}
    `))

  const deleteRow = async (row: any) => {
    if (await confirmTargetRow(props.table, props.columns, row)) {
      if (!window.confirm('Delete?')) return
      await fetchQuery(`
          DELETE
          FROM ${props.table}
          WHERE ${props.columns.map(col => `${prepareDbColumn(col.column_name)} = ${prepareDbValue(row[col.column_name])}`).join(' AND ')}
      `, true).catch(() => {
        alert('Error: failed to delete the row.')
      }).finally(async () => {
        await queryClient.invalidateQueries(['table', props.table])
      })
    } else {
      alert('Error: failed to find target row.')
    }
  }

  const editRow = async () => {
    const row = selectedRowForEditing
    if (await confirmTargetRow(props.table, props.columns, row)) {
      const setStatements: string[] = []
      Object.keys(updateValues).forEach(key => {
        const column = prepareDbColumn(key)
        const value = prepareDbValue(updateValues[key])
        setStatements.push(`${column} = ${value}`)
      })
      const sql = `UPDATE ${props.table}\n` +
        `SET \n  ${setStatements.join(',\n  ')}`
      console.log('Execute?', sql, 'note: WHERE clause not included')

      if (window.confirm(`Execute? (see console if truncated)\n\n${sql}\nnote: WHERE clause not included`)) {
        await fetchQuery(
          sql + `\n WHERE ${props.columns.map(col => `${prepareDbColumn(col.column_name)} = ${prepareDbValue(row[col.column_name])}`).join(' AND ')}`,
          true
        ).catch(() => {
          alert('Error: failed to update the row.')
        }).finally(() => {
          queryClient.invalidateQueries(['table', props.table])
          setUpdateValues({})
          setRowForEditing(null)
        })
      } else {
        // Cancelled edit
      }
    } else {
      alert('Error: failed to find target row.')
    }
  }

  return <>
    {rowsQuery.data && rowsQuery.data.map(row => <tr className={"odd:bg-gray-100 hover:bg-gray-200"}>
      {props.columns.map(column => <td
        className="max-w-xs truncate border-r border-r-slate-300">{JSON.stringify(row[column.column_name])}</td>)}
      <td>
        <button
          onClick={() => {
            setUpdateValues(Object.assign({}, row))
            setRowForEditing(row)
          }}
          className={"mx-2 text-btn"}
        >
          Edit
        </button>
        <button
          onClick={() => {
            deleteRow(row)
          }}
          className={"mx-2 text-btn"}
        >
          Delete
        </button>
      </td>
    </tr>) || null}
    {selectedRowForEditing && <RowModal
        tableName={props.table}
        columns={props.columns}
        close={() => setRowForEditing(null)}
        rowOperationText={"Edit"}
        values={updateValues}
        onChange={setUpdateValues}
        onSubmit={editRow}
        submitText={"Update"}
    />}
  </>
}

const ColumnInput = (props: { column: TableColumn, value: any, onChange: (val: any) => void }) => {
  const classes = "border p-2 border-blue-500 empty:border-gray-200 focus:border-blue-500 outline-none rounded w-60"
  const type = props.column.data_type
  const placeholder = props.column.column_default ? `Default: ${props.column.column_default}` : undefined
  const isRequired = props.column.is_nullable === "NO" && !props.column.column_default
  let input

  if (type === 'boolean') {
    input = <input
      type={"checkbox"}
      checked={!!props.value}
      onChange={e => props.onChange(e.target.checked)}
    />
  } else if (type === "jsonb") {
    input = <textarea
      placeholder={placeholder}
      className={classes}
      value={JSON.stringify(props.value) || ''}
      onChange={e => props.onChange(JSON.parse(e.target.value))}
    />
  } else if (type === "integer" || type === 'bigint') {
    input = <input
      placeholder={placeholder}
      className={`${classes}`}
      type={"number"}
      value={props.value || ''}
      onChange={e => props.onChange(e.target.value)}
    />
  } else if (type === "text" || type === 'character varying') {
    input = <input
      placeholder={placeholder}
      className={`${classes}`}
      type={"text"}
      value={props.value || ''}
      onChange={e => props.onChange(e.target.value)}
    />
  } else if (type === 'timestamp with time zone' || type === 'timestamp without time zone') {
    input = <input
      placeholder={placeholder}
      className={`${classes}`}
      type={"datetime-local"}
      value={props.value || ''}
      onChange={e => props.onChange(e.target.value)}
    />
  } else {
    input = <span>
      Incompatible type <b>"{props.column.data_type}"</b>, please open an issue stating the column's type
    </span>
  }

  return <div className={"mt-2"}>
    <h6>{props.column.column_name}{isRequired && <span className={"text-rose-400"}>*</span>} ({props.column.data_type})</h6>
    {input} {props.value && <button onClick={() => props.onChange(undefined)} className={"text-blue-500 hover:text-blue-700 hover:underline"}>clear</button>}
  </div>
}

const TableView = (props: { name: string }) => {
  const [page, setPage] = useState<number>(1)
  const queryClient = useQueryClient()
  const [pageSize, setPageSize] = useState<number>(10)
  const pageSizes = [5, 10, 20, 40, 80, 160]
  const tableCountQuery = useTableCount(props.name)
  const viewQuery = useQuery<TableColumn[]>(['table', props.name, 'columns'], () => fetchQuery(`
      SELECT *
      FROM information_schema.columns
      WHERE table_name = '${props.name}'
  `))

  const columns: TableColumn[] = (viewQuery.data || []).sort((a, b) => a.ordinal_position - b.ordinal_position)
  const numberOfRecords = typeof tableCountQuery.data === 'number' ? tableCountQuery.data : 0
  const numberOfPages = Math.ceil(numberOfRecords / pageSize)
  const recordFrom = (page - 1) * pageSize
  const recordTo = Math.min(numberOfRecords, recordFrom + pageSize)

  const [addModalOpen, setModalOpen] = useState<boolean>(false)
  const [addValues, setAddValues] = useState<any>({})
  const toggleAddModal = () => {
    setModalOpen(!addModalOpen)
    setAddValues({})
  }

  const create = async () => {
    const columnsToInsert: string[] = []
    const valuesToInsert: string[] = []
    Object.keys(addValues).forEach(key => {
      const column = prepareDbColumn(key)
      const value = prepareDbValue(addValues[key])
      columnsToInsert.push(column)
      valuesToInsert.push(value)
    })
    const sql = `INSERT INTO ${props.name} (\n  ${columnsToInsert.join(',\n  ')}\n) ` +
                `VALUES (\n  ${valuesToInsert.join(',\n  ')}\n)`
    console.log('Execute?', sql)
    if (window.confirm(`Execute? (see console if truncated)\n\n${sql}`)) {
      fetchQuery(sql, true).catch(() => {
        alert('Error: failed to add the row.')
      }).then(() => {
        queryClient.invalidateQueries(['table', props.name])
        setAddValues({})
        setModalOpen(false)
      })
    }
  }

  return <div>
    <h1 className="font-bold text-xl">
      {props.name} {viewQuery.isFetching && <span className="text-grey-500 text-xs">(Loading...)</span>}
    </h1>
    <table className="table-fixed w-full border-grey-500 border">
      <thead className={"bg-slate-100"}>
      <tr className="text-left align-top border-b">
        {viewQuery.data && viewQuery.data.map(col =>
          <th className="p-2 border-r border-r-slate-200">{col.column_name}<br/><span
            className="text-xs">({col.data_type})</span></th>
        )}
        <th className={"p-2 w-28"}>{/* Actions */}</th>
      </tr>
      </thead>
      <tbody>
      {viewQuery.data &&
          <TableRows offset={recordFrom} limit={pageSize} columns={columns} table={props.name}/>}
      </tbody>
    </table>
    <div className="flex">
      <div className={"border border-slate-200 p-2 border-r-0 border-t-0"}>
        <button
          onClick={toggleAddModal} className={"text-blue-500 hover:text-blue-700 hover:underline"}
        >
          Add Record
        </button>
      </div>
      <div className={"border border-slate-200 p-2 border-r-0 border-t-0"}>
        <label className={"pr-4"}>Page</label>
        <button
          onClick={() => {
            setPage(page - 1)
          }}
          disabled={page <= 1}
          className={"mx-2 text-btn"}
        >
          Back
        </button>
        {page} of {Math.max(numberOfPages, 1)}
        <button
          onClick={() => {
            setPage(page + 1)
          }}
          disabled={page >= numberOfPages}
          className={"mx-2 text-btn"}
        >
          Next
        </button>
      </div>
      <div className={"border border-slate-200 p-2 border-t-0"}>
        <label className={"pr-4"}>Page Size</label>
        <select
          className={"text-blue-500 hover:underline hover:text-blue-700 cursor-pointer bg-transparent"}
          value={pageSize}
          onChange={e => setPageSize(+e.target.value)}
        >
          {pageSizes.map(size => <option value={size}>{size}</option>)}
        </select>
      </div>
      <div className="flex-1"/>
      <div>{Math.min(recordFrom + 1, recordTo)} - {recordTo} of {typeof tableCountQuery.data === 'number' ? tableCountQuery.data : (tableCountQuery.isFetching ? '…' : '?')} results</div>
    </div>
    {addModalOpen && <RowModal
        tableName={props.name}
        values={addValues}
        onChange={setAddValues}
        columns={columns}
        close={() => setModalOpen(false)}
        rowOperationText={"Add"}
        onSubmit={create}
        submitText={"Create"}
    />}
  </div>
}

const RowModal = (props: {
  close: () => void
  tableName: string
  rowOperationText: string
  columns: TableColumn[]
  values: any
  onChange: (values: any) => void
  onSubmit: () => void
  submitText: string
}) => {
  const {
    close,
    tableName,
    rowOperationText,
    columns,
    values,
    onChange,
    submitText,
    onSubmit
  } = props
  return <div
    className={"absolute h-full w-full top-0 left-0 right-0 bottom-0 bg-slate-100 bg-opacity-70"}
    onClick={(e) => e.target === e.currentTarget && close()}
  >
    <div className={"container mx-auto m-0 sm:m-4 md:m-8 lg:m-12 xl:m-16 p-4 bg-white rounded shadow-md"}>
      <h1 className={"text-xl mb-4"}>{rowOperationText} '<span className={"font-bold"}>{tableName}</span>' entry</h1>
      <div className={undefined/*"grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"*/}>
        {columns.map(col =>
          <ColumnInput
            key={col.column_name}
            value={values[col.column_name]}
            onChange={(v) => onChange({
              ...values,
              [col.column_name]: v
            })}
            column={col}
          />)}
      </div>
      <button
        onClick={onSubmit}
        className={"mt-4 mr-2 p-2 text-btn"}
      >
        {submitText}
      </button>
      <button
        onClick={close}
        className={"mt-4 p-2 text-btn"}
      >
        Cancel
      </button>
    </div>
  </div>
}

export const DatabasePage = () => {
  /*
    SELECT tablename AS name, (SELECT COUNT(*) FROM `tablename`) AS count FROM (SELECT * FROM pg_catalog.pg_tables WHERE schemaname != 'pg_catalog' AND schemaname != 'information_schema')
  */

  const tableQuery = useQuery<{ name: string }[]>(
    'tables',
    () => fetchQuery(
      `SELECT tablename AS name
       FROM pg_catalog.pg_tables
       WHERE schemaname != 'pg_catalog' AND schemaname != 'information_schema'`
    )
  )

  const [selectedTable, setSelectedTable] = useState<string | undefined>(undefined)

  return (
    <div className={"flex grid grid-cols-5 h-full"}>
      <div className="p-4 w-50 border-r border-grey-50 h-full">
        <h2 className="text-xs">Tables {tableQuery.isFetching &&
            <span className="text-gray-500">(Loading...)</span>}</h2>
        <ul>
          {tableQuery.data && tableQuery.data.map(table =>
            <li className="flex">
              <TableLink name={table.name} onClick={() => setSelectedTable(table.name)}/>
            </li>
          )}

        </ul>
      </div>
      <div className="p-4 col-span-4 overflow-x-auto">
        {!selectedTable && <div className="text-gray-500">
            No table selected.
        </div>}
        {selectedTable && <TableView name={selectedTable}/>}
      </div>
    </div>
  )
}
