import React, {cloneElement, useState} from "react"
import {
  autoPlacement,
  autoUpdate,
  FloatingFocusManager,
  Placement,
  useClick,
  useDismiss,
  useFloating,
  useInteractions,
  useRole
} from "@floating-ui/react-dom-interactions"

interface Props {
  render: () => React.ReactNode
  open: boolean,
  setOpen: (o: boolean) => void
  placement?: Placement
  children: JSX.Element
}

export const Popover = ({open, setOpen, children, render, placement}: Props) => {

  const {x, y, reference, floating, strategy, context} = useFloating({
    open,
    onOpenChange: setOpen,
    middleware: [autoPlacement()],
    placement,
    whileElementsMounted: autoUpdate
  })

  const {getReferenceProps, getFloatingProps} = useInteractions([
    useClick(context),
    useRole(context),
    useDismiss(context)
  ])

  return (
    <>
      {cloneElement(
        children,
        getReferenceProps({ref: reference, ...children.props})
      )}
      <FloatingFocusManager
        context={context}
        modal={false}
        order={["reference", "content"]}
        returnFocus={false}
      >
        <div
          {...getFloatingProps({
            ref: floating,
            style: {
              position: strategy,
              top: y ?? 0,
              left: x ?? 0,
              pointerEvents: open ? 'auto' : 'none',
              opacity: open ? 1 : 0
            }
          })}
        >
          {render()}
        </div>
      </FloatingFocusManager>
    </>
  )
}
