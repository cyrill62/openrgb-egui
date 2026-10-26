import tailwindcss from '@tailwindcss/vite'
import vike from 'vike/plugin'
import { defineConfig } from 'vite'

export default defineConfig({
  base: process.env.BASE_PATH || '/',
  publicDir: '../docs/screenshots',
  plugins: [tailwindcss(), vike()],
})
