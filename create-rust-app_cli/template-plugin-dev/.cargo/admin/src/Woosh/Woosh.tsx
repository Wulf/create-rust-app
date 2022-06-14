import React, { createContext, useContext, useEffect, useMemo, useRef, useState } from 'react'
import mjml2html from 'mjml-browser'
import Frame, { useFrame } from 'react-frame-component'
import omitBy from 'lodash/omitBy'
import isNil from 'lodash/isNil'
import { ModifyParentMenu } from './ModifyParentMenu'
// eslint-disable-next-line import/no-webpack-loader-syntax
import rawCSS from '!!raw-loader!postcss-loader!./woosh.css'
import { Modal } from './Modal'
import clsx from 'clsx'
import { FaRegHandPointer } from '@react-icons/all-files/fa/FaRegHandPointer'
import { FaRegImage } from '@react-icons/all-files/fa/FaRegImage'
import { IoCodeSlash } from '@react-icons/all-files/io5/IoCodeSlash'
import { AiOutlineColumnHeight } from '@react-icons/all-files/ai/AiOutlineColumnHeight'
import { AiOutlineMinus } from '@react-icons/all-files/ai/AiOutlineMinus'
import { BsCursorText } from '@react-icons/all-files/bs/BsCursorText'
import { RichTextEditor } from './RichTextEditor/RichTextEditor'
import { fonts, fontsMap } from './fonts'
import { Overlay } from './Overlay'

/////////////////// Utility functions
export const updateChildSchema = <T extends ContainerElement>(
  onSchemaChange: (updatedParentSchema: T) => void,
  parentSchema: T,
  index: number,
  child: WooshElement[] | WooshElement | null
) => {
  if (!onSchemaChange) return
  const schema = JSON.parse(JSON.stringify(parentSchema))
  if (!child) {
    schema.children.splice(index, 1)
  } else if (Array.isArray(child)) {
    schema.children.splice(index, 1, ...child)
  } else {
    schema.children[index] = child
  }
  onSchemaChange(schema)
}
export const updateChild = <T extends ContainerElement>(
  parentSchema: T,
  index: number,
  child: WooshElement[] | WooshElement | null
): T => {
  const schema = JSON.parse(JSON.stringify(parentSchema))
  if (!child) {
    schema.children.splice(index, 1)
  } else if (Array.isArray(child)) {
    schema.children.splice(index, 1, ...child)
  } else {
    schema.children[index] = child
  }
  return schema
}
export const updateAttributes = <T extends WooshElement>(schema: T, attrs: Partial<T['attributes']>): T => {
  const clone = JSON.parse(JSON.stringify(schema)) as T

  const attrsClone = JSON.parse(JSON.stringify(attrs)) as Partial<T['attributes']>
  clone.attributes = Object.assign(clone.attributes, attrsClone) as T['attributes']

  return clone
}
export const updateSchema = <T extends WooshElement>(schema: T, update: Partial<T>): T => {
  const clone = JSON.parse(JSON.stringify(schema))

  return Object.assign(clone, update)
}

///////////////////

interface ElementBase {
  tagName: string
  attributes: Record<string, any>
}

export interface ContainerElement extends ElementBase {
  children: WooshElement[]
}

interface ContentElement extends ElementBase {
  content: string
}

// not implemented
// interface MJAccordion extends ElementBase {}
// interface MJCarousel extends ElementBase {}
// interface MJNavbar extends ElementBase {}
// interface MJTable extends ElementBase {}
// interface MJSocial extends ElementBase {}
// interface MJGroup extends ElementBase {}
// interface MJWrapper extends ElementBase {}

// implemented
export interface MJColumn extends ContainerElement {
  tagName: 'mj-column'
  attributes: {
    'background-color'?: string
    // 'inner-background-color'?: string // => not implemented
    border?: string
    'border-bottom'?: string
    'border-left'?: string
    'border-right'?: string
    'border-top'?: string
    'border-radius'?: string
    // 'inner-border'?: string          // => not implemented
    // 'inner-border-bottom'?: string
    // 'inner-border-left'?: string
    // 'inner-border-right'?: string
    // 'inner-border-top'?: string
    // 'inner-border-radius'?: string
    width?: string
    'vertical-align'?: string
    padding?: string
    'padding-top'?: string
    'padding-bottom'?: string
    'padding-left'?: string
    'padding-right'?: string
    'css-class'?: string
  }
}

