//
// License warning: this file contains modified code from the slatejs editor
//                  example and thus some of it may be exclusively licensed
//                  under the MIT license without the option of the APACHE2
//                  license as in the create-rust-app repository.
//
// See https://github.com/ianstormtaylor/slate/blob/main/License.md
//
import React, { Ref, PropsWithChildren } from 'react'
import ReactDOM from 'react-dom'
import clsx from 'clsx'
import { useFrame } from 'react-frame-component'

interface BaseProps {
  className: string
  [key: string]: unknown
}
type OrNull<T> = T | null

interface SelectProps {
  value: string
  onChange: (value: string) => void
  children: React.ReactNode
}

export const Select = (props: SelectProps) => {
  return (
    <select
      value={props.value}
      className={'clickable m-0.5 h-6 rounded-sm border border-solid border-gray-400 bg-white'}
      onChange={(e) => {
        props.onChange(e.target.value)
      }}
    >
      {props.children}
    </select>
  )
}

export const Button = React.forwardRef(
  (
    { className, active, reversed, ...props }: PropsWithChildren<{ active: boolean; reversed: boolean } & BaseProps>,
    ref: Ref<OrNull<HTMLSpanElement>>
  ) => (
    <span
      {...props}
      ref={ref}
      className={clsx(
        className,
        'relative m-0.5 flex h-6 w-6 cursor-pointer items-center justify-center rounded-sm border border-solid border-gray-400 border-opacity-0 hover:border-opacity-100 active:top-2 active:bg-violet-600 active:ring active:ring-inset active:ring-black',
        reversed
          ? active
            ? 'bg-slate-400 text-white'
            : 'text-slate-500'
          : active
          ? 'bg-slate-600 text-black'
          : 'text-slate-300'
      )}
    />
  )
)

export const Icon = React.forwardRef(
  ({ className, ...props }: PropsWithChildren<BaseProps>, ref: Ref<OrNull<HTMLSpanElement>>) => (
    <span
      {...props}
      ref={ref}
      className={clsx('material-icons', className)}
      style={{
        fontSize: '18px',
        verticalAlign: 'text-bottom',
      }}
    />
  )
)

export const Portal = ({ children }) => {
  const { document, window } = useFrame()
  return typeof document === 'object' ? ReactDOM.createPortal(children, document.body) : null
}
