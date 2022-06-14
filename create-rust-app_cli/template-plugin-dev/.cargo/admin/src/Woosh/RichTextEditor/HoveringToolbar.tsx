//
// License warning: this file contains modified code from the slatejs editor
//                  example and thus some of it may be exclusively licensed
//                  under the MIT license without the option of the APACHE2
//                  license as in the create-rust-app repository.
//
// See https://github.com/ianstormtaylor/slate/blob/main/License.md
//
import React, { CSSProperties, PropsWithChildren, useEffect, useMemo, useRef } from 'react'
import { useFocused, useSlate } from 'slate-react'
import { Editor, Element, Element as SlateElement, Location, Range, Text, Transforms } from 'slate'
import { Button, Portal } from './components'
import { useFrame } from 'react-frame-component'
import { AiOutlineBold } from '@react-icons/all-files/ai/AiOutlineBold'
import { AiOutlineItalic } from '@react-icons/all-files/ai/AiOutlineItalic'
import { AiOutlineUnderline } from '@react-icons/all-files/ai/AiOutlineUnderline'
import { BiHeading } from '@react-icons/all-files/bi/BiHeading'
import { DEFAULT_FONT, DEFAULT_FONT_SIZE, fonts, fontsMap } from '../fonts'
import { BiFont } from '@react-icons/all-files/bi/BiFont'
import { BiParagraph } from '@react-icons/all-files/bi/BiParagraph'
import clsx from 'clsx'
import { CustomText } from '../types'
import isNil from 'lodash/isNil'
import omitBy from 'lodash/omitBy'
import { BiLink } from '@react-icons/all-files/bi/BiLink'
import { BiAlignLeft } from '@react-icons/all-files/bi/BiAlignLeft'
import { BiAlignMiddle } from '@react-icons/all-files/bi/BiAlignMiddle'
import { BiAlignRight } from '@react-icons/all-files/bi/BiAlignRight'
import { BiAlignJustify } from '@react-icons/all-files/bi/BiAlignJustify'

const LIST_TYPES = ['numbered-list', 'bulleted-list']
const TEXT_ALIGN_TYPES = ['left', 'center', 'right', 'justify']

const setStyle = (editor: Editor, style: Partial<CSSProperties>) => {
  if (!editor || !editor.selection) return

  const selected = Editor.nodes(editor, {
    at: editor.selection,
    match: (n) => Text.isText(n),
  })

  Editor.withoutNormalizing(editor, () => {
    for (const [node, path] of selected) {
      const currentStyle = (node as CustomText).style || {}
      let newStyle = omitBy(Object.assign({}, currentStyle, JSON.parse(JSON.stringify(style))), isNil)
      if (Object.keys(newStyle).length === 0) {
        newStyle = undefined
      }
      let location: Location = path
      if (JSON.stringify(path) === JSON.stringify(Range.start(editor.selection).path)) {
        // start of selection -- we need to split how it's applied to this node at the start of the selection
        const selectedPartOfTheNode = {
          anchor: {
            path: path,
            offset: Range.start(editor.selection).offset,
          },
          focus: {
            path: path,
            offset: Editor.end(editor, path).offset,
          },
        }
        location = selectedPartOfTheNode
      } else if (JSON.stringify(path) === JSON.stringify(Range.end(editor.selection).path)) {
        // end of selection -- we need to split how it's applied to the end of the selection
        const selectedPartOfTheNode = {
          anchor: {
            path: path,
            offset: 0,
          },
          focus: {
            path: path,
            offset: Range.end(editor.selection).offset,
          },
        }
        location = selectedPartOfTheNode
      } else {
        // middle of selection -- the entire node is within the selection, we don't need to split it!
        location = path
      }

      if (!newStyle) {
        Transforms.unsetNodes(editor, 'style', { at: path, split: true })
      } else {
        Transforms.setNodes(editor, { style: newStyle }, { at: path, split: true })
      }
    }
  })
}

