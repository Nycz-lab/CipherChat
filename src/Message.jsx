import React from 'react';

import { TextField } from '@mui/material';
import { useEffect, useState, useRef } from "react";

const Message = ({ message, index, payload, image_types, video_types }) => {

    let div_css_class = message.author !== "You" ? 'flex-start' : 'flex-end';
    let message_css_class = message.author !== "You" ? "message message__recipient" : "message message__sender";

    let specific_css = " message__text";

    const img_elem = <img className="image" src={payload.data}/>;
    const vid_elem = <video className="image" src={payload.data} controls></video>;

    const txt_elem = <p>{payload.data}</p>;

    let cur_elem = txt_elem;

    switch(payload.mime_type){
        case 'text/plain':
            cur_elem = txt_elem;
            specific_css = " message__text";
            break;
        default:
            cur_elem = txt_elem;
            specific_css = " message__text";
            break;
    }

    if(image_types.includes(payload.mime_type)){
        cur_elem = img_elem;
        specific_css = " message__image";

    }else if(video_types.includes(payload.mime_type)){
        cur_elem = vid_elem;
        specific_css = " message__image";
    }

    console.log(message);

  return (
    <>
        
            <div key={"chats-" + index} className="message__chats" style={{ alignItems: div_css_class }}>
                <p key={index}>{message.author}</p>
                <div className={message_css_class + specific_css}>
                    {cur_elem}
                </div>
            </div>
      </>
    
  );
};

export default Message;