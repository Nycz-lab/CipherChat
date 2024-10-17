import * as React from 'react';

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';

import Button from '@mui/material/Button';
import Container from '@mui/material/Container';
import Stack from '@mui/material/Stack';

import TextField from '@mui/material/TextField';

import { ThemeProvider, createTheme } from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';

import { emit, listen } from '@tauri-apps/api/event';

import ChatComponent from "./ChatComponent";


function Chat({token, setToken, user}) {
  const [recipient, setRecipient] = useState("");
  const [message, setMessage] = useState("");

  const [chat, setChat] = useState([]);


  async function sendMessage(){

    let msgStruct = {
      content: {
        ciphertext: '',
        nonce: '',
        cleartext: message
      },
      timestamp: Math.floor(Date.now()/1000),
      auth: null,
      token: token,
      author: 'You',
      recipient: recipient
    }

    invoke("send_msg", { msg: msgStruct });
    setChat(chat => [...chat, {payload: msgStruct}]);
  }

  async function toast(options) {
    let permissionGranted = await isPermissionGranted();
    if (!permissionGranted) {
      const permission = await requestPermission();
      permissionGranted = permission === 'granted';
    }
    if (permissionGranted) {
      sendNotification(options);
    }
  }

  useEffect(() => {

    const unlisten = listen("msg", (e) => {
      console.log(e);
      if(e.payload.content !== null && e.payload.content !== undefined){
        toast({ title: 'Message received!', body: e.payload.content.cleartext });

        setChat(chat => [...chat, e]);

        console.log(chat);
      }
      
    });


    return () => {
      unlisten.then(f => f());
    }


  }, []);



  const darkTheme = createTheme({
    palette: {
      mode: 'dark',
    },
  });



  return (
    <ThemeProvider theme={darkTheme}>
      <CssBaseline />

      <div className="container">


        <Container>


        <ChatComponent chat={chat}/>

          <TextField
            id="recipient"
            onChange={(e) => {setRecipient(e.currentTarget.value);}}
            placeholder="Enter a Username..."
            label="Recipient"
          />
          <TextField
            id="login-password"
            onChange={(e) => setMessage(e.currentTarget.value)}
            placeholder="Enter your Message..."
            type="Message"
          />




          <Stack style={{ margin: 'auto', width: '30%', padding: '10px' }} spacing={2} direction="row">
            <Button variant="outlined" onClick={() => sendMessage()}>Send</Button>
          </Stack>
        </Container>
        
      </div>
    </ThemeProvider>
  );
}

export default Chat;