export interface MJSection extends ContainerElement {
  tagName: 'mj-section'
  stack?: 'yes' | 'no' | '2x2'
  attributes: {
    'background-color'?: string
    'background-position'?: string
    'background-position-x'?: string
    'background-position-y'?: string
    'background-repeat'?: string
    'background-size'?: string
    'background-url'?: string
    border?: string
    'border-bottom'?: string
    'border-left'?: string
    'border-radius'?: string
    'border-right'?: string
    'border-top'?: string
    'css-class'?: string
    direction?: 'ltr' | 'rtl'
    'full-width'?: 'full-width'
    padding?: string
    'padding-bottom'?: string
    'padding-left'?: string
    'padding-right'?: string
    'padding-top'?: string
    'text-align'?: 'center' | 'justify' | 'left' | 'right'
  }
}

interface MJHero extends ContainerElement {
  tagName: 'mj-hero'
  attributes: {}
}

interface MJButton extends ContentElement {
  tagName: 'mj-button'
  attributes: {
    align?: string
    'background-color'?: string
    border?: string
    'border-bottom'?: string
    'border-left'?: string
    'border-radius'?: string
    'border-right'?: string
    'border-top'?: string
    color?: string
    'container-background-color'?: string
    'css-class'?: string
    'font-family'?: string
    'font-size'?: string
    'font-style'?: string
    'font-weight'?: string
    height?: string
    href?: string
    'inner-padding'?: string
    'letter-spacing'?: string
    'line-height'?: string
    padding?: string
    'padding-bottom'?: string
    'padding-left'?: string
    'padding-right'?: string
    'padding-top'?: string
    rel?: string
    target?: string
    'text-align'?: string
    'text-decoration'?: string
    'text-transform'?: string
    title?: string
    'vertical-align'?: string
    width?: string
  }
}

interface MJRaw extends ContentElement {
  tagName: 'mj-raw'
  attributes: {}
}

interface MJText extends ContentElement {
  tagName: 'mj-text'
  attributes: {
    color?: string
    'font-family'?: string
    'font-size'?: string
    'font-style'?: string
    'font-weight'?: string
    'line-height'?: string
    'letter-spacing'?: string
    height?: string
    'text-decoration'?: string
    'text-transform'?: 'capitalize' | 'lowercase' | 'uppercase'
    align?: 'center' | 'justify' | 'left' | 'right'
    'container-background-color'?: string
    padding?: string
    'padding-top'?: string
    'padding-bottom'?: string
    'padding-left'?: string
    'padding-right'?: string
    'css-class'?: string
  }
}

interface MJImage extends ContentElement {
  tagName: 'mj-image'
  attributes: {
    align?: 'center' | 'justify' | 'left' | 'right'
    alt?: string
    border?: string
    'border-top'?: string
    'border-bottom'?: string
    'border-left'?: string
    'border-right'?: string
    'border-radius'?: string
    'container-background-color'?: string
    'css-class'?: string
    'fluid-on-mobile'?: string
    height?: string
    href?: string
    name?: string
    padding?: string
    'padding-bottom'?: string
    'padding-left'?: string
    'padding-right'?: string
    'padding-top'?: string
    rel?: string
    sizes?: string
    src?: string
    srcset?: string
    target?: string
    title?: string
    // 'usemap'?: string // => not implemented
    width?: string
  }
}

interface MJDivider extends ElementBase {
  tagName: 'mj-divider'
  attributes: {
    'border-color'?: string
    'border-style'?: string
    'border-width'?: string
    'container-background-color'?: string
    'css-class'?: string
    padding?: string
    'padding-bottom'?: string
    'padding-left'?: string
    'padding-right'?: string
    'padding-top'?: string
    width?: string
    align?: 'center' | 'justify' | 'left' | 'right'
  }
}

interface MJSpacer extends ElementBase {
  tagName: 'mj-spacer'
  attributes: {
    'container-background-color'?: string
    'css-class'?: string
    height?: string
    padding?: string
    'padding-bottom'?: string
    'padding-left'?: string
    'padding-right'?: string
    'padding-top'?: string
  }
}

