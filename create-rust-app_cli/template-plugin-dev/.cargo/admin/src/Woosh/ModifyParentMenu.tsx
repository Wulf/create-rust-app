import React, { useState } from 'react'
import type { ContainerElement, ElementEntry, MJColumn, MJSection } from './Woosh'
import {
  updateAttributes,
  updateChild,
  updateChildSchema,
  updateSchema,
  useWooshChildContext,
  useWooshContext,
} from './Woosh'
import { IoMdSettings } from '@react-icons/all-files/io/IoMdSettings'
import { Modal, ModalHeading, ModalInput, ModalInputLabel } from './Modal'
import { IoMdAdd } from '@react-icons/all-files/io/IoMdAdd'
import { ImCross } from '@react-icons/all-files/im/ImCross'
import { Tabs } from './Tabs'
import { AiFillCaretLeft } from '@react-icons/all-files/ai/AiFillCaretLeft'
import { AiFillCaretRight } from '@react-icons/all-files/ai/AiFillCaretRight'
import { IoMdTrash } from '@react-icons/all-files/io/IoMdTrash'

interface PaddingSelectorProps {
  padding: [string, string, string, string]
  setPadding: (padding: [string, string, string, string]) => void
}

const PaddingSelector = ({ padding, setPadding }: PaddingSelectorProps) => {
  const sanitize = (input: string): string => {
    const pad = Math.max(0, parseInt(input))
    return `${pad}px`
  }

  return (
    <>
      <ModalHeading className={'text-center'}>Top</ModalHeading>
      <ModalInput
        className={'w-16 self-center'}
        type={'number'}
        min={0}
        step={1}
        value={parseInt(padding[0])}
        onChange={(e) => setPadding([sanitize(e.target.value), padding[1], padding[2], padding[3]])}
      />
      <div className={'flex'}>
        <div className={'flex-1'}>
          <ModalHeading className={'text-center'}>Left</ModalHeading>
          <ModalInput
            className={'w-16 self-center'}
            type={'number'}
            min={0}
            step={1}
            value={parseInt(padding[3])}
            onChange={(e) => setPadding([padding[0], padding[1], padding[2], sanitize(e.target.value)])}
          />
        </div>
        <div className={'flex-1'}>
          <ModalHeading className={'text-center'}>Right</ModalHeading>
          <ModalInput
            className={'w-16 self-center'}
            type={'number'}
            min={0}
            step={1}
            value={parseInt(padding[1])}
            onChange={(e) => setPadding([padding[0], sanitize(e.target.value), padding[2], padding[3]])}
          />
        </div>
      </div>
      <ModalHeading className={'text-center'}>Bottom</ModalHeading>
      <ModalInput
        className={'w-16 self-center'}
        type={'number'}
        min={0}
        step={1}
        value={parseInt(padding[2])}
        onChange={(e) => setPadding([padding[0], padding[1], sanitize(e.target.value), padding[3]])}
      />
    </>
  )
}

interface ColumnSelectorProps {
  stacking?: 'yes' | 'no' | '2x2'
  setStacking: (groups: 'yes' | 'no' | '2x2') => void
  columnWidths: string[]
  setColumnWidths: (widths: string[]) => void
}

