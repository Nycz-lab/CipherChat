import React from 'react';

const ChatComponent = ({ chat }) => {
  return (
    <><div className="message__container">
      {chat.map((message, index) => (
        <div key={index} className="message__chats">
          <p style={{marginRight: "100%"}}>{message.payload.author}</p>
          <div className="message__recipient">
            <p>{message.payload.content}</p>
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
    </div>
        
      </>
    
  );
};

export default ChatComponent;