import { useMemo, useState } from 'react'
import useSWR from 'swr'

import { httpClient } from '../lib/http'

export interface UseGetSwrOptions<TData, TView = TData> {
  fallbackData?: TView
  normalize?: (payload: unknown) => TView
}

export function useGetSWR<TData = unknown, TView = TData>(
  key: string | null,
  options: UseGetSwrOptions<TData, TView> = {},
) {
  const query = useSWR<TData>(key)

  const data = useMemo(() => {
    if (query.data !== undefined) {
      if (options.normalize) return options.normalize(query.data)
      return query.data as unknown as TView
    }
    return options.fallbackData as TView | undefined
  }, [query.data, options])

  return {
    ...query,
    data,
  }
}

export function usePostSWR<TResponse = unknown, TBody = unknown>(path: string) {
  const [isSubmitting, setIsSubmitting] = useState(false)

  const post = async (body?: TBody) => {
    setIsSubmitting(true)
    try {
      return await httpClient.post<TResponse>(path, body)
    } finally {
      setIsSubmitting(false)
    }
  }

  return {
    post,
    isSubmitting,
  }
}
