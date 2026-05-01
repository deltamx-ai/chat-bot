import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

import { SwrProvider } from './providers/SwrProvider'
import './index.css'
import WorkbenchPage from './pages/WorkbenchPage'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <SwrProvider>
      <WorkbenchPage />
    </SwrProvider>
  </StrictMode>,
)
