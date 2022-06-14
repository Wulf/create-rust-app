import { CSSProperties } from 'react'

export const escapeHTML = (unsafe: string, convertSpacesToEntities: boolean = false) => {
  let safe = ''

  for (let i = 0; i < unsafe.length; i++) {
    const char = unsafe[i]

    if (char === ' ' && convertSpacesToEntities) {
      safe += '&nbsp;'
    } else if (char === '&') {
      safe += '&amp;'
    } else if (char === '<') {
      safe += '&lt;'
    } else if (char === '>') {
      safe += '&gt;'
    } else if (char === '"') {
      safe += '&quot;'
    } else if (char === "'") {
      safe += '&#039;'
    } else {
      safe += char
    }
  }

  return safe
}

const formatCSSKey = (key: string): string => {
  let formattedKey = ''
  for (let i = 0; i < key.length; i++) {
    const char = key[i]
    if (char === char.toUpperCase()) formattedKey += '-'
    formattedKey += char.toLowerCase()
  }
  return formattedKey
}
const formatCSSValue = (value: string): string => value
export const inlineStyles = (style: CSSProperties): string => {
  return `${Object.entries(style)
    .map(([k, v]) => `${formatCSSKey(k)}: ${formatCSSValue(v)}`)
    .join(';')}`
}
