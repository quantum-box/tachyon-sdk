'use client'

import type { ImageProps } from 'next/image'
import Image from 'next/image'
import tachyonImageLoader from '../image-loader'

type TachyonImageProps = Omit<ImageProps, 'src' | 'loader'> & {
  storageKey: string
}

export function TachyonImage({ storageKey, ...props }: TachyonImageProps) {
  return <Image {...props} src={storageKey} loader={tachyonImageLoader} />
}