interface MJML extends ContainerElement {
  wooshVersion: 1
  tagName: 'mjml'
  fonts: string[]
  title: string
  previewLine: string
}

type WooshElement =
  | MJColumn
  | MJSection
  | MJHero
  | MJButton
  | MJRaw
  | MJText
  | MJImage
  | MJDivider
  | MJSpacer
  | ContainerElement
  | ContentElement
  | ElementBase
  | MJML

const computePadding = <
  T extends {
    'padding-top'?: string
    'padding-right'?: string
    'padding-bottom'?: string
    'padding-left'?: string
  }
>(
  attributes: T,
  defaults?: [string, string]
): [string, string, string, string] =>
  [
    attributes['padding-top'] || defaults?.['0'],
    attributes['padding-right'] || defaults?.['1'],
    attributes['padding-bottom'] || defaults?.['0'],
    attributes['padding-left'] || defaults?.['1'],
  ].map((n) => (n === undefined ? '0px' : n)) as [string, string, string, string]

const WooshSection = () => {
  const context = useWooshChildContext()
  const schema = context.schema as MJSection

  const padding = computePadding(schema.attributes, ['20px', '0px'])

  return (
    <div>
      <Overlay
        hoverMenuPosition={'before'}
        colour={'blue'}
        padding={padding}
        className={schema.attributes['css-class']}
        style={omitBy(
          {
            backgroundColor: schema.attributes['background-color'],
            backgroundPosition: schema.attributes['background-position'] || 'top center',
            backgroundPositionX: schema.attributes['background-position-x'],
            backgroundPositionY: schema.attributes['background-position-y'],
            backgroundRepeat: schema.attributes['background-repeat'] || 'repeat',
            backgroundSize: schema.attributes['background-size'] || 'auto',
            background: schema.attributes['background-url'] ? `url(${schema.attributes['background-url']})` : undefined,
            border: schema.attributes['border'] || 'none',
            borderBottom: schema.attributes['border-bottom'],
            borderLeft: schema.attributes['border-left'],
            borderRadius: schema.attributes['border-radius'],
            borderRight: schema.attributes['border-right'],
            borderTop: schema.attributes['border-top'],
            direction: schema.attributes['direction'] || 'ltr',
            width: schema.attributes['full-width'] ? '100%' : undefined,
            maxWidth: schema.attributes['full-width'] ? undefined : '600px',
            margin: '0 auto',
            padding: padding.join(' '),
            textAlign: schema.attributes['text-align'] || 'center',
          },
          isNil
        )}
      >
        <WooshChildren
          parentSchema={schema}
          childSchemas={schema.children}
          onSchemaChange={(index, child) => {
            updateChildSchema(context.onSchemaChange, schema, index, child)
          }}
        />
      </Overlay>
    </div>
  )
}

const WooshColumn = () => {
  const context = useWooshChildContext()
  const schema = context.schema as MJColumn

  const parentSchema = context.parentSchema as MJSection
  const numColumnsInSection = parentSchema.children.filter((c) => c.tagName === 'mj-column').length
  const numColumnsInGroup = parentSchema.stack === 'no' ? numColumnsInSection : parentSchema.stack === '2x2' ? 2 : 1

  const padding = computePadding(schema.attributes)

  return (
    <Overlay
      hoverMenuPosition={'after'}
      colour={'rose'}
      data-woosh-type={'column'}
      data-woosh-test={JSON.stringify(padding)}
      padding={padding}
      className={clsx(
        schema.attributes['css-class'],
        /* for mobile media query */
        numColumnsInGroup === 1
          ? 'w-full'
          : parentSchema.stack === 'no'
          ? `w-[${schema.attributes['width'] || `${100 / numColumnsInSection}%`}]`
          : `w-1/${numColumnsInGroup}`,
        // `mj:w-1/${numColumnsInSection}`,
        `mj:w-[${schema.attributes['width'] || `${100 / numColumnsInSection}%`}]`,
        schema.children.length === 0 && 'text-center text-gray-400 hover:text-rose-400'
      )}
      style={omitBy(
        {
          display: 'inline-block',
          backgroundColor: schema.attributes['background-color'],
          // innerBackgroundColor: schema.attributes['inner-background-color'],
          border: schema.attributes['border'],
          borderBottom: schema.attributes['border-bottom'],
          borderLeft: schema.attributes['border-left'],
          borderRight: schema.attributes['border-right'],
          borderTop: schema.attributes['border-top'],
          borderRadius: schema.attributes['border-radius'],
          // innerBorder: schema.attributes['inner-border'],
          // innerBorderBottom: schema.attributes['inner-border-bottom'],
          // innerBorderLeft: schema.attributes['inner-border-left'],
          // innerBorderRight: schema.attributes['inner-border-right'],
          // innerBorderTop: schema.attributes['inner-border-top'],
          // innerBorderRadius: schema.attributes['inner-border-radius'],
          verticalAlign: schema.attributes['vertical-align'] || 'top',
          padding: padding.join(' '),
        },
        isNil
      )}
    >
      <WooshChildren
        parentSchema={schema}
        childSchemas={schema.children}
        onSchemaChange={(index, child) => {
          updateChildSchema(context.onSchemaChange, schema, index, child)
        }}
      />
    </Overlay>
  )
}

