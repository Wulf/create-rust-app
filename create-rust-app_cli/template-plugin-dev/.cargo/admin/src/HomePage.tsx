import React, { PropsWithChildren } from 'react'
import { AiFillMail } from '@react-icons/all-files/ai/AiFillMail'
import { AiFillDatabase } from '@react-icons/all-files/ai/AiFillDatabase'
import { Pages } from './index'

const FeatureButton = (props: { onClick: () => void } & PropsWithChildren) => {
  return (
    <div
      onClick={props.onClick}
      className={
        'flex cursor-pointer select-none items-center rounded-sm border border-blue-500 p-4 text-xl text-blue-500 hover:border-blue-700 hover:text-blue-700 active:border-blue-900 active:text-blue-900'
      }
    >
      {props.children}
    </div>
  )
}

export const HomePage = (props: { setPage: (page: Pages) => void }) => {
  return (
    <div className={'p-4'}>
      <strong>Admin portal</strong>
      <hr />
      Please note: the utilities here are to help in the development process and are not ready for production use cases
      yet!
      <div className={'mt-2 space-y-2'}>
        <FeatureButton onClick={() => props.setPage('database')}>
          <AiFillDatabase className={'mr-2'} />
          Database Editor (alpha release)
        </FeatureButton>
        <FeatureButton onClick={() => props.setPage('email')}>
          <AiFillMail className={'mr-2'} />
          Email Editor (alpha release)
        </FeatureButton>
      </div>
    </div>
  )
}
