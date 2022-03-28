import React, { useEffect, useState } from 'react'

const FilesAPI = {
    all: async () =>
        await (await fetch(`/api/files`)).json(),
    create: async (formData: FormData) =>
        await fetch('/api/files', {
            method: 'POST',
            body: formData,
        }),
    delete: async (id: number) =>
        await fetch(`/api/files/${id}`, { method: 'DELETE' })
}

export const Files = () => {
    const [files, setFiles] = useState<FileInfo[]>([])
    const [processing, setProcessing] = useState<boolean>(false)

    const createFile = async (form: FormData) => {
        setProcessing(true)
        await FilesAPI.create(form)
        setFiles(await FilesAPI.all())
        const el = document.getElementById("file")! as HTMLInputElement
        el.value = ''
        setProcessing(false)
    }

    const deleteFile = async (file: FileInfo) => {
        setProcessing(true)
        await FilesAPI.delete(file.id)
        setFiles(await FilesAPI.all())
        setProcessing(false)
    }

    useEffect(() => {
        setProcessing(true)
        FilesAPI.all().then((files) => {
            setFiles(files)
            setProcessing(false)
        })
    }, [])

    return (
        <div style={{ display: 'flex', flexFlow: 'column', textAlign: 'left' }}>
            <h1>Files</h1>
            {files.map((file, index) =>
                (
                    <div className="Form">
                        <div style={{ flex: 1 }}>
                            #{index + 1}. {file.name} ({file.url})
                        </div>
                        <div>
                            <a href={file.url} className="App-link">
                                download
                            </a>
                            &nbsp;
                            <a href="#" className="App-link" onClick={() => deleteFile(file)}>
                                delete
                            </a>
                        </div>
                    </div>
                )
            )}
            {files.length === 0 && "No files, upload some!"}

            <div className="Form">
                <div style={{ display: 'flex' }}>
                    <input
                        style={{ flex: 1 }}
                        id="file"
                        type="file"
                        placeholder="New todo..."
                        multiple={false}
                    />
                    <button
                        disabled={processing}
                        style={{ height: '40px' }}
                        onClick={() => {
                            const form = new FormData()
                            const el = document.getElementById("file")! as HTMLInputElement
                            form.append("file", el.files![0])
                            createFile(form)
                        }}
                    >
                        Upload
                    </button>
                </div>
            </div>
        </div>
    )
}