const setSelectionFont = (editor: Editor, fontName: string) => {
  if (!editor || !editor.selection) return
  const font = fontsMap[fontName]
  if (!font) return

  const fontsToSet = [font.name, ...font.fallback, font.type]
  setStyle(editor, {
    fontFamily: fontsToSet.join(', '),
  })
}

const setSelectionFontSize = (editor: Editor, fontSize: string) => {
  if (!editor || !editor.selection) return

  setStyle(editor, { fontSize })
}

export const toggleBlock = (editor, format) => {
  const isActive = isBlockActive(editor, format, TEXT_ALIGN_TYPES.includes(format) ? 'align' : 'type')
  const isList = LIST_TYPES.includes(format)

  Transforms.unwrapNodes(editor, {
    match: (n) =>
      !Editor.isEditor(n) &&
      SlateElement.isElement(n) &&
      LIST_TYPES.includes(n.type) &&
      !TEXT_ALIGN_TYPES.includes(format),
    split: true,
  })
  let newProperties: Partial<SlateElement>
  if (TEXT_ALIGN_TYPES.includes(format)) {
    newProperties = {
      align: isActive ? undefined : format,
    }
  } else {
    newProperties = {
      type: isActive ? 'paragraph' : isList ? 'list-item' : format,
    }
  }
  Transforms.setNodes<SlateElement>(editor, newProperties)

  if (!isActive && isList) {
    const block = { type: format, children: [] }
    Transforms.wrapNodes(editor, block)
  }
}

export const toggleLink = (editor, url) => {
  const isActive = isBlockActive(editor, 'a')

  Transforms.unwrapNodes(editor, { match: (n) => SlateElement.isElement(n) && n.type === 'a', split: true })
  if (url)
    Transforms.wrapNodes(
      editor,
      { type: 'a', href: url, target: '_blank', children: [] },
      { match: Text.isText, split: true }
    )
}

export const toggleFormat = (editor, format) => {
  const isActive = isFormatActive(editor, format)
  Transforms.setNodes(editor, { [format]: isActive ? null : true }, { match: Text.isText, split: true })
}

const isFormatActive = (editor, format) => {
  const [match] = Editor.nodes(editor, {
    match: (n) => n[format] === true,
    mode: 'all',
  })
  return !!match
}

const useSelectionFont = (): string[] => {
  const editor = useSlate()
  return useMemo(() => {
    const { selection, children } = editor
    if (!selection) return []

    const q = Editor.nodes(editor, {
      at: editor.selection,
      match: (n) => Text.isText(n),
    })
    let fonts: Record<string, boolean> = {}
    for (const [node, path] of q) {
      if ((node as CustomText).style?.fontFamily) fonts[(node as CustomText).style.fontFamily.split(',')[0]] = true
      else fonts[DEFAULT_FONT.name] = true
    }
    return Object.keys(fonts)
  }, [editor, editor.selection, editor.children])
}

const useSelectionFontSizes = (): number[] => {
  const editor = useSlate()
  return useMemo(() => {
    const { selection, children } = editor
    if (!selection) return []

    const q = Editor.nodes(editor, {
      at: editor.selection,
      match: (n) => Text.isText(n),
    })
    let sizes = new Set<number>()
    for (const [node, path] of q) {
      if ((node as CustomText).style?.fontSize)
        sizes.add(+String((node as CustomText).style.fontSize).replace('px', ''))
      else sizes.add(DEFAULT_FONT_SIZE)
    }
    return [...sizes.values()]
  }, [editor, editor.selection, editor.children])
}

const useSelectedBlockTypes = (editor: Editor): string[] =>
  useMemo(() => {
    const { selection, children } = editor
    if (!selection) return []

    const elements = []
    for (let i = Range.start(selection).path[0]; i < Range.end(selection).path[0]; i++) {
      if (i >= 0 && i < children.length) {
        elements.push(children[i])
      }
    }
    return elements.filter((e) => Element.isElement(e)).map((e) => e.type)
  }, [editor, editor.children, editor.selection])

