import { Descendant, BaseEditor } from 'slate'
import { ReactEditor } from 'slate-react'
import { HistoryEditor } from 'slate-history'
import { CSSProperties } from 'react'

// html elements
export type BlockQuoteElement = { type: 'blockquote'; align?: string; children: Descendant[] }
export type BulletedListElement = { type: 'ul'; align?: string; children: Descendant[] }
export type Heading1Element = { type: 'h1'; align?: string; children: Descendant[] }
export type Heading2Element = { type: 'h2'; align?: string; children: Descendant[] }
export type Heading3Element = { type: 'h3'; align?: string; children: Descendant[] }
export type Heading4Element = { type: 'h4'; align?: string; children: Descendant[] }
export type Heading5Element = { type: 'h5'; align?: string; children: Descendant[] }
export type Heading6Element = { type: 'h6'; align?: string; children: Descendant[] }
export type LinkElement = { type: 'a'; href: string; target: string; children: Descendant[] }
export type ButtonElement = { type: 'button'; children: Descendant[] }
export type ListItemElement = { type: 'li'; children: Descendant[] }
export type ParagraphElement = { type: 'p'; align?: string; children: Descendant[] }

// non-html elements
export type MentionElement = { type: 'mention'; character: string; children: CustomText[] }

type CustomElement =
  | BlockQuoteElement
  | BulletedListElement
  | Heading1Element
  | Heading2Element
  | Heading3Element
  | Heading4Element
  | Heading5Element
  | Heading6Element
  | LinkElement
  | ButtonElement
  | ListItemElement
  | ParagraphElement
  | MentionElement

export type CustomText = {
  bold?: boolean
  italic?: boolean
  code?: boolean
  text: string
  style?: CSSProperties
}

export type CustomEditor = BaseEditor & ReactEditor & HistoryEditor

declare module 'slate' {
  interface CustomTypes {
    Editor: CustomEditor
    Element: CustomElement
    Text: CustomText | { text: string }
  }
}
