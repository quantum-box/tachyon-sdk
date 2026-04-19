'use client'

import { useCallback, useRef, useState } from 'react'

interface UploadOptions {
  apiBase?: string
  apiKey?: string
  appId?: string
  headers?: Record<string, string>
}

interface UploadProgress {
  loaded: number
  total: number
  percentage: number
}

export interface UploadResult {
  storageKey: string
  url: string
  contentType: string
  contentLength: number
}

interface UploadState {
  uploading: boolean
  progress: UploadProgress | null
  error: Error | null
}

interface PresignedUrlResponse {
  url: string
  storage_key: string
  expires_in_secs: number
}

interface ConfirmResponse {
  storage_key: string
  url: string
  content_type: string
  content_length: number
}

function getFileExtension(file: File): string {
  const name = file.name
  const dotIndex = name.lastIndexOf('.')
  if (dotIndex === -1) return 'bin'
  return name.slice(dotIndex + 1).toLowerCase()
}

export function useUpload(options?: UploadOptions) {
  const [state, setState] = useState<UploadState>({
    uploading: false,
    progress: null,
    error: null,
  })
  const abortControllerRef = useRef<AbortController | null>(null)

  const apiBase = options?.apiBase || 'https://api.n1.tachy.one'

  const authHeaders: Record<string, string> = options?.apiKey
    ? { Authorization: `Bearer ${options.apiKey}` }
    : {}

  const upload = useCallback(
    async (file: File): Promise<UploadResult> => {
      setState({ uploading: true, progress: null, error: null })
      abortControllerRef.current = new AbortController()

      try {
        const ext = getFileExtension(file)
        const res = await fetch(`${apiBase}/v1/storage/upload-url`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            ...authHeaders,
            ...options?.headers,
          },
          body: JSON.stringify({
            content_type: file.type || 'application/octet-stream',
            extension: ext,
            app_id: options?.appId,
          }),
          signal: abortControllerRef.current.signal,
        })

        if (!res.ok) {
          throw new Error(`Failed to get upload URL: ${res.status} ${res.statusText}`)
        }

        const presigned: PresignedUrlResponse = await res.json()

        await new Promise<void>((resolve, reject) => {
          const xhr = new XMLHttpRequest()
          xhr.open('PUT', presigned.url)
          xhr.setRequestHeader('Content-Type', file.type || 'application/octet-stream')

          xhr.upload.onprogress = e => {
            if (e.lengthComputable) {
              setState(prev => ({
                ...prev,
                progress: {
                  loaded: e.loaded,
                  total: e.total,
                  percentage: Math.round((e.loaded / e.total) * 100),
                },
              }))
            }
          }

          xhr.onload = () => {
            if (xhr.status >= 200 && xhr.status < 300) {
              resolve()
            } else {
              reject(new Error(`Upload failed: ${xhr.status} ${xhr.statusText}`))
            }
          }

          xhr.onerror = () => reject(new Error('Upload failed: network error'))

          abortControllerRef.current?.signal.addEventListener('abort', () => {
            xhr.abort()
            reject(new Error('Upload cancelled'))
          })

          xhr.send(file)
        })

        const confirmRes = await fetch(`${apiBase}/v1/storage/confirm`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            ...authHeaders,
            ...options?.headers,
          },
          body: JSON.stringify({ storage_key: presigned.storage_key }),
          signal: abortControllerRef.current.signal,
        })

        if (!confirmRes.ok) {
          throw new Error(`Failed to confirm upload: ${confirmRes.status} ${confirmRes.statusText}`)
        }

        const confirmed: ConfirmResponse = await confirmRes.json()

        const result: UploadResult = {
          storageKey: confirmed.storage_key,
          url: confirmed.url,
          contentType: confirmed.content_type,
          contentLength: confirmed.content_length,
        }

        setState({ uploading: false, progress: null, error: null })
        return result
      } catch (error) {
        const err = error instanceof Error ? error : new Error('Upload failed')
        setState({ uploading: false, progress: null, error: err })
        throw err
      }
    },
    [apiBase, options?.appId, options?.headers, authHeaders],
  )

  const cancel = useCallback(() => {
    abortControllerRef.current?.abort()
  }, [])

  return { upload, cancel, ...state }
}
