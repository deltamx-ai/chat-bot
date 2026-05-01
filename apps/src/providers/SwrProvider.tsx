import type { PropsWithChildren } from 'react'
import { SWRConfig } from 'swr'

import { reportGlobalError, shouldIgnoreError } from '../lib/errorHandling'
import { httpClient } from '../lib/http'

export function SwrProvider({ children }: PropsWithChildren) {
  return (
    <SWRConfig
      value={{
        fetcher: (path: string) => httpClient.get(path),
        shouldRetryOnError: false,
        onError: (error, key) => {
          if (!shouldIgnoreError(error, typeof key === 'string' ? key : undefined)) {
            reportGlobalError(error)
          }
        },
      }}
    >
      {children}
    </SWRConfig>
  )
}