const WooshRaw = () => {
  const context = useWooshChildContext()
  const schema = context.schema as MJRaw

  return (
    <div
      style={{
        display: 'table',
        width: '100%',
        color: '#000000',
        fontFamily: 'Ubuntu, Helvetica, Arial, sans-serif',
        fontSize: '13px',
        lineHeight: '1',
        textAlign: 'left',
      }}
      dangerouslySetInnerHTML={{ __html: schema.content }}
    />
  )
}

const WooshText = () => {
  const context = useWooshChildContext()
  const schema = context.schema as MJText

  return (
    <RichTextEditor
      className={schema.attributes['css-class']}
      style={omitBy(
        {
          // overflowWrap: 'anywhere',
          color: schema.attributes['color'] || '#000000',
          fontFamily: schema.attributes['font-family'] || 'Ubuntu, Helvetica, Arial, sans-serif',
          fontSize: schema.attributes['font-size'] || '13px',
          fontStyle: schema.attributes['font-style'],
          fontWeight: schema.attributes['font-weight'],
          lineHeight: schema.attributes['line-height'] || '1',
          letterSpacing: schema.attributes['letter-spacing'],
          height: schema.attributes['height'],
          textDecoration: schema.attributes['text-decoration'],
          textTransform: schema.attributes['text-transform'],
          textAlign: schema.attributes['align'] || 'left',
          backgroundColor: schema.attributes['container-background-color'],
          padding: schema.attributes['padding'] || '10px 25px',
          paddingTop: schema.attributes['padding-top'],
          paddingBottom: schema.attributes['padding-bottom'],
          paddingLeft: schema.attributes['padding-left'],
          paddingRight: schema.attributes['padding-right'],
        },
        isNil
      )}
      onChange={(html) => {
        const newSchema = JSON.parse(JSON.stringify(schema))
        newSchema.content = html
        context.onSchemaChange(newSchema)
      }}
      initialValue={schema.content}
    />
  )
}

const WooshButton = () => {
  const context = useWooshChildContext()
  const schema = context.schema as MJButton

  return (
    <>
      <a
        className={schema.attributes['css-class']}
        style={omitBy(
          {
            align: schema.attributes['align'] || 'center',
            backgroundColor: schema.attributes['background-color'] || '#414141',
            border: schema.attributes['border'] || 'none',
            borderBottom: schema.attributes['border-bottom'],
            borderLeft: schema.attributes['border-left'],
            borderRadius: schema.attributes['border-radius'] || '3px',
            borderRight: schema.attributes['border-right'],
            borderTop: schema.attributes['border-top'],
            color: schema.attributes['color'] || '#FFFFFF',
            containerBackgroundColor: schema.attributes['container-background-color'],
            fontFamily: schema.attributes['font-family'] || 'Ubuntu, Helvetica, Arial, sans-serif',
            fontSize: schema.attributes['font-size'] || '13px',
            fontStyle: schema.attributes['font-style'],
            fontWeight: schema.attributes['font-weight'] || 'normal',
            height: schema.attributes['height'],
            padding: schema.attributes['inner-padding'] || '10px 25px',
            letterSpacing: schema.attributes['letter-spacing'],
            lineHeight: schema.attributes['line-height'] || '120%',
            margin: schema.attributes['padding'] || '10px 25px',
            marginBottom: schema.attributes['padding-bottom'],
            marginLeft: schema.attributes['padding-left'],
            marginRight: schema.attributes['padding-right'],
            marginTop: schema.attributes['padding-top'],
            textAlign: schema.attributes['text-align'] || 'none',
            textDecoration: schema.attributes['text-decoration'] || 'none',
            textTransform: schema.attributes['text-transform'] || 'none',
            verticalAlign: schema.attributes['vertical-align'] || 'middle',
            width: schema.attributes['width'],
            display: 'inline-block',
            wordBreak: 'break-word',
          },
          isNil
        )}
        title={schema.attributes['title']}
        href={schema.attributes['href'] || '#'}
        rel={schema.attributes['rel']}
        target={schema.attributes['target'] || '_blank'}
        onClick={(e) => e.preventDefault()}
      >
        <RichTextEditor
          className=""
          style={{}}
          onChange={(html) => {
            const newSchema = JSON.parse(JSON.stringify(schema))
            newSchema.content = html
            context.onSchemaChange(newSchema)
          }}
          initialValue={schema.content}
        />
      </a>
    </>
  )
}

