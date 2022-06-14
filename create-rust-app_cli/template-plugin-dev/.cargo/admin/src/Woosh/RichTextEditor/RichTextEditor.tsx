//
// License warning: this file contains modified code from the slatejs editor
//                  example and thus some of it may be exclusively licensed
//                  under the MIT license without the option of the APACHE2
//                  license as in the create-rust-app repository.
//
// See https://github.com/ianstormtaylor/slate/blob/main/License.md
//
import React, { CSSProperties, useCallback, useMemo } from 'react'
import { jsx } from 'slate-hyperscript'
import { createEditor, Descendant, Element as SlateElement, Transforms } from 'slate'
import { withHistory } from 'slate-history'
import { Editable, Slate, withReact } from 'slate-react'
import { HoveringToolbar, toggleBlock, toggleFormat } from './HoveringToolbar'
import isHotkey from 'is-hotkey'
import { escapeHTML, inlineStyles } from '../util'
import omitBy from 'lodash/omitBy'
import isNil from 'lodash/isNil'
import { CustomElement } from '../types'
import parse from 'html-react-parser'

const toJss = (inlineCss: string): CSSProperties => {
  if (!inlineCss) return {}

  const cssInlineEntries = inlineCss.split(';').filter((t) => !!t)
  if (cssInlineEntries.length === 0) return {}

  return cssInlineEntries.reduce((p, c) => {
    const keyValue = c.split(':')
    let key = keyValue[0]?.trim()
    let value = keyValue[1]?.trim()
    p[key] = value
    return p
  }, {} as CSSProperties)
}

const extractAttrs = (el: HTMLElement) => {
  return Object.assign(
    {},
    ...el.getAttributeNames().map((n) => ({ [n]: n === 'style' ? toJss(el.getAttribute(n)) : el.getAttribute(n) }))
  )
}

const ELEMENT_TAGS = {
  SPAN: (el) => ({ type: 'span', ...extractAttrs(el) }),
  DIV: (el) => ({ type: 'div', ...extractAttrs(el) }),
  A: (el) => ({ type: 'a', ...extractAttrs(el) }),
  BLOCKQUOTE: (el) => ({ type: 'quote', ...extractAttrs(el) }),
  H1: (el) => ({ type: 'h1', ...extractAttrs(el) }),
  H2: (el) => ({ type: 'h2', ...extractAttrs(el) }),
  H3: (el) => ({ type: 'h3', ...extractAttrs(el) }),
  H4: (el) => ({ type: 'h4', ...extractAttrs(el) }),
  H5: (el) => ({ type: 'h5', ...extractAttrs(el) }),
  H6: (el) => ({ type: 'h6', ...extractAttrs(el) }),
  LI: (el) => ({ type: 'li', ...extractAttrs(el) }),
  OL: (el) => ({ type: 'ol', ...extractAttrs(el) }),
  P: (el) => ({ type: 'p', ...extractAttrs(el) }),
  PRE: (el) => ({ type: 'code', ...extractAttrs(el) }),
  UL: (el) => ({ type: 'ul', ...extractAttrs(el) }),
}

// COMPAT: `B` is omitted here because Google Docs uses `<b>` in weird ways.
const TEXT_TAGS = {
  CODE: () => ({ code: true }),
  DEL: () => ({ strikethrough: true }),
  EM: () => ({ italic: true }),
  I: () => ({ italic: true }),
  S: () => ({ strikethrough: true }),
  STRONG: () => ({ bold: true }),
  U: () => ({ underline: true }),
}

const TEXT_FORMATTING_HOTKEYS = {
  'mod+b': 'bold',
  'mod+i': 'italic',
  'mod+u': 'underline',
  'mod+`': 'code',
}

