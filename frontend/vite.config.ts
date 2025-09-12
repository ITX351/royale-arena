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
  server: {
    proxy: {
      '/royale-arena/api': {
        target: 'http://localhost:3000',
        changeOrigin: true
      }
    }
  }
})
