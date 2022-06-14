interface FontResource {
  name: string
  href?: string
  fallback: readonly string[]
  type: 'sans-serif' | 'serif' | 'monospace'
}

export const fonts: readonly FontResource[] = [
  { name: 'Arial', fallback: ['Helvetica'], type: 'sans-serif' },
  { name: 'Helvetica', fallback: ['Arial'], type: 'sans-serif' },
  { name: 'Times New Roman', fallback: ['Times'], type: 'serif' },
  { name: 'Times', fallback: ['Times New Roman'], type: 'serif' },
  { name: 'Courier New', fallback: ['Courier'], type: 'monospace' },
  {
    name: 'Open Sans',
    fallback: [],
    href: 'https://fonts.googleapis.com/css?family=Open+Sans:400,700',
    type: 'sans-serif',
  },
  { name: 'Roboto', fallback: [], href: 'https://fonts.googleapis.com/css?family=Roboto:400,700', type: 'sans-serif' },
  { name: 'Ubuntu', fallback: [], href: 'https://fonts.googleapis.com/css?family=Ubuntu:400,700', type: 'sans-serif' },
  {
    name: 'Montserrat',
    fallback: [],
    href: 'https://fonts.googleapis.com/css?family=Montserrat:400,700',
    type: 'sans-serif',
  },
  { name: 'Lato', fallback: [], href: 'https://fonts.googleapis.com/css?family=Lato:400,700', type: 'sans-serif' },
  { name: 'Lora', fallback: [], href: 'https://fonts.googleapis.com/css?family=Lora:400,700', type: 'serif' },
  {
    name: 'Source Serif Pro',
    fallback: [],
    href: 'https://fonts.googleapis.com/css?family=Source+Serif+Pro:400,700',
    type: 'serif',
  },
]

export const DEFAULT_FONT_SIZE = 13
export const DEFAULT_FONT = fonts.find((f) => f.name === 'Ubuntu')
export const fontsMap = fonts.reduce((p, c) => (p[c.name] = c) && p, {} as Record<string, FontResource>)
