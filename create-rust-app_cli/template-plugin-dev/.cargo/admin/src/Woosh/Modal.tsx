import React from 'react'
import clsx from 'clsx'

export const ModalInputLabel = ({
  className,
  ...props
}: React.DetailedHTMLProps<React.LabelHTMLAttributes<HTMLLabelElement>, HTMLLabelElement>) => {
  return (
    <label
      {...props}
      className={(className ?? '') + ' ' + 'm-0 inline-block h-6 p-0 align-middle text-xs text-gray-400'}
    />
  )
}

export const ModalInput = ({
  className,
  ...props
}: React.DetailedHTMLProps<React.InputHTMLAttributes<HTMLInputElement>, HTMLInputElement>) => {
  return (
    <input
      {...props}
      className={clsx(
        className,
        'm-0 inline-block h-6 rounded-sm border border-solid border-gray-400 p-1 align-middle focus:border-blue-400 focus:outline-none'
      )}
    />
  )
}

export const ModalHeading = ({
  className,
  children,
  ...props
}: React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>) => (
  <div {...props} className={clsx('mt-2 mb-1 text-left text-xs text-gray-400', className)}>
    {children}
  </div>
)

export const Modal = (props: {
  title: string
  onClose: () => void

  wrapperProps?: React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>
  onSubmit?: () => void
  submitText?: string

  children: React.ReactNode
}) => {
  return (
    <div
      {...props.wrapperProps}
      className={
        'fixed top-0 left-0 right-0 bottom-0 z-20 flex h-full w-full flex-col justify-end bg-slate-100 bg-opacity-50 text-center font-sans'
      }
      onClick={(e) => e.target === e.currentTarget && props.onClose()}
    >
      <div className={'mx-auto inline-block rounded bg-white p-4 shadow-md'}>
        <h1 className={'m-0 mb-4 p-0 text-left text-xs text-gray-400'}>{props.title}</h1>
        <div>{props.children}</div>
      </div>
    </div>
  )
}
