export const API = {
  health: {
    check: async () => {
      return (await fetch('/api/health')).json()
    }
  },
  todos: {
    get: async (page: number, size: number) => await (await fetch(`/api/todos?page=${page}&page_size=${size}`)).json(),
    create: async (todo: string) => await (await fetch('/api/todos', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ text: todo })
    })).json(),
    delete: async (id: number) => await (await fetch(`/api/todos/${id}`, { method: 'DELETE' })),
    update: async (id: number, todo: string) => await (await fetch(`/api/todos/${id}`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ text: todo })
    }))
  }
}