const ColumnSelector = (props: ColumnSelectorProps) => {
  const { columnWidths, setColumnWidths, stacking, setStacking } = props
  const columns = columnWidths.length

  const columnWidthConfigurations = {
    2: [
      ['25%', '75%'],
      [`${100 / 3}%`, `${200 / 3}%`],
      ['50%', '50%'],
      [`${200 / 3}%`, `${100 / 3}%`],
      ['75%', `${100 / 3}%`],
    ],
    3: [
      [`${100 / 3}%`, `${100 / 3}%`, `${100 / 3}%`],
      ['50%', '25%', '25%'],
      ['25%', '50%', '25%'],
      ['25%', '25%', '50%'],
    ],
  }

  const nextWidthConfiguration = () => {
    const config = columnWidthConfigurations[columns]
    if (!config) return
    const index =
      config.findIndex((widthConfig: string[]) =>
        widthConfig.every((width, widthIndex) => width === columnWidths[widthIndex])
      ) || 0
    const nextIndex = index + 1
    setColumnWidths(config[nextIndex % config.length])
  }

  const previousWidthConfiguration = () => {
    const config = columnWidthConfigurations[columns]
    if (!config) return
    const index =
      config.findIndex((c) => {
        for (let i = 0; i < c.length; i++) {
          if (c[i] !== columnWidths[i]) return false
        }
        return true
      }) || 0
    let previousIndex = index - 1
    if (previousIndex < 0) previousIndex = config.length - 1
    setColumnWidths(config[previousIndex])
  }

  return (
    <>
      <ModalHeading>Number of columns</ModalHeading>
      <ModalInput
        min={1}
        step={1}
        max={4}
        type={'number'}
        value={columns}
        onChange={(e) => {
          const newNumColumns = Math.min(4, Math.max(1, Math.round(parseInt(e.target.value))))
          setColumnWidths([...new Array(newNumColumns)].fill(100 / newNumColumns).map((w) => `${w}%`))
        }}
      />
      {(columns === 2 || columns === 3) && (
        <>
          <ModalHeading>Column widths</ModalHeading>
          <div
            className={
              'flex h-6 cursor-pointer cursor-default items-center justify-center rounded bg-slate-100 px-2 py-1'
            }
          >
            <div
              onClick={(e) => {
                e.stopPropagation()
                previousWidthConfiguration()
              }}
              className={'h-4 w-8 text-blue-400 hover:text-blue-500 active:text-blue-700'}
            >
              {columnWidthConfigurations[columns] && (
                <div className={'cursor-pointer'}>
                  <AiFillCaretLeft />
                </div>
              )}
            </div>
            <div className={'flex-1'}>
              <div
                className={
                  'middle inline-block flex w-full select-none divide-x divide-y-0 divide-solid divide-gray-400 border border-solid border-gray-400 text-xs text-gray-400'
                }
              >
                {[...new Array(columns)].map((_, i) => (
                  <div style={{ width: columnWidths[i] || `${100 / columns}%` }} className={'overflow-hidden'}>
                    {parseFloat(columnWidths[i] || `${100 / columns}%`).toFixed(0)}%
                  </div>
                ))}
              </div>
            </div>
            <div
              onClick={(e) => {
                e.stopPropagation()
                nextWidthConfiguration()
              }}
              className={'h-4 w-8 text-blue-400 hover:text-blue-500 active:text-blue-700'}
            >
              {columnWidthConfigurations[columns] && (
                <div className={'cursor-pointer'}>
                  <AiFillCaretRight />
                </div>
              )}
            </div>
          </div>
        </>
      )}
      {columns > 1 && (
        <>
          <ModalHeading>On mobile devices,</ModalHeading>
          <div className={'flex flex-col items-start'}>
            <ModalInputLabel className={'flex w-full items-center text-left'}>
              <ModalInput
                onClick={() => setStacking('yes')}
                type={'radio'}
                name={'mobile-device-behaviour'}
                value={'yes'}
                checked={stacking === 'yes' || stacking === undefined}
              />
              <span className={'inline-block flex-1 pl-1 align-middle'}>stack columns vertically.</span>
            </ModalInputLabel>
            <ModalInputLabel className={'flex w-full items-center text-left'}>
              <ModalInput
                onClick={() => setStacking('no')}
                type={'radio'}
                name={'mobile-device-behaviour'}
                value={'no'}
                checked={stacking === 'no'}
              />
              <span className={'inline-block flex-1 pl-1 align-middle'}>keep columns side-by-side.</span>
            </ModalInputLabel>
            {columns === 4 && (
              <ModalInputLabel className={'flex w-full items-center text-left'}>
                <ModalInput
                  onClick={() => setStacking('2x2')}
                  type={'radio'}
                  name={'mobile-device-behaviour'}
                  value={'2x2'}
                  checked={stacking === '2x2'}
                />
                <span className={'inline-block flex-1 pl-1 align-middle'}>
                  stack the last two columns under the first two.
                </span>
              </ModalInputLabel>
            )}
          </div>
        </>
      )}
    </>
  )
}

interface ModifyParentMenuProps {
  isOpen: boolean
  innerProps: React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>
}

