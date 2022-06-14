import React, { useState } from 'react'
import { useQuery, useQueryClient } from 'react-query'

import { Woosh, WooshSchema } from './Woosh/Woosh'
import { RichTextEditor } from './Woosh/RichTextEditor/RichTextEditor'
import { ModifyParentMenu } from './Woosh/ModifyParentMenu'

interface MailFile {
  file_path: string
  file_name_with_dir: string
  file_content: string
}

const useMailFiles = () =>
  useQuery<MailFile[]>(['mail', 'files'], () =>
    fetch('/api/development/mail_files', {
      headers: { 'Content-Type': 'application/json' },
    }).then((r) => r.json())
  )

export const MailPage = () => {
  const queryClient = useQueryClient()
  const mailFileQuery = useMailFiles()
  const [mail, setMail] = useState<MailFile[]>([])
  const [selectedMail, setSelectedMail] = useState<'new' | MailFile | undefined>()

  return <EditMailFile file={'new'} />
  // return <div className="flex h-full">
  //   <div className="p-4 border-r h-full" style={{width: 197}}>
  //     <div>Mail Files {mailFileQuery.isFetching &&
  //         <span className="text-gray-500">(Loading...)</span>}
  //     </div>
  //     <ul>
  //       {mailFileQuery.data && mailFileQuery.data.map(file => <li key={file.file_path} className={"flex"}>
  //         <button
  //           onClick={() => setSelectedMail(file)}
  //           disabled={selectedMail && selectedMail !== 'new' && file.file_name_with_dir === selectedMail.file_name_with_dir}
  //           className="flex-1 truncate text-left hover:underline text-blue-500 hover:text-blue-700 disabled:text-gray-500"
  //         >
  //           {selectedMail && selectedMail !== 'new' && selectedMail.file_name_with_dir === file.file_name_with_dir && "> "}{file.file_name_with_dir}
  //         </button>
  //       </li>)}
  //     </ul>
  //     <br/>
  //     <button
  //       onClick={() => queryClient.invalidateQueries(['mail', 'files'])}
  //       className={"text-btn"}
  //     >
  //       Reload
  //     </button>
  //   </div>
  //
  //   <div className={"flex-1 p-4"}>
  //     {!selectedMail && <div className={"text-gray-500"}>No mail file selected. <button className={'text-btn'} onClick={() => setSelectedMail('new')}>Create a new one?</button></div>}
  //     {selectedMail && <EditMailFile file={selectedMail}/>}
  //   </div>
  // </div>
}

