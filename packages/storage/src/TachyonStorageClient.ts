export interface UploadResult {
  storageKey: string
  url: string
  contentType: string
  contentLength: number
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

function getExtension(file: File | Blob): string {
  if (file instanceof File) {
    const dotIndex = file.name.lastIndexOf('.')
    if (dotIndex !== -1) return file.name.slice(dotIndex + 1).toLowerCase()
  }
  return 'bin'
}

export class TachyonStorageClient {
  private apiKey: string
  private baseUrl: string
  private appId?: string

  constructor(options: { apiKey: string; baseUrl?: string; appId?: string }) {
    this.apiKey = options.apiKey
    this.baseUrl = options.baseUrl || 'https://api.n1.tachy.one'
    this.appId = options.appId
  }

  async upload(file: File | Blob, options?: { fileName?: string }): Promise<UploadResult> {
    const ext =
      options?.fileName
        ? options.fileName.slice(options.fileName.lastIndexOf('.') + 1).toLowerCase()
        : getExtension(file)
    const contentType = file.type || 'application/octet-stream'

    const res = await fetch(`${this.baseUrl}/v1/storage/upload-url`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${this.apiKey}`,
      },
      body: JSON.stringify({
        content_type: contentType,
        extension: ext,
        app_id: this.appId,
      }),
    })

    if (!res.ok) {
      throw new Error(`Failed to get upload URL: ${res.status} ${res.statusText}`)
    }

    const presigned: PresignedUrlResponse = await res.json()

    await new Promise<void>((resolve, reject) => {
      const xhr = new XMLHttpRequest()
      xhr.open('PUT', presigned.url)
      xhr.setRequestHeader('Content-Type', contentType)

      xhr.onload = () => {
        if (xhr.status >= 200 && xhr.status < 300) {
          resolve()
        } else {
          reject(new Error(`Upload failed: ${xhr.status} ${xhr.statusText}`))
        }
      }

      xhr.onerror = () => reject(new Error('Upload failed: network error'))
      xhr.send(file)
    })

    const confirmRes = await fetch(`${this.baseUrl}/v1/storage/confirm`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${this.apiKey}`,
      },
      body: JSON.stringify({ storage_key: presigned.storage_key }),
    })

    if (!confirmRes.ok) {
      throw new Error(`Failed to confirm upload: ${confirmRes.status} ${confirmRes.statusText}`)
    }

    const confirmed: ConfirmResponse = await confirmRes.json()

    return {
      storageKey: confirmed.storage_key,
      url: confirmed.url,
      contentType: confirmed.content_type,
      contentLength: confirmed.content_length,
    }
  }

  storageUrl(
    storageKey: string,
    opts?: { width?: number; quality?: number; cdnBase?: string },
  ): string {
    const cdnBase = opts?.cdnBase || 'https://cdn.txcloud.app'
    if (opts?.width) {
      const params = `w=${opts.width},q=${opts.quality || 80},f=auto`
      return `${cdnBase}/cdn-cgi/image/${params}/${storageKey}`
    }
    return `${cdnBase}/${storageKey}`
  }
}