const WooshImage = () => {
  const context = useWooshChildContext()
  const schema = context.schema as MJImage

  return (
    <a
      className={clsx(schema.attributes['css-class'], 'text-decoration-none inline-block')}
      href={schema.attributes['href']}
      rel={schema.attributes['rel']}
      target={schema.attributes['target']}
      style={{
        verticalAlign: schema.attributes['align'] || 'center',
        width: schema.attributes['width'] || '100%',
        height: schema.attributes['height'] || 'auto',
        backgroundColor: schema.attributes['container-background-color'],
      }}
    >
      <img
        alt={schema.attributes['alt']}
        style={omitBy(
          {
            border: schema.attributes['border'] || 'none',
            borderTop: schema.attributes['border-top'] || 'none',
            borderBottom: schema.attributes['border-bottom'] || 'none',
            borderLeft: schema.attributes['border-left'] || 'none',
            borderRight: schema.attributes['border-right'] || 'none',
            borderRadius: schema.attributes['border-radius'],
            fluidOnMobile: schema.attributes['fluid-on-mobile'],
            name: schema.attributes['name'],
            padding: schema.attributes['padding'] || '10px 25px',
            paddingBottom: schema.attributes['padding-bottom'],
            paddingLeft: schema.attributes['padding-left'],
            paddingRight: schema.attributes['padding-right'],
            paddingTop: schema.attributes['padding-top'],
            // usemap: schema.attributes['usemap'],
            maxWidth: '100%',
            maxHeight: '100%',
          },
          isNil
        )}
        sizes={schema.attributes['sizes']}
        src={schema.attributes['src']}
        srcSet={schema.attributes['srcset']}
        title={schema.attributes['title']}
      />
    </a>
  )
}

const WooshDivider = () => {
  const context = useWooshChildContext()
  const schema = context.schema as MJDivider

  return (
    <p
      style={omitBy(
        {
          margin: 0,
          padding: schema.attributes['padding'] || '10px 25px',
          paddingBottom: schema.attributes['padding-bottom'],
          paddingLeft: schema.attributes['padding-left'],
          paddingRight: schema.attributes['padding-right'],
          paddingTop: schema.attributes['padding-top'],
          wordBreak: 'break-word',
          fontSize: '0px',
        },
        isNil
      )}
    >
      <div
        className={schema.attributes['css-class']}
        style={omitBy(
          {
            borderTopColor: schema.attributes['border-color'] || '#000',
            borderTopStyle: schema.attributes['border-style'] || 'solid',
            borderTopWidth: schema.attributes['border-width'] || '4px',
            backgroundColor: schema.attributes['container-background-color'],
            width: schema.attributes['width'] || '100%',
            textAlign: schema.attributes['align'] || 'center',
            fontSize: '1px',
            margin: '0 auto',
          },
          isNil
        )}
      />
    </p>
  )
}

