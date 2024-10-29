import React from 'react';

import { TextField } from '@mui/material';
import { useEffect, useState, useRef } from "react";

import Message from './Message';

import { writeFile, BaseDirectory } from '@tauri-apps/plugin-fs';

const ChatComponent = ({ chat, contact, message }) => {

  const image_types = ["image/apng", "image/avif", "image/gif", "image/jpeg", "image/png", "image/svg+xml", "image/webp"];
  const video_types = ["video/mp4", "video/webm", "video/mpeg"]

  const supported_mime_types = image_types.concat(video_types)

  useEffect(() => {
    document.getElementById("chatTextbox").scrollIntoView(true);
}, []);



  return (
    <>
    <div className="message__container">


      {contact !== "" && chat[contact] && chat[contact].map((chat_message, index) => {
        return(

          <Message key={chat_message.author === 'You' ? chat_message.message_id : chat_message.message_id + "-1"} message={chat_message} index={index} image_types={image_types} video_types={video_types}/>
        )
      })}
      
          
        

      {/* <div className="message__chats">
          <p className="sender__name">You</p>
          <div className="message__sender">
            <p>Hello there</p>
          </div>
        </div> */}

        

        {/*This is triggered when a user is typing*/}
        {/* <div className="message__status">
          <p>Someone is typing...</p>
        </div> */}

          <TextField
            autoComplete='off'
            style={{marginTop: "auto"}}
            id="chatTextbox"
            // onChange={(e) => setMessage(e.currentTarget.value)}
            inputRef={message}
            placeholder="Enter your Message..."
            type="Message"
          />
    </div>
        
      </>
    
  );
};

export default ChatComponent;