export { TachyonStorageClient } from './TachyonStorageClient'
export type { UploadResult } from './TachyonStorageClient'

export function storageUrl(
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