const WooshSpacer = () => {
  const context = useWooshChildContext()
  const schema = context.schema as MJSpacer

  return (
    <div
      className={schema.attributes['css-class']}
      style={omitBy(
        {
          containerBackgroundColor: schema.attributes['container-background-color'],
          cssClass: schema.attributes['css-class'],
          height: schema.attributes['height'] || '20px',
          padding: schema.attributes['padding'],
          paddingBottom: schema.attributes['padding-bottom'],
          paddingLeft: schema.attributes['padding-left'],
          paddingRight: schema.attributes['padding-right'],
          paddingTop: schema.attributes['padding-top'],
        },
        isNil
      )}
    />
  )
}

export interface ElementEntry<S extends ElementBase = ElementBase> {
  getDefaultSchema: () => S
  humanName: string
  icon: React.ReactNode
  tagName: S['tagName']
  component: () => JSX.Element
  toMJML: (ctx: { elementsMap: Record<string, ElementEntry> }, schema: S) => string
}

interface WooshContext {
  elementsMap: Record<string, ElementEntry>
  elements: ElementEntry[]
}

const renderChildren = (ctx: { elementsMap: Record<string, ElementEntry> }, children: ElementBase[]) =>
  children.map((child) => ctx.elementsMap[child.tagName].toMJML(ctx, child)).join('')
const renderAttributes = (attributes: Record<string, any>) =>
  Object.keys(attributes)
    .filter((k) => !isNil(attributes[k]))
    .map((k) => `${k}="${attributes[k]}"`)
    .join(' ')
const rootElementToMJML = (ctx, schema) =>
  `<mjml><mj-head>${schema.fonts.map(
    (f) => `<mj-font name="${fontsMap[f].name}" href="${fontsMap[f].href}" />`
  )}<mj-title>${schema.title}</mj-title>${
    schema.previewLine ? `<mj-preview>${schema.previewLine}</mj-preview>` : ''
  }</mj-head><mj-body>${renderChildren(ctx, schema.children)}</mj-body></mjml>`

const elements: ElementEntry[] = [
  {
    humanName: 'Section',
    tagName: 'mj-section',
    getDefaultSchema: () => ({
      children: [],
      tagName: 'mj-section',
      attributes: {
        'padding-top': '20px',
        'padding-bottom': '20px',
        'padding-left': '0px',
        'padding-right': '0px',
      },
    }),
    component: WooshSection,
    toMJML: (ctx, s) => {
      const nColumns = s.children.filter((t) => t.tagName === 'mj-column').length
      if (s.stack === 'no' && nColumns !== 1) {
        return `<mj-section ${renderAttributes(s.attributes)}><mj-group>${renderChildren(
          ctx,
          s.children
        )}</mj-group></mj-section>`
      } else if (s.stack === '2x2' && nColumns === 4) {
        const columns = s.children.filter((t) => t.tagName === 'mj-column')
        const firstGroup = [columns[0], columns[1]].map((t) => ctx.elementsMap['mj-column'].toMJML(ctx, t)).join('')
        const secondGroup = [columns[2], columns[3]].map((t) => ctx.elementsMap['mj-column'].toMJML(ctx, t)).join('')

        return `<mj-section ${renderAttributes(
          s.attributes
        )}><mj-group>${firstGroup}</mj-group><mj-group>${secondGroup}</mj-group></mj-section>`
      }

      return `<mj-section ${renderAttributes(s.attributes)}>${renderChildren(ctx, s.children)}</mj-section>`
    },
  } as ElementEntry<MJSection>,
  {
    humanName: 'Column',
    tagName: 'mj-column',
    getDefaultSchema: () =>
      ({
        children: [],
        tagName: 'mj-column',
        attributes: {
          'vertical-align': 'middle',
        },
      } as MJColumn),
    component: WooshColumn,
    toMJML: (ctx, s) => `<mj-column ${renderAttributes(s.attributes)}>${renderChildren(ctx, s.children)}</mj-column>`,
  } as ElementEntry<MJColumn>,
  {
    humanName: 'Text',
    tagName: 'mj-text',
    icon: <BsCursorText className={'text-2xl'} />,
    getDefaultSchema: () => ({
      attributes: {},
      content:
        '<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.</p>',
      tagName: 'mj-text',
    }),
    component: WooshText,
    toMJML: (ctx, s) => `<mj-text ${renderAttributes(s.attributes)}>${s.content}</mj-text>`,
  } as ElementEntry<MJText>,
  {
    tagName: 'mj-button',
    humanName: 'Button',
    icon: <FaRegHandPointer className={'text-xl'} />,
    getDefaultSchema: () => ({
      content: '<span>Click Here</span>',
      attributes: {},
      tagName: 'mj-button',
    }),
    component: WooshButton,
    toMJML: (ctx, s) => `<mj-button ${renderAttributes(s.attributes)}>${s.content}</mj-button>`,
  } as ElementEntry<MJButton>,
  {
    humanName: 'Image',
    tagName: 'mj-image',
    component: WooshImage,
    icon: <FaRegImage className={'text-2xl'} />,
    getDefaultSchema: () => ({
      content: '',
      attributes: { src: 'https://create-rust-app.dev/img/cra-logo-rust.svg' },
      tagName: 'mj-image',
    }),
    toMJML: (ctx, s) => `<mj-image ${renderAttributes(s.attributes)}/>`,
  } as ElementEntry<MJImage>,
  {
    humanName: 'HTML',
    tagName: 'mj-raw',
    icon: <IoCodeSlash className={'text-2xl'} />,
    getDefaultSchema: () => ({
      content: '<p style="color: slategray">Your html here...</p>',
      attributes: { padding: 0, align: 'left' },
      tagName: 'mj-raw',
    }),
    component: WooshRaw,
    toMJML: (ctx, s) => `<mj-text ${renderAttributes(s.attributes)}>${s.content}</mj-text>`,
  } as ElementEntry<MJRaw>,
  {
    humanName: 'Divider',
    tagName: 'mj-divider',
    icon: <AiOutlineMinus className={'text-2xl'} />,
    component: WooshDivider,
    getDefaultSchema: () => ({
      content: '',
      attributes: {},
      tagName: 'mj-divider',
    }),
    toMJML: (ctx, s) => `<mj-divider ${renderAttributes(s.attributes)}/>`,
  } as ElementEntry<MJDivider>,
  {
    humanName: 'Space',
    tagName: 'mj-spacer',
    icon: <AiOutlineColumnHeight className={'text-2xl'} />,
    component: WooshSpacer,
    getDefaultSchema: () => ({
      content: '',
      attributes: {},
      tagName: 'mj-spacer',
    }),
    toMJML: (ctx, s) => `<mj-spacer ${renderAttributes(s.attributes)} />`,
  } as ElementEntry<MJSpacer>,
]
const elementsMap = elements.reduce((prev, curr) => {
  prev[curr.tagName] = curr
  return prev
}, {})

