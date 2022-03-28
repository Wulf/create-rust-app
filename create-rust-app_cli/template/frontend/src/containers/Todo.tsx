import React, { useEffect, useState } from 'react'

const TodoAPI = {
  get: async (page: number, size: number) =>
    await (await fetch(`/api/todos?page=${page}&page_size=${size}`)).json(),
  create: async (todo: string) =>
    await (
      await fetch('/api/todos', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ text: todo }),
      })
    ).json(),
  delete: async (id: number) =>
    await fetch(`/api/todos/${id}`, { method: 'DELETE' }),
  update: async (id: number, todo: string) =>
    await fetch(`/api/todos/${id}`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ text: todo }),
    }),
}

export const Todos = () => {
  const [text, setText] = useState<string>('')
  const [selectedTodo, editTodo] = useState<Todo | null>(null)
  const [todos, setTodos] = useState<Todo[]>([])
  const pageSize = 5
  const [page, setPage] = useState<number>(0)
  const [numPages, setPages] = useState<number>(1)
  const [processing, setProcessing] = useState<boolean>(false)

  const createTodo = async (todo: string) => {
    setProcessing(true)
    await TodoAPI.create(todo)
    setTodos(await TodoAPI.get(page, pageSize))
    setText('')
    setProcessing(false)
  }

  const updateTodo = async (todo: Todo) => {
    setProcessing(true)
    await TodoAPI.update(todo.id, text)
    setTodos(await TodoAPI.get(page, pageSize))
    setText('')
    editTodo(null)
    setProcessing(false)
  }

  const deleteTodo = async (todo: Todo) => {
    setProcessing(true)
    await TodoAPI.delete(todo.id)
    setTodos(await TodoAPI.get(page, pageSize))
    setProcessing(false)
  }

  useEffect(() => {
    setText(selectedTodo?.text || '')
  }, [selectedTodo])

  useEffect(() => {
    setProcessing(true)
    TodoAPI.get(page, pageSize).then((todos) => {
      setTodos(todos)
      setProcessing(false)
    })
  }, [page])

  useEffect(() => {
    const numPages = Math.ceil(todos.length / pageSize)
    setPages(numPages)
    if (page < 0 || page > numPages) setPage(0)
  }, [todos, page])

  return (
    <div style={{ display: 'flex', flexFlow: 'column', textAlign: 'left' }}>
      <h1>Todos</h1>
      {todos.map((todo, index) =>
        todo.id === selectedTodo?.id ? (
          <div className="Form">
            <div style={{ display: 'flex' }}>
              <input
                style={{ flex: 1 }}
                value={text}
                onChange={(e) => setText(e.target.value)}
              />
              <button
                disabled={processing}
                style={{ height: '40px' }}
                onClick={() => updateTodo(todo)}
              >
                Save
              </button>
              <button
                disabled={processing}
                style={{ height: '40px' }}
                onClick={() => editTodo(null)}
              >
                Cancel
              </button>
            </div>
          </div>
        ) : (
          <div className="Form">
            <div style={{ flex: 1 }}>
              #{todo.id} {todo.text}
            </div>
            <div>
              <a href="#" className="App-link" onClick={() => editTodo(todo)}>
                edit
              </a>
              &nbsp;
              <a href="#" className="App-link" onClick={() => deleteTodo(todo)}>
                delete
              </a>
            </div>
          </div>
        )
      )}
      {selectedTodo === null && (
        <div className="Form">
          <div style={{ display: 'flex' }}>
            <input
              style={{ flex: 1 }}
              placeholder="New todo..."
              value={text}
              onChange={(e) => setText(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  createTodo(text)
                }
              }}
            />
            <button
              disabled={processing}
              style={{ height: '40px' }}
              onClick={() => createTodo(text)}
            >
              Add
            </button>
          </div>
        </div>
      )}
      <div className="Form">
        <div style={{ display: 'flex' }}>
          <button onClick={() => setPage(page - 1)}>{`<<`}</button>
          <span style={{ flex: 1, textAlign: 'center' }}>
            Page {page + 1} of {numPages}
          </span>
          <button
            disabled={processing}
            onClick={() => setPage(page + 1)}
          >{`>>`}</button>
        </div>
      </div>
    </div>
  )
}
