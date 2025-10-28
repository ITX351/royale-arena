import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(process.cwd(), 'src')
    }
  },
  base: '/royale-arena/',
  server: {
    proxy: {
      '/royale-arena/api': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
        ws: true,
        rewrite: (path) => path.replace(/^\/royale-arena\/api/, '')
      }
    }
  }
})
