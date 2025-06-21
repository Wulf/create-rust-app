import react from '@vitejs/plugin-react'
import glob from 'glob'
import { resolve } from 'path'
import { defineConfig } from 'vite'

const buildRollupInput = (isDevelopment): { [entryAlias: string]: string } => {
    const rollupInput: { [entryAlias: string]: string } = isDevelopment ? {
        'dev.tsx': resolve(__dirname, './src/dev.tsx')
    } : {}

    // TODO: use import.meta.glob() + npm uninstall glob

    glob.sync(resolve(__dirname, './bundles/**/*.tsx')).map((inputEntry: string) => {
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
    clearScreen: false,
    build: {
        manifest: true,
        rollupOptions: {
            input: buildRollupInput(command === 'serve')
        },
    },
    define: {
        // When this variable is set, setupDevelopment.tsx will also be loaded!
        // See `dev.tsx` which is included in development.
        'import.meta.env.DEV_SERVER_PORT': String(process.env.DEV_SERVER_PORT),
    },
    plugins: [
        react(),
    ],

    server: {
        port: 21012,
        host: '0.0.0.0',
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
