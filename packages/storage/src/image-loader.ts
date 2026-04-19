export default function tachyonImageLoader({
  src,
  width,
  quality,
}: {
  src: string
  width: number
  quality?: number
}): string {
  const cdnBase = process.env.NEXT_PUBLIC_TACHYON_STORAGE_URL || 'https://cdn.txcloud.app'
  const params = `w=${width},q=${quality || 80},f=auto`
  return `${cdnBase}/cdn-cgi/image/${params}/${src}`
}