const BLOCK_FORMATTING_HOTKEYS = {
  'mod+1': 'h1',
  'mod+2': 'h2',
  'mod+3': 'code',
  'mod+4': 'ol',
  'mod+5': 'ul',
  'mod+l': 'left',
  'mod+r': 'right',
  'mod+e': 'center',
  'mod+j': 'justify',
}
const serializeAttrs = (el: CustomElement) => {
  const attrs = Object.entries(omitBy(el, (v, k) => isNil(v) || ['type', 'children'].includes(k)))
  return `${attrs.map(([key, value]: [string, any]) => `${key}="${escapeHTML(value)}"`).join(' ')}`
}

export const serialize = (v: Descendant[]) => {
  // for empty elements which act as a line break, we make sure their text descendant is a space so it is
  // rendered as a linebreak in MJML
  if (v.length === 1 && Object.keys(v[0]).length === 1 && (v[0] as any).text === '') {
    return '&nbsp;'
  }

  const html = v
    .map((d) => {
      if (SlateElement.isElement(d)) {
        let tag = Object.keys(ELEMENT_TAGS).find((t) => t.toLowerCase() === d.type.toLowerCase())

        if (tag) {
          tag = tag.toLowerCase()
          return `<${tag} ${serializeAttrs(d)}>${serialize(d.children)}</${tag}>`
        }
      } else {
        let text = escapeHTML(d.text, true)
        Object.entries(TEXT_TAGS).forEach((entry) => {
          let tag = entry[0].toLowerCase()
          let property = Object.keys(entry[1]())[0]
          if (d[property]) text = `<${tag}>${text}</${tag}>`
        })
        return d['style'] ? `<span style="${inlineStyles(d['style'])}">${text}</span>` : text
      }

      return '' // not found
    })
    .join('')

  console.log('SERIALIZED', v, html)

  return html
}

export const deserialize = (el) => {
  if (el.nodeType === 3) {
    return el.textContent
  } else if (el.nodeType !== 1) {
    return null
  } else if (el.nodeName === 'BR') {
    return '\n'
  }

  const { nodeName } = el
  let parent = el

  if (nodeName === 'PRE' && el.childNodes[0] && el.childNodes[0].nodeName === 'CODE') {
    parent = el.childNodes[0]
  }
  let children = Array.from(parent.childNodes).map(deserialize).flat()

  if (children.length === 0) {
    children = [{ text: '' }]
  }

  if (el.nodeName === 'BODY') {
    return jsx('fragment', {}, children)
  }

  if (ELEMENT_TAGS[nodeName]) {
    const attrs = ELEMENT_TAGS[nodeName](el)
    return jsx('element', attrs, children)
  }

  if (TEXT_TAGS[nodeName]) {
    const attrs = TEXT_TAGS[nodeName](el)
    return children.map((child) => jsx('text', attrs, child))
  }

  return children
}

type HTML = string

interface Props {
  className: string
  style: CSSProperties
  initialValue: HTML
  onChange: (newValue: HTML) => void
}

export const RichTextEditor = (props: Props) => {
  const renderElement = useCallback((props) => <Element {...props} />, [])
  const renderLeaf = useCallback((props) => <Leaf {...props} />, [])
  const editor = useMemo(() => withHtml(withReact(withHistory(createEditor()))), [])
  const initialValue = useMemo(() => {
    const parsed = new DOMParser().parseFromString(props.initialValue, 'text/html')
    const nodes = deserialize(parsed.body)
    console.log('DESERIALIZED', props.initialValue, nodes)
    return nodes
  }, [])
  const onChange = (v: Descendant[]) => {
    console.log('onChange')
    props.onChange(serialize(v))
  }
  return (
    <Slate editor={editor} value={initialValue} onChange={onChange}>
      <HoveringToolbar />
      <Editable
        className={props.className}
        style={props.style}
        renderElement={renderElement}
        renderLeaf={renderLeaf}
        placeholder="Lorem ipsum..."
        onKeyDown={(event) => {
          for (const hotkey in TEXT_FORMATTING_HOTKEYS) {
            if (isHotkey(hotkey, event)) {
              event.preventDefault()
              const mark = TEXT_FORMATTING_HOTKEYS[hotkey]
              toggleFormat(editor, mark)
            }
          }
          for (const hotkey in BLOCK_FORMATTING_HOTKEYS) {
            if (isHotkey(hotkey, event)) {
              event.preventDefault()
              const block = BLOCK_FORMATTING_HOTKEYS[hotkey]
              toggleBlock(editor, block)
            }
          }
        }}
        onDOMBeforeInput={(event: InputEvent) => {
          if (event.inputType === 'formatBold') {
            event.preventDefault()
            return toggleFormat(editor, 'bold')
          } else if (event.inputType === 'formatItalic') {
            event.preventDefault()
            return toggleFormat(editor, 'italic')
          } else if (event.inputType === 'formatUnderline') {
            event.preventDefault()
            return toggleFormat(editor, 'underline')
          }
        }}
      />
    </Slate>
  )
}

