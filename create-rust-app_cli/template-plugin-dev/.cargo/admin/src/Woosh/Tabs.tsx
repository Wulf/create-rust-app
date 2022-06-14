import React, {useState} from 'react'
import clsx from "clsx";

interface Props {
  tabs: string[]
  children: (tab: string) => React.ReactNode
}

export const Tabs = (props: Props) => {
  const [tab, setTab] = useState(props.tabs.length > 0 ? props.tabs[0] : '')

  return <div className={"select-none flex flex-col"}>
    <div className={"text-xs flex border-b border-blue-100"}>
      {props.tabs.map(t => (
        <div
          onClick={() => setTab(t)}
          className={clsx(
            "border-solid border-b-2 border-0 px-2 py-1",
            tab === t ? "border-blue-700 text-blue-700" : "text-blue-400 border-blue-400 hover:border-blue-500 hover:text-blue-500 active:border-blue-700 active:text-blue-700 cursor-pointer"
          )}
        >
          {t}
        </div>
      ))}
    </div>
    <div className={"flex-1 pt-1"}>
      {props.children(tab)}
    </div>
  </div>

}