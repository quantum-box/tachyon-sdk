import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import dts from 'vite-plugin-dts'

// https://vite.dev/config/
export default defineConfig({
	plugins: [react(), dts()],
	build: {
		outDir: 'dist',
		lib: {
			entry: 'src/index.ts',
			name: '@tachyon-sdk/agent-chat',
			fileName: 'index',
			formats: ['es', 'umd'],
		},
		rollupOptions: {
			external: ['react', 'react-dom'],
			output: {
				globals: {
					react: 'React',
					'react-dom': 'ReactDOM',
				},
			},
		},
	},
})