const withHtml = (editor) => {
  const { insertData, isInline, isVoid } = editor

  editor.isInline = (element) => {
    return element.type === 'a' ? true : isInline(element)
  }

  editor.insertData = (data) => {
    const html = data.getData('text/html')

    if (html) {
      const parsed = new DOMParser().parseFromString(html, 'text/html')
      const fragment = deserialize(parsed.body)
      Transforms.insertFragment(editor, fragment)
      return
    }

    insertData(data)
  }

  return editor
}

const Element = (props) => {
  const { attributes, children, element } = props

  const elementAttrs = omitBy(element, (v, k) => ['type', 'children'].includes(k))

  const e = parse(
    `<${element.type} ${Object.entries(elementAttrs)
      .map(([k, v]) => `${k}="${escapeHTML(v)}"`)
      .join(' ')} />`
  )

  return React.cloneElement(e as React.ReactElement, {
    ...attributes,
    children,
  })

  // switch (element.type) {
  //   default:
  //     return <span {...elementAttrs} {...attributes}>{children}</span>
  //   case 'blockquote':
  //     return <blockquote {...elementAttrs} {...attributes}>{children}</blockquote>
  //   case 'code':
  //     return <pre><code {...elementAttrs} {...attributes}>{children}</code></pre>
  //   case 'ul':
  //     return <ul {...elementAttrs} {...attributes}>{children}</ul>
  //   case 'h1':
  //     return <h1 {...elementAttrs} {...attributes}>{children}</h1>
  //   case 'h2':
  //     return <h2 {...elementAttrs} {...attributes}>{children}</h2>
  //   case 'h3':
  //     return <h3 {...elementAttrs} {...attributes}>{children}</h3>
  //   case 'h4':
  //     return <h4 {...elementAttrs} {...attributes}>{children}</h4>
  //   case 'h5':
  //     return <h5 {...elementAttrs} {...attributes}>{children}</h5>
  //   case 'h6':
  //     return <h6 {...elementAttrs} {...attributes}>{children}</h6>
  //   case 'li':
  //     return <li {...elementAttrs} {...attributes}>{children}</li>
  //   case 'ol':
  //     return <ol {...elementAttrs} {...attributes}>{children}</ol>
  //   case 'a':
  //     return <a {...elementAttrs} {...attributes}>{children}<br/><pre className={"text-left"}><code>{JSON.stringify(element, null, 2)}</code></pre><br/></a>
  // }
}

const Leaf = ({ attributes, children, leaf }) => {
  const style: CSSProperties = omitBy(leaf.style || {}, isNil)

  if (leaf.bold) {
    children = <strong style={style}>{children}</strong>
  }

  if (leaf.code) {
    children = <code style={style}>{children}</code>
  }

  if (leaf.italic) {
    children = <em style={style}>{children}</em>
  }

  if (leaf.underline) {
    children = <u style={style}>{children}</u>
  }

  if (leaf.strikethrough) {
    children = <del style={style}>{children}</del>
  }

  return (
    <span {...attributes} style={style}>
      {children}
    </span>
  )
}
