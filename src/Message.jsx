import React from 'react';

import { TextField } from '@mui/material';
import { useEffect, useState, useRef } from "react";

import { readFile, BaseDirectory } from '@tauri-apps/plugin-fs';


const Message = ({ message, index, image_types, video_types }) => {

    let payload = message.content.cleartext;

    let div_css_class = message.author !== "You" ? 'flex-start' : 'flex-end';
    let message_css_class = message.author !== "You" ? "message message__recipient" : "message message__sender";

    let specific_css = " message__text";

    const [url, setUrl] = useState("");
    const [output_message, setOutput_message] = useState("");

    
    const img_elem = <img className="image" src={url}/>;
    const vid_elem = <video className="image" src={url} controls></video>;
    
    const txt_elem = <p>{output_message}</p>;
    const [cur_elem, setCur_elem] = useState(txt_elem);

    useEffect(() => {

        async function run(){
            if(payload.mime_type !== "text/plain" && url === ""){
                genUrl();
            }
            switch(payload.mime_type){
                case 'text/plain':
                    setOutput_message(payload.data);
                    setCur_elem(txt_elem);
                    specific_css = " message__text";
                    break;
                default:
                    setOutput_message("something went wrong...");
                    setCur_elem(txt_elem);
                    specific_css = " message__text";
                    break;
            }
        
            if(image_types.includes(payload.mime_type)){
                console.log("found image");
                setCur_elem(img_elem);
                specific_css = " message__image";
        
            }else if(video_types.includes(payload.mime_type)){
                setCur_elem(vid_elem);
                specific_css = " message__image";
            }
        }
        run();
    }, [url, output_message]);

    const genUrl = async () => {
        if(payload.mime_type !== "text/plain"){
            const data = await readFile(payload.data);
            let blob = new Blob([data], {type: payload.mime_type});
            setUrl(URL.createObjectURL(blob));
        }
    }

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