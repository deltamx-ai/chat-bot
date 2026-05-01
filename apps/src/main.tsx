import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

import './index.css'
import { SwrProvider } from './providers/SwrProvider'
import { ThemeProvider } from './providers/ThemeProvider'
import WorkbenchPage from './pages/WorkbenchPage'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ThemeProvider>
      <SwrProvider>
        <WorkbenchPage />
      </SwrProvider>
    </ThemeProvider>
  </StrictMode>,
)