const isBlockActive = (editor, format, blockType = 'type') => {
  const { selection } = editor
  if (!selection) return false

  const [match] = Array.from(
    Editor.nodes(editor, {
      at: Editor.unhangRange(editor, selection),
      match: (n) => !Editor.isEditor(n) && SlateElement.isElement(n) && n[blockType] === format,
    })
  )

  return !!match
}

export const Annotate = (props: PropsWithChildren & { annotation: React.ReactNode; low?: boolean }) => {
  return (
    <div className={'relative right-[2px] flex'}>
      {props.children}
      <span className={clsx('relative right-[6px] w-0 text-[10px]', props.low ? 'bottom-[4px]' : '-top-[4px]')}>
        {props.annotation}
      </span>
    </div>
  )
}

export const HoveringToolbar = () => {
  const ref = useRef<HTMLDivElement | null>()
  const editor = useSlate()
  const inFocus = useFocused()

  const { document, window } = useFrame()
  // const selectedBlockTypes = useSelectedBlockTypes(editor)
  const selectionFonts = useSelectionFont()
  const selectionFontSizes = useSelectionFontSizes()

  useEffect(() => {
    const el = ref.current
    const { selection } = editor

    if (!el) {
      return
    }

    if (!selection || !inFocus || Range.isCollapsed(selection) || Editor.string(editor, selection) === '') {
      el.removeAttribute('style')
      return
    }

    const domSelection = window.getSelection()
    if (domSelection.rangeCount < 1) return
    const domRange = domSelection.getRangeAt(0)
    const rect = domRange.getBoundingClientRect()

    el.style.opacity = '1'
    el.style.top = `${rect.bottom + window.pageYOffset + 130 - el.offsetHeight}px`
    el.style.left = `${rect.left + window.pageXOffset - el.offsetWidth / 2 + rect.width / 2}px`
  })

  const selectableFontSizes = [8, 9, 10, 11, 12, 14, 16, 18, 20, 24, 26, 28, 36, 48, 72]

  return (
    <Portal>
      <div
        ref={ref}
        className={
          'z-100 absolute -top-96 -left-96 -mt-1 rounded bg-slate-200 px-2 py-1 opacity-100 shadow-md transition-none'
        }
        style={/* this is overriden! */ undefined}
        onMouseDown={(e) => {
          // prevent toolbar from taking focus away from editor
          if ((e.target as HTMLElement).classList.contains('clickable')) return
          e.preventDefault()
        }}
      >
        <div className={'flex'}>
          <small className={'w-16 self-center pr-2 font-sans text-xs text-slate-400'}>Selection</small>
          <div className={'flex-1'}>
            <div className={'flex'}>
              <FormatButton format="bold" icon={<AiOutlineBold />} />
              <FormatButton format="italic" icon={<AiOutlineItalic />} />
              <FormatButton format="underline" icon={<AiOutlineUnderline />} />
              <FormatLinkButton />
              <div className={'flex-1'} />
            </div>
          </div>
        </div>
        <hr className={'m-0 border-0 border-t border-solid border-t-gray-300 p-0'} />
        <div className={'flex'}>
          <small className={'w-16 self-center pr-2 font-sans text-xs text-slate-400'}>Block</small>
          <div className={'flex flex-1 flex-col'}>
            <div className={'flex'}>
              <select
                className={'clickable m-0.5 h-6 rounded-sm border border-solid border-gray-400 bg-white'}
                value={selectionFonts.join(', ')}
                onChange={(e) => {
                  setSelectionFont(editor, e.target.value)
                }}
              >
                {selectionFonts.length > 1 && (
                  <option value={selectionFonts.join(', ')}>Multiple: {selectionFonts.join(', ')}</option>
                )}
                {fonts.map((f) => (
                  <option key={f.name} value={f.name}>
                    {f.name}
                  </option>
                ))}
              </select>
              <select
                className={'clickable m-0.5 h-6 rounded-sm border border-solid border-gray-400 bg-white'}
                value={selectionFontSizes.join(', ')}
                onChange={(e) => {
                  setSelectionFontSize(editor, `${e.target.value}px`)
                }}
              >
                {selectionFontSizes.length > 1 && (
                  <option value={selectionFontSizes.join(', ')}>{selectionFontSizes.join(', ')}</option>
                )}
                {selectionFontSizes.length === 1 &&
                  selectableFontSizes.findIndex((s) => s === selectionFontSizes[0]) === -1 && (
                    <option value={selectionFontSizes[0]}>{selectionFontSizes[0]}</option>
                  )}
                {selectableFontSizes.map((n) => (
                  <option key={n} value={n}>
                    {n}
                  </option>
                ))}
              </select>
              <Button
                reversed
                onClick={() => {
                  if (selectionFontSizes.length > 0) {
                    const newSize = 1 + selectionFontSizes.reduce((p, c) => Math.max(p, c), 0)
                    setSelectionFontSize(editor, `${newSize}px`)
                  }
                }}
              >
                <Annotate annotation={'+'}>
                  <BiFont />
                </Annotate>
              </Button>
              <Button
                reversed
                onClick={() => {
                  if (selectionFontSizes.length > 0) {
                    const newSize = -1 + selectionFontSizes.reduce((p, c) => Math.min(p, c), Number.MAX_VALUE)
                    setSelectionFontSize(editor, `${newSize}px`)
                  }
                }}
              >
                <Annotate annotation={'−'}>
                  <BiFont />
                </Annotate>
              </Button>
            </div>
            <div className={'flex'}>
              <BlockFormatButton format="p" icon={<BiParagraph />} />
              <BlockFormatButton
                format="h1"
                icon={
                  <span className={'font-sans text-xs tracking-tighter'}>
                    <BiHeading />1
                  </span>
                }
              />
              <BlockFormatButton
                format="h2"
                icon={
                  <span className={'font-sans text-xs tracking-tighter'}>
                    <BiHeading />2
                  </span>
                }
              />
              <BlockFormatButton
                format="h3"
                icon={
                  <span className={'font-sans text-xs tracking-tighter'}>
                    <BiHeading />3
                  </span>
                }
              />
              <BlockFormatButton
                format="h4"
                icon={
                  <span className={'font-sans text-xs tracking-tighter'}>
                    <BiHeading />4
                  </span>
                }
              />
              <BlockFormatButton
                format="h5"
                icon={
                  <span className={'font-sans text-xs tracking-tighter'}>
                    <BiHeading />5
                  </span>
                }
              />
              <BlockFormatButton
                format="h6"
                icon={
                  <span className={'font-sans text-xs tracking-tighter'}>
                    <BiHeading />6
                  </span>
                }
              />
            </div>
            <div className={'flex'}>
              <BlockFormatButton format={'left'} icon={<BiAlignLeft />} />
              <BlockFormatButton format={'center'} icon={<BiAlignMiddle />} />
              <BlockFormatButton format={'right'} icon={<BiAlignRight />} />
              <BlockFormatButton format={'justify'} icon={<BiAlignJustify />} />
            </div>
          </div>
        </div>
      </div>
    </Portal>
  )
}

interface BlockFormatButtonProps {
  format: string
  icon: React.ReactNode
}

const BlockFormatButton = (props: BlockFormatButtonProps) => {
  const editor = useSlate()
  return (
    <Button
      reversed
      active={isBlockActive(editor, props.format, TEXT_ALIGN_TYPES.includes(props.format) ? 'align' : 'type')}
      onClick={() => toggleBlock(editor, props.format)}
    >
      {props.icon}
    </Button>
  )
}

interface FormatButtonProps {
  format: string
  icon: React.ReactNode
}

const FormatButton = (props: FormatButtonProps) => {
  const editor = useSlate()
  return (
    <Button reversed active={isFormatActive(editor, props.format)} onClick={() => toggleFormat(editor, props.format)}>
      {props.icon}
    </Button>
  )
}

const FormatLinkButton = () => {
  const editor = useSlate()
  const isActive = isBlockActive(editor, 'a')
  return (
    <Button reversed active={isActive} onClick={() => toggleLink(editor, isActive ? undefined : prompt('Url?'))}>
      <BiLink />
    </Button>
  )
}
