import React, { useEffect, useState } from 'react'
import { API } from '../api'

export const Todos = () => {
  const [text, setText] = useState<string>('')
  const [selectedTodo, editTodo] = useState<Todo | null>(null)
  const [todos, setTodos] = useState<Todo[]>([])
  const pageSize = 5
  const [page, setPage] = useState<number>(0)
  const [numPages, setPages] = useState<number>(1)
  const [healthCheckOutput, setHealthCheckOutput] = useState<string>('')
  const healthCheck = async () => {
    const response = await API.health.check()
    setHealthCheckOutput(JSON.stringify(response, null, 2))
  }

  const createTodo = async (todo: string) => {
    await API.todos.create(todo)
    setTodos(await API.todos.get(page, pageSize))
    setText('')
  }
  
  const updateTodo = async (todo: Todo) => {
    await API.todos.update(todo.id, text)
    setTodos(await API.todos.get(page, pageSize))
    setText('')
    editTodo(null)
  }
  
  const deleteTodo = async (todo: Todo) => {
    await API.todos.delete(todo.id)
    setTodos(await API.todos.get(page, pageSize))
  }

  useEffect(() => {
    setText(selectedTodo?.text || '')
  }, [selectedTodo])

  useEffect(() => {
    API.todos.get(page, pageSize).then(todos => setTodos(todos))
  }, [page])

  useEffect(() => {
    const numPages = Math.ceil(todos.length / pageSize)
    setPages(numPages)
    if (page < 0 || page >= numPages) setPage(0)
  }, [todos, page])
  
  return (
    <div className="App-queries">
      <div className="App-query">
        <button className="execute" onClick={healthCheck}>API Health Check</button>
        <pre className="output">{healthCheckOutput}</pre>
      </div>
      <div className="todos">
        <b>Todos</b>
        <ul>
          {todos.map(todo => todo.id === selectedTodo?.id ? <li className="todo editing">
            <div style={{ flex: 1 }}>
              <input value={text} onChange={e => setText(e.target.value)} />
            </div>
            <div>
              <button onClick={() => updateTodo(todo)}>save</button>
              <button onClick={() => editTodo(null)}>cancel</button>
            </div>
          </li> : <li className="todo">
            <div style={{ flex: 1 }}>{todo.text}</div>
            <div>
              <button onClick={() => editTodo(todo)}>edit</button>
              <button onClick={() => deleteTodo(todo)}>delete</button>
            </div>
          </li>)}
          {selectedTodo === null && <li className="todo new">
            <div style={{ flex: 1 }}>
              <input placeholder="New todo..." value={text} onChange={e => setText(e.target.value)} />
            </div>
            <div>
              <button onClick={() => createTodo(text)}>create</button>
            </div>
          </li>}
        </ul>
        <div className="todos-pagination">
          <button onClick={() => setPage(page - 1)}>{`<<`}</button>
          <span>{page + 1} / {numPages}</span>
          <button onClick={() => setPage(page + 1)}>{`>>`}</button>
        </div>
      </div>
    </div>
  )
}