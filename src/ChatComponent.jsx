import React from 'react';

import { TextField } from '@mui/material';
import { useEffect, useState, useRef } from "react";

import Message from './Message';

const ChatComponent = ({ chat, contact, message }) => {

  const image_types = ["image/apng", "image/avif", "image/gif", "image/jpeg", "image/png", "image/svg+xml", "image/webp"];
  const video_types = ["video/mp4"]

  const supported_mime_types = image_types.concat(video_types)

  useEffect(() => {
    document.getElementById("chatTextbox").scrollIntoView(true);
}, []);



  return (
    <>
    <div className="message__container">


      {contact !== "" && chat[contact] && chat[contact].map((chat_message, index) => {
        let payload = JSON.parse(chat_message.content.cleartext);
        if(payload.mime_type !== "text/plain"){
          let u8_2 = new Uint8Array(atob(payload.data).split("").map(function(c) {
              return c.charCodeAt(0); }));
          let binary = Uint8Array.from(u8_2);

          let url = "";

          if(supported_mime_types.includes(payload.mime_type)){

            let blob = new Blob([binary], {type: payload.mime_type});
            url = URL.createObjectURL(blob);
          }else{
            let blob = new Blob([binary], {type: "application/octet-stream"});
            url = URL.createObjectURL(blob);
            
          }
          
          
          payload.data = url;
        }
        return(

          <Message message={chat_message} index={index} payload={payload} image_types={image_types} video_types={video_types}/>
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