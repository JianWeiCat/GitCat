import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { ThemeProvider } from '@lobehub/ui'
import './index.css'
import App from './App.tsx'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ThemeProvider customTheme={{ neutralColor: 'slate', primaryColor: 'blue' }} themeMode="light">
      <App />
    </ThemeProvider>
  </StrictMode>,
)
