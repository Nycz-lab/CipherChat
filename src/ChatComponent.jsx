import React from 'react';

import { TextField } from '@mui/material';
import { useEffect, useState, useRef } from "react";

const ChatComponent = ({ chat, contact, message }) => {

  useEffect(() => {
    document.getElementById("chatTextbox").scrollIntoView(true);
}, []);

  return (
    <>
    <div className="message__container">


      {contact !== "" && chat[contact] && chat[contact].map((message, index) => (
        <div key={index} className="message__chats">
          {message.author !== "You" && 
          <><p style={{ marginRight: "100%" }}>{message.author}</p><div className="message__recipient">
              <p>{message.content.cleartext}</p>
            </div></>
          }

          {message.author == "You" && 
          <><p style={{ marginLeft: "95%" }}>{message.author}</p><div className="message__sender">
              <p>{message.content.cleartext}</p>
            </div></>
          }
          
        </div>
      ))}
      
          
        

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