const WooshContext = createContext<WooshContext>({
  elementsMap,
  elements,
})
export const WooshProvider = WooshContext.Provider
export const useWooshContext = (): WooshContext => {
  return useContext(WooshContext)
}

interface WooshChildContext {
  parentSchema: ContainerElement
  schema: WooshElement
  onSchemaChange: (schema: WooshElement | null) => void
}

const WooshChildContext = createContext<WooshChildContext>({
  parentSchema: undefined,
  schema: { tagName: 'mjml', attributes: {}, children: [] },
  onSchemaChange: () => {
    /* noop */
  },
})
export const WooshChildProvider = WooshChildContext.Provider
export const useWooshChildContext = (): WooshChildContext => {
  return useContext(WooshChildContext)
}

const WooshUnknown = (props: { tagName: string }) => {
  return <div>Unknown '{props.tagName}' element</div>
}

interface WooshChildrenProps {
  parentSchema: ContainerElement
  childSchemas: WooshElement[]
  onSchemaChange: (index: number, value: WooshElement | null) => void
}

function WooshChildren(props: WooshChildrenProps) {
  const elements = useWooshContext().elementsMap

  return (
    <>
      {props.childSchemas.map((child, index) => {
        const Element = elements[child.tagName]?.component

        return (
          <WooshChildContext.Provider
            key={index}
            value={{
              parentSchema: props.parentSchema,
              schema: child,
              onSchemaChange: (newSchema) => props.onSchemaChange(index, newSchema),
            }}
          >
            {Element ? <Element /> : <WooshUnknown tagName={child.tagName} />}
          </WooshChildContext.Provider>
        )
      })}
    </>
  )
}

const JustMJML = (props: { schema: WooshSchema }) => {
  const { document, window } = useFrame()

  useEffect(() => {
    let mjml = rootElementToMJML({ elementsMap }, props.schema)
    const html = mjml2html(mjml).html

    document.body.innerHTML = html
    document.body.className = 'create-rust-app-mail'
  }, [JSON.stringify(props.schema)])

  return <>Loading...</>
}