export const ModifyParentMenu = (props: ModifyParentMenuProps) => {
  const wooshCtx = useWooshContext()
  const ctx = useWooshChildContext()
  const [addModalOpen, setAddModalOpen] = useState(false)
  const [configureModalOpen, setConfigureModalOpen] = useState(false)
  const schema = ctx.schema as ContainerElement

  if (!props.isOpen && schema.children.length !== 0 && !addModalOpen && !configureModalOpen) return undefined

  if (schema.tagName === 'mj-section') {
    // const elementFilter = (e: ElementEntry): boolean => e.tagName === 'mj-column'
    let sectionSchema = schema as MJSection

    const padding = [
      sectionSchema.attributes['padding-top'],
      sectionSchema.attributes['padding-right'],
      sectionSchema.attributes['padding-bottom'],
      sectionSchema.attributes['padding-left'],
    ].map((t) => t ?? '0px') as [string, string, string, string]
    const setPadding = (padding: [string, string, string, string]) => {
      const newSchema = updateAttributes(sectionSchema, {
        'padding-top': padding[0],
        'padding-right': padding[1],
        'padding-bottom': padding[2],
        'padding-left': padding[3],
      })
      ctx.onSchemaChange(newSchema)
    }
    const columnWidths: string[] = schema.children
      .filter((c) => c.tagName === 'mj-column')
      .map((c) => (c as MJColumn).attributes['width'])
    const setColumns = (widths: string[]) => {
      const currentNumColumns = columnWidths.length
      const newNumColumns = widths.length

      let newSchema = sectionSchema

      if (currentNumColumns === newNumColumns) {
        /* do nothing */
      } else if (currentNumColumns < newNumColumns) {
        const col = wooshCtx.elementsMap['mj-column']
        const children = [...new Array(newNumColumns - currentNumColumns)].map(() => col.getDefaultSchema())
        newSchema = updateChild(newSchema, newSchema.children.length, children)
      } else if (currentNumColumns > newNumColumns) {
        for (let i = 0; i < currentNumColumns - newNumColumns; i++) {
          // remove ending element
          newSchema = updateChild(newSchema, newSchema.children.length - 1, null)
        }
      }

      for (let i = 0; i < newSchema.children.length; i++) {
        newSchema.children[i].attributes['width'] = newNumColumns === 4 ? undefined : widths[i]
      }

      if (currentNumColumns !== newNumColumns) {
        if (newNumColumns === 1) newSchema.stack = undefined
        else if (newNumColumns !== 4 && newSchema.stack === '2x2') newSchema.stack = 'yes'
      }

      ctx.onSchemaChange(newSchema)
    }
    const setStackOnMobile = (stack: 'yes' | 'no' | '2x2') => {
      console.log('stacking', stack)
      if (stack === 'yes' && (!sectionSchema.stack || sectionSchema.stack == 'yes')) return
      if (stack === 'no' && sectionSchema.stack === 'no') return
      if (stack === '2x2' && sectionSchema.stack === '2x2') return

      if (columnWidths.length === 1) {
        ctx.onSchemaChange(updateSchema(sectionSchema, { stack: undefined }))
      } else if (stack === '2x2' && columnWidths.length !== 4) {
        ctx.onSchemaChange(updateSchema(sectionSchema, { stack: 'yes' }))
      } else {
        ctx.onSchemaChange(updateSchema(sectionSchema, { stack }))
      }
    }

    return (
      <>
        <div {...props.innerProps} className={'relative -top-4 z-10 h-0 w-full text-center'}>
          <button
            className={
              'h-8 w-8 cursor-pointer rounded-full border border-solid border-slate-400 bg-white text-slate-400 hover:bg-slate-100 active:bg-slate-200'
            }
            onClick={() => {
              setConfigureModalOpen(true)
            }}
          >
            <IoMdSettings className={'align-middle text-xl'} />
          </button>
          <button
            className={
              'h-8 w-8 cursor-pointer rounded-full border border-solid border-slate-400 bg-white text-slate-400 hover:bg-slate-100 active:bg-slate-200'
            }
            onClick={() => {
              window.confirm('Delete?') && ctx.onSchemaChange(null)
            }}
          >
            <IoMdTrash className={'align-middle text-xl'} />
          </button>
        </div>
        {configureModalOpen && (
          <Modal
            title={'Configure section'}
            onClose={() => {
              setConfigureModalOpen(false)
            }}
          >
            <Tabs tabs={['Columns', 'Padding', 'Background']}>
              {(tab) => (
                <>
                  <div className={'h-48'}>
                    {tab === 'Columns' && (
                      <div className={'flex max-w-md flex-col'}>
                        <ColumnSelector
                          stacking={sectionSchema.stack}
                          setStacking={setStackOnMobile}
                          columnWidths={columnWidths}
                          setColumnWidths={setColumns}
                        />
                      </div>
                    )}
                    {tab === 'Padding' && (
                      <div className={'flex max-w-md flex-col'}>
                        <PaddingSelector padding={padding} setPadding={setPadding} />
                      </div>
                    )}
                    {tab === 'Background' && <div />}
                  </div>
                </>
              )}
            </Tabs>
          </Modal>
        )}
      </>
    )
  }

  if (schema.tagName === 'mj-column') {
    const elementFilter = (e: ElementEntry): boolean => typeof e.getDefaultSchema()['content'] !== 'undefined'
    let columnSchema = schema as MJColumn

    const padding = [
      columnSchema.attributes['padding-top'],
      columnSchema.attributes['padding-right'],
      columnSchema.attributes['padding-bottom'],
      columnSchema.attributes['padding-left'],
    ].map((t) => t ?? '0px') as [string, string, string, string]
    const setPadding = (padding: [string, string, string, string]) => {
      const newSchema = updateAttributes(columnSchema, {
        'padding-top': padding[0],
        'padding-right': padding[1],
        'padding-bottom': padding[2],
        'padding-left': padding[3],
      })
      ctx.onSchemaChange(newSchema)
    }

    return (
      <>
        {schema.children.length === 0 && (
          <div onClick={() => setAddModalOpen(true)} className={'cursor-pointer p-2 text-xs'}>
            + Add content
          </div>
        )}
        {addModalOpen && (
          <Modal
            title={'Add to column'}
            onClose={() => {
              setAddModalOpen(false)
            }}
          >
            <div className={'flex max-w-md flex-wrap'}>
              {wooshCtx.elements.filter(elementFilter).map((el) => (
                <div
                  key={el.tagName}
                  onClick={() => {
                    updateChildSchema(ctx.onSchemaChange, schema, schema.children.length, el.getDefaultSchema())
                    setAddModalOpen(false)
                  }}
                  className={
                    'm-1 flex h-16 w-14 cursor-pointer flex-col truncate rounded border border-solid border-blue-500 text-xs text-blue-500 hover:border-blue-700 hover:text-blue-700'
                  }
                >
                  <span className={'flex flex flex-1 items-center justify-center'}>{el.icon}</span>
                  <span>{el.humanName}</span>
                </div>
              ))}
              {/*<div*/}
              {/*  onClick={() => {*/}
              {/*    ctx.onSchemaChange(null)*/}
              {/*    setAddModalOpen(false)*/}
              {/*  }}*/}
              {/*  className={*/}
              {/*    'm-1 flex h-16 w-14 cursor-pointer flex-col truncate rounded border border-solid border-rose-500 text-xs text-rose-500 hover:border-rose-700 hover:text-rose-700'*/}
              {/*  }*/}
              {/*>*/}
              {/*  <span className={'flex flex flex-1 items-center justify-center'}>*/}
              {/*    <ImCross className={'text-md'} />*/}
              {/*  </span>*/}
              {/*  <span>Delete</span>*/}
              {/*</div>*/}
            </div>
          </Modal>
        )}
        {configureModalOpen && (
          <Modal
            title={'Configure column'}
            onClose={() => {
              setConfigureModalOpen(false)
            }}
          >
            {ctx.parentSchema?.children?.length > 2 && (
              <div className={'flex max-w-md flex-col'}>
                <div
                  onClick={() => {
                    ctx.onSchemaChange(null)
                    setConfigureModalOpen(false)
                  }}
                  className={
                    'mb-2 cursor-pointer rounded bg-white px-2 py-1 text-xs text-rose-500 hover:bg-rose-100 hover:text-rose-700'
                  }
                >
                  Delete
                </div>
              </div>
            )}
          </Modal>
        )}
        {schema.children.length > 0 && (
          <div {...props.innerProps} className={'relative -top-4 z-10 h-0'}>
            <button
              className={
                'h-8 w-8 cursor-pointer rounded-full border border-solid border-slate-400 bg-white text-slate-400 hover:bg-slate-100 active:bg-slate-200'
              }
              onClick={() => {
                setAddModalOpen(true)
              }}
            >
              <IoMdAdd className={'align-middle text-xl'} />
            </button>
            <button
              className={
                'h-8 w-8 cursor-pointer rounded-full border border-solid border-slate-400 bg-white text-slate-400 hover:bg-slate-100 active:bg-slate-200'
              }
              onClick={() => {
                setConfigureModalOpen(true)
              }}
            >
              <IoMdSettings className={'align-middle text-xl'} />
            </button>
          </div>
        )}
      </>
    )
  }

  return undefined
}
