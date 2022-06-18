import react from '@vitejs/plugin-react'
import {resolve} from 'path'
import {defineConfig} from 'vite'
import glob from 'glob'

const buildRollupInput = (isDevelopment) => {
    const rollupInput = isDevelopment ? {
        'dev.tsx': resolve(__dirname, './src/dev.tsx')
    } : {}

    // TODO: use import.meta.glob() + npm uninstall glob

    glob.sync(resolve(__dirname, './bundles/**/*.tsx')).map(inputEntry => {
        let outputEntry = inputEntry
        // output entry is an absolute path, let's remove the absolute part:
        outputEntry = outputEntry.replace(`${__dirname}/`, '')
        // replace directory separator with "__"
        outputEntry = outputEntry.replace(/\//g, '__')

        rollupInput[outputEntry] = inputEntry
    })

    return rollupInput
}

// https://vitejs.dev/config/
export default defineConfig(async ({ command, mode }) => ({
    base: command === 'serve' ? 'http://localhost:21012' : '/',
    build: {
        manifest: true,
        rollupOptions: {
            input: buildRollupInput(command === 'serve')
        },
    },
    plugins: [react()],
    server: {
        port: 21012,
        proxy: {
            // with options
            '/api': {
                target: 'http://localhost:3000',
                changeOrigin: true,
            },
            '/graphql': {
                target: 'http://localhost:3000',
                changeOrigin: true
            }
        }
    },
}))