export type WooshSchema = MJML

interface Props {
  schema: WooshSchema
  onSchemaChange?: (schema: WooshSchema) => void
}

export const Woosh = (props: Props): JSX.Element => {
  const [sectionModalOpen, setSectionModalOpen] = useState(false)

  const fontImportCSS = useMemo(
    () =>
      fonts
        .filter((f) => f.href)
        .map((f) => `@import url(${f.href});`)
        .join('\n'),
    [fonts]
  )

  if (!props.onSchemaChange) {
    const fontImportCSS = props.schema.fonts
      .map((f) => fontsMap[f])
      .filter((f) => f.href)
      .map((f) => `@import url(${f.href});`)
      .join('\n')

    return (
      <Frame
        style={{ width: '100%', height: '100%' }}
        head={
          <>
            <style>
              {`
                .create-rust-app-mail {
                  margin-top: 24px;
                  margin-bottom: 24px;
                  border-top: 1px solid #c4c4c4;
                  border-bottom: 1px solid #c4c4c4;
                }
              `}
              {fontImportCSS}
            </style>
          </>
        }
      >
        <JustMJML schema={props.schema} />
      </Frame>
    )
  }

  const addSection = (sectionChildren: ElementBase[]) => {
    const section = elementsMap['mj-section']
    const sectionSchema = section.getDefaultSchema()
    sectionSchema.children = sectionChildren
    const schema = JSON.parse(JSON.stringify(props.schema))
    schema.children.push(sectionSchema)
    props.onSchemaChange(schema)
    setSectionModalOpen(false)
  }

  const addSections = (amt: number) => {
    let col = elementsMap['mj-column']
    const schemas = [...new Array(amt)].map(() => col.getDefaultSchema())
    addSection(schemas)
  }

  return (
    <Frame
      style={{ width: '100%', height: '100%' }}
      head={
        <>
          <style type="text/css">{fontImportCSS}</style>
          <style type={'text/css'}>{`
           body {
             margin: 0;
             padding: 0;
           }
    
           * {
             box-sizing: border-box;
             transition: all .1s ease-out;
           }
           
           .mail-body {
             margin-top: 24px;
             margin-bottom: 24px;
             border-top: 1px solid #c4c4c4;
             border-bottom: 1px solid #c4c4c4;
           }
           ${rawCSS}
         `}</style>
        </>
      }
    >
      <div className={'mail-body'} style={{ maxWidth: '100%' }}>
        {props.schema.children?.length === 0 && (
          <div className={'text-center text-sm text-gray-400'}>
            <p>Looks like this email is empty -- there are no sections!</p>
            <p>Add one using the button below.</p>
          </div>
        )}
        <WooshChildren
          parentSchema={props.schema}
          childSchemas={props.schema.children}
          onSchemaChange={(index, child) => {
            updateChildSchema(
              (parentSchema) => {
                if (props.onSchemaChange) props.onSchemaChange(parentSchema as MJML)
              },
              props.schema,
              index,
              child
            )
          }}
        />
        {sectionModalOpen && (
          <Modal title={'Add a section'} onClose={() => setSectionModalOpen(false)}>
            <p>How many columns?</p>
            {[1, 2, 3, 4].map((n) => (
              <div
                key={n}
                onClick={() => addSections(n)}
                className={'cursor-pointer rounded px-2 py-1 hover:bg-slate-100'}
              >
                <div
                  className={
                    'mb-2 flex w-full select-none divide-x divide-y-0 divide-solid divide-blue-500 border border-solid border-blue-500 hover:divide-blue-700 hover:border-blue-700'
                  }
                >
                  {[...new Array(n)].map((_, i) => (
                    <div key={i} className={'flex-1 overflow-hidden'}>
                      &nbsp;
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </Modal>
        )}
        {!sectionModalOpen && (
          <div
            onClick={() => setSectionModalOpen(true)}
            className={
              'mt-8 cursor-pointer border border-solid border-transparent p-2 text-center text-xs text-blue-500 hover:border-blue-700 hover:text-blue-700'
            }
          >
            Add a section
          </div>
        )}
      </div>
    </Frame>
  )
}
