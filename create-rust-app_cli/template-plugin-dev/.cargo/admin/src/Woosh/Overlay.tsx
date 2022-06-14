import React, { useEffect, useRef, useState } from 'react'
import clsx from 'clsx'
import { ModifyParentMenu } from './ModifyParentMenu'

export const Overlay = ({
  colour,
  padding,
  className,
  hoverMenuPosition,
  children,
  ...props
}: {
  hoverMenuPosition: 'before' | 'after'
  colour: string
  padding: [string, string, string, string]
} & React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>) => {
  const ref = useRef<HTMLSpanElement>()
  const [isHovering, setHovering] = useState(false)
  const [isHoveringConfig, setConfigHover] = useState(false)
  useEffect(() => {
    if (!ref.current) return
    const el = ref.current
    el.style.setProperty('--top', padding[0])
    el.style.setProperty('--right', padding[1])
    el.style.setProperty('--bottom', padding[2])
    el.style.setProperty('--left', padding[3])
  })

  const showOverlay = isHovering || isHoveringConfig

  return (
    <>
      {hoverMenuPosition == 'before' && (
        <ModifyParentMenu
          innerProps={{
            onMouseOver: () => setConfigHover(true),
            onMouseOut: () => setConfigHover(false),
          }}
          isOpen={showOverlay}
        />
      )}
      <div
        onMouseOver={() => setHovering(true)}
        onMouseOut={() => setHovering(false)}
        {...props}
        className={clsx('group relative', className)}
      >
        <span className={'relative'}>
          {children}
          {hoverMenuPosition === 'after' && (
            <ModifyParentMenu
              innerProps={{
                onMouseOver: () => setConfigHover(true),
                onMouseOut: () => setConfigHover(false),
              }}
              isOpen={showOverlay}
            />
          )}
        </span>

        {showOverlay && (
          <>
            {/* border+background to show width/height */}
            <span
              className={`pointer-events-none absolute bg-${colour}-700 inset-0 block border-2 border-solid bg-opacity-10 transition-none border-${colour}-500`}
            />
          </>
        )}

        {showOverlay && (
          <>
            {/* dashed lines indicating padding */}
            <span
              ref={ref}
              className={`pointer-events-none absolute inset-1 block border border-dashed transition-none border-${colour}-700`}
              style={{
                top: 'var(--top)',
                right: 'var(--right)',
                left: 'var(--left)',
                bottom: 'var(--bottom)',
              }}
            />
          </>
        )}
      </div>
    </>
  )
}
