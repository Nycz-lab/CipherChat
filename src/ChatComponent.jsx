import React from 'react';

import { TextField } from '@mui/material';

const ChatComponent = ({ chat, contact }) => {
  return (
    <><div className="message__container">
      {contact !== "" && chat[contact] && chat[contact].map((message, index) => (
        <div key={index} className="message__chats">
          <p style={{ marginRight: "100%" }}>{message.author}</p>
          <div className="message__recipient">
            <p>{message.content.cleartext}</p>
          </div>
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
            id="login-password"
            onChange={(e) => setMessage(e.currentTarget.value)}
            placeholder="Enter your Message..."
            type="Message"
          />
    </div>
        
      </>
    
  );
};

export default ChatComponent;