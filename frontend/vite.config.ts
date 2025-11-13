import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'
import { readFileSync } from 'fs'

const { version: appVersion } = JSON.parse(
  readFileSync(new URL('./package.json', import.meta.url), 'utf-8')
)

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(process.cwd(), 'src')
    }
  },
  define: {
    __APP_VERSION__: JSON.stringify(appVersion)
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
