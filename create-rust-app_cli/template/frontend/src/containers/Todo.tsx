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
  const [todos, setTodos] = useState<PaginationResult<Todo>>()
  const [createdTodo, setCreatedTodo] = useState<Todo>()
  const pageSize = 5
  const [page, setPage] = useState<number>(0)
  const [numPages, setPages] = useState<number>(1)
  const [processing, setProcessing] = useState<boolean>(false)

  const createTodo = async (todo: string) => {
    setProcessing(true)
    let createdTodo = await TodoAPI.create(todo)
    let todos = await TodoAPI.get(page, pageSize)
    setTodos(todos)
    setCreatedTodo(createdTodo)
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

  // fetch on page change
  useEffect(() => {
    setProcessing(true)
    TodoAPI.get(page, pageSize).then((todos) => {
      setTodos(todos)
      setProcessing(false)
    })
  }, [page])

  // update total number of pages
  useEffect(() => {
    if (todos) setPages(todos?.num_pages)
  }, [todos])

  useEffect(() => {
    editTodo(null)
    if (page < 0) setPage(0)
    if (numPages != 0 && page >= numPages) setPage(numPages - 1)
  }, [page, numPages])

  useEffect(() => {
    // go to the latest page when a new todo is created
    setPage(numPages - 1)
    setCreatedTodo(undefined)
  }, [createdTodo])

  return (
      <div style={{ display: 'flex', flexFlow: 'column', textAlign: 'left' }}>
        <h1>Todos</h1>
        {(!todos || todos.total_items === 0) && "No todos, create one!"}
        {todos?.items.map((todo, index) =>
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
            <button disabled={processing || page === 0} onClick={() => setPage(page - 1)}>{`<<`}</button>
            <span style={{ flex: 1, textAlign: 'center' }}>
            Page {page + 1} of {numPages}
          </span>
            <button
                disabled={processing || page === numPages - 1}
                onClick={() => setPage(page + 1)}
            >{`>>`}</button>
          </div>
        </div>
      </div>
  )
}