const EditMailFile = (props: { file: 'new' | MailFile }) => {
  const [schema, setSchema] = useState<WooshSchema>({
    tagName: 'mjml',
    children: [
      // {
      //   tagName: 'mj-section',
      //   attributes: { 'padding-top': '0', 'padding-bottom': '0' },
      //   children: [
      //     {
      //       tagName: 'mj-column',
      //       attributes: {
      //         'padding-left': '32px',
      //         'padding-right': '32px',
      //       },
      //       children: [
      //         {
      //           tagName: 'mj-text',
      //           attributes: { align: 'center' },
      //           content: '<p>Happy Holidays! <a href="#" style="color: #ff4050;">View in your browser</a></p>',
      //         },
      //       ],
      //     },
      //   ],
      // },
      // {
      //   tagName: 'mj-section',
      //   attributes: { 'padding-bottom': '20px', 'padding-top': '20px' },
      //   children: [
      //     {
      //       tagName: 'mj-column',
      //       attributes: { 'vertical-align': 'middle', width: 100 / 3 + '%' },
      //       children: [
      //         {
      //           tagName: 'mj-image',
      //           attributes: { src: 'http://localhost:3000/images/mail-logo.png' },
      //           content: '',
      //         },
      //       ],
      //     },
      //     {
      //       tagName: 'mj-column',
      //       attributes: { 'vertical-align': 'middle', width: 200 / 3 + '%' },
      //       children: [
      //         {
      //           tagName: 'mj-text',
      //           attributes: {},
      //           content:
      //             '<p><a href="#">Women</a>&nbsp;&nbsp;&nbsp;&nbsp;<a href="#">Men</a>&nbsp;&nbsp;&nbsp;&nbsp;<a href="#">Girls</a>&nbsp;&nbsp;&nbsp;&nbsp;<a href="#">Boys</a></p>',
      //         },
      //       ],
      //     },
      //   ],
      // },
      // {
      //   tagName: 'mj-section',
      //   attributes: { 'padding-top': '20px', 'padding-bottom': '20px' },
      //   children: [
      //     {
      //       tagName: 'mj-column',
      //       attributes: {},
      //       children: [
      //         {
      //           tagName: 'mj-image',
      //           attributes: { src: 'http://localhost:3000/images/mail-1.png' },
      //           content: '',
      //         },
      //       ],
      //     },
      //   ],
      // },
      // {
      //   tagName: 'mj-section',
      //   attributes: { 'padding-top': '20px', 'padding-bottom': '20px' },
      //   children: [
      //     {
      //       tagName: 'mj-column',
      //       attributes: {},
      //       children: [
      //         {
      //           tagName: 'mj-image',
      //           attributes: { src: 'http://localhost:3000/images/mail-gift.png', width: '200px' },
      //           content: '',
      //         },
      //         {
      //           tagName: 'mj-text',
      //           attributes: { align: 'center' },
      //           content: `<h1 style="color: #FF4050;">'Tis the season</h1>`,
      //         },
      //         {
      //           tagName: 'mj-text',
      //           attributes: { align: 'center' },
      //           content: `<p>It’s that time of year again! With the holidays right around the corner, we have everything you’ll need.</p>`,
      //         },
      //       ],
      //     },
      //   ],
      // },
      // {
      //   tagName: 'mj-section',
      //   attributes: { 'padding-top': '20px', 'padding-bottom': '20px' },
      //   children: [
      //     {
      //       tagName: 'mj-column',
      //       attributes: {},
      //       children: [
      //         {
      //           tagName: 'mj-image',
      //           attributes: {
      //             'padding-top': '0px',
      //             'padding-left': '7px',
      //             'padding-right': '7px',
      //             'padding-bottom': '14px',
      //             src: 'http://localhost:3000/images/mail-bear.jpeg',
      //           },
      //           content: '',
      //         },
      //         {
      //           tagName: 'mj-image',
      //           attributes: {
      //             'padding-top': '0px',
      //             'padding-left': '7px',
      //             'padding-right': '7px',
      //             'padding-bottom': '14px',
      //             src: 'http://localhost:3000/images/mail-dino.jpeg',
      //           },
      //           content: '',
      //         },
      //         {
      //           tagName: 'mj-image',
      //           attributes: {
      //             'padding-top': '0px',
      //             'padding-left': '7px',
      //             'padding-right': '7px',
      //             'padding-bottom': '14px',
      //             src: 'http://localhost:3000/images/mail-robot.jpeg',
      //           },
      //           content: '',
      //         },
      //         {
      //           tagName: 'mj-image',
      //           attributes: {
      //             'padding-top': '0px',
      //             'padding-left': '7px',
      //             'padding-right': '7px',
      //             'padding-bottom': '14px',
      //             src: 'http://localhost:3000/images/mail-bear2.jpeg',
      //           },
      //           content: '',
      //         },
      //       ],
      //     },
      //     {
      //       tagName: 'mj-column',
      //       attributes: {},
      //       children: [
      //         {
      //           tagName: 'mj-image',
      //           attributes: {
      //             'padding-top': '0px',
      //             'padding-left': '7px',
      //             'padding-right': '7px',
      //             'padding-bottom': '14px',
      //             src: 'http://localhost:3000/images/mail-robot.jpeg',
      //           },
      //           content: '',
      //         },
      //         {
      //           tagName: 'mj-image',
      //           attributes: {
      //             'padding-top': '0px',
      //             'padding-left': '7px',
      //             'padding-right': '7px',
      //             'padding-bottom': '14px',
      //             src: 'http://localhost:3000/images/mail-bear2.jpeg',
      //           },
      //           content: '',
      //         },
      //         {
      //           tagName: 'mj-image',
      //           attributes: {
      //             'padding-top': '0px',
      //             'padding-left': '7px',
      //             'padding-right': '7px',
      //             'padding-bottom': '14px',
      //             src: 'http://localhost:3000/images/mail-dino.jpeg',
      //           },
      //           content: '',
      //         },
      //         {
      //           tagName: 'mj-image',
      //           attributes: {
      //             'padding-top': '0px',
      //             'padding-left': '7px',
      //             'padding-right': '7px',
      //             'padding-bottom': '14px',
      //             src: 'http://localhost:3000/images/mail-bear.jpeg',
      //           },
      //           content: '',
      //         },
      //       ],
      //     },
      //     {
      //       tagName: 'mj-column',
      //       attributes: {},
      //       children: [
      //         {
      //           tagName: 'mj-image',
      //           attributes: {
      //             'padding-top': '0px',
      //             'padding-left': '7px',
      //             'padding-right': '7px',
      //             'padding-bottom': '14px',
      //             src: 'http://localhost:3000/images/mail-dino.jpeg',
      //           },
      //           content: '',
      //         },
      //         {
      //           tagName: 'mj-image',
      //           attributes: {
      //             'padding-top': '0px',
      //             'padding-left': '7px',
      //             'padding-right': '7px',
      //             'padding-bottom': '14px',
      //             src: 'http://localhost:3000/images/mail-bear.jpeg',
      //           },
      //           content: '',
      //         },
      //         {
      //           tagName: 'mj-image',
      //           attributes: {
      //             'padding-top': '0px',
      //             'padding-left': '7px',
      //             'padding-right': '7px',
      //             'padding-bottom': '14px',
      //             src: 'http://localhost:3000/images/mail-bear2.jpeg',
      //           },
      //           content: '',
      //         },
      //         {
      //           tagName: 'mj-image',
      //           attributes: {
      //             'padding-top': '0px',
      //             'padding-left': '7px',
      //             'padding-right': '7px',
      //             'padding-bottom': '14px',
      //             src: 'http://localhost:3000/images/mail-robot.jpeg',
      //           },
      //           content: '',
      //         },
      //       ],
      //     },
      //     {
      //       tagName: 'mj-column',
      //       attributes: {},
      //       children: [
      //         {
      //           tagName: 'mj-image',
      //           attributes: {
      //             'padding-top': '0px',
      //             'padding-left': '7px',
      //             'padding-right': '7px',
      //             'padding-bottom': '14px',
      //             src: 'http://localhost:3000/images/mail-bear2.jpeg',
      //           },
      //           content: '',
      //         },
      //         {
      //           tagName: 'mj-image',
      //           attributes: {
      //             'padding-top': '0px',
      //             'padding-left': '7px',
      //             'padding-right': '7px',
      //             'padding-bottom': '14px',
      //             src: 'http://localhost:3000/images/mail-robot.jpeg',
      //           },
      //           content: '',
      //         },
      //         {
      //           tagName: 'mj-image',
      //           attributes: {
      //             'padding-top': '0px',
      //             'padding-left': '7px',
      //             'padding-right': '7px',
      //             'padding-bottom': '14px',
      //             src: 'http://localhost:3000/images/mail-bear.jpeg',
      //           },
      //           content: '',
      //         },
      //         {
      //           tagName: 'mj-image',
      //           attributes: {
      //             'padding-top': '0px',
      //             'padding-left': '7px',
      //             'padding-right': '7px',
      //             'padding-bottom': '14px',
      //             src: 'http://localhost:3000/images/mail-dino.jpeg',
      //           },
      //           content: '',
      //         },
      //       ],
      //     },
      //   ],
      // },
      // {
      //   /* 1-col section: find more gifts button */
      //   tagName: 'mj-section',
      //   attributes: { 'padding-top': '20px', 'padding-bottom': '20px' },
      //   children: [
      //     {
      //       tagName: 'mj-column',
      //       attributes: {},
      //       children: [{ tagName: 'mj-button', attributes: {}, content: '<span>Find More Gifts</span>' }],
      //     },
      //   ],
      // },
      // {
      //   /* 3-col (image + text each col) */
      //   tagName: 'mj-section',
      //   attributes: { 'padding-top': '20px', 'padding-bottom': '20px' },
      //   children: [
      //     {
      //       tagName: 'mj-column',
      //       attributes: {},
      //       children: [
      //         {
      //           tagName: 'mj-image',
      //           attributes: { src: 'http://localhost:3000/images/mail-snow.png' },
      //           content: '',
      //         },
      //         {
      //           tagName: 'mj-text',
      //           attributes: {},
      //           content: '<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor.</p>',
      //         },
      //       ],
      //     },
      //     {
      //       tagName: 'mj-column',
      //       attributes: {},
      //       children: [
      //         {
      //           tagName: 'mj-image',
      //           attributes: { src: 'http://localhost:3000/images/mail-bell.png' },
      //           content: '',
      //         },
      //         {
      //           tagName: 'mj-text',
      //           attributes: {},
      //           content: '<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor.</p>',
      //         },
      //       ],
      //     },
      //     {
      //       tagName: 'mj-column',
      //       attributes: {},
      //       children: [
      //         {
      //           tagName: 'mj-image',
      //           attributes: { src: 'http://localhost:3000/images/mail-bell2.png' },
      //           content: '',
      //         },
      //         {
      //           tagName: 'mj-text',
      //           attributes: {},
      //           content: '<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor.</p>',
      //         },
      //       ],
      //     },
      //   ],
      // },
      // {
      //   /* 1-col footer (4 social buttons, text) */
      //   tagName: 'mj-section',
      //   attributes: { 'padding-top': '20px', 'padding-bottom': '20px' },
      //   children: [
      //     {
      //       tagName: 'mj-column',
      //       attributes: {},
      //       children: [
      //         { tagName: 'mj-text', attributes: {}, content: '<p>...social buttons...</p>' },
      //         {
      //           tagName: 'mj-text',
      //           attributes: {},
      //           content:
      //             '<p style="text-align: center;">Inspired by Mailjet</p><p style="text-align: center; line-height: 25px;">We’ve contacted you because you’ve opted-in to receive news and promotional emails from shop.com. Click <a href="#">here</a> to unsubscribe.</p>',
      //         },
      //       ],
      //     },
      //   ],
      // },
    ],
    attributes: {},
    wooshVersion: 1,
    title: 'Title',
    previewLine: 'Preview text',
    fonts: ['Ubuntu'],
  })

  return (
    <div className={'flex h-full'}>
      <div className="flex-1 border-r-2">
        <Woosh schema={schema} onSchemaChange={setSchema} />
      </div>
      {/*<div className="flex-1">*/}
      {/*  <Woosh schema={schema} />*/}
      {/*</div>*/}
    </div>
  )
}
