import * as React from 'react';

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Container from '@mui/material/Container';

import TextField from '@mui/material/TextField';

import { ThemeProvider, createTheme } from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';

import { fetch } from '@tauri-apps/plugin-http';

import ToggleButton from '@mui/material/ToggleButton';
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup';

import { FaLock, FaLockOpen } from "react-icons/fa";
import { emit, listen } from '@tauri-apps/api/event';

import Connection from "./Connection";


function aesDash() {
  const [output, setOutput] = useState("");
  const [message, setMessage] = useState("");
  const [key, setKey] = useState("");
  const [mode, setMode] = useState("encrypt");


  const [animeFact, setAnimeFact] = useState({ anime: '', character: '', quote: '' });

  async function processMessage() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    try {
      if (mode === 'encrypt') {
        setOutput(await invoke("encrypt", { txt: message, key: key }));
      } else if (mode === 'decrypt') {
        setOutput(await invoke("decrypt", { txt: message, key: key }));
      }
    } catch (error) {
      console.log(error);
      toast({ title: 'Your Message didnt work', body: 'The Encryption or Decryption didnt work 😁!' });
    }

  }

  async function sendMsg() {
    let msgStruct = {
      message_type: 'string',
      message_content: message
    }
    invoke("send_msg", { msg: msgStruct });
  }

  async function test() {
    for (let index = 0; index < 1000; index++) {
      console.log(index);
      invoke("send_msg", { msg: output });
    }
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

  async function copyToClip() {
    try {
      await navigator.clipboard.writeText(output);
      console.log('Content copied to clipboard');
    } catch (err) {
      console.error('Failed to copy: ', err);
    }

  }

  async function closeConnection(){
    console.log("closing conn");
    let x = await invoke("closeConn", {});
    console.log(x);
  }


  useEffect(() => {
    fetch('https://animechan.xyz/api/random', {
      method: 'GET',
      timeout: 30,
    }).then((response) => {
      setAnimeFact(response.data);
    });

    const unlisten = listen("msg", (e) => {
      console.log(e);
      toast({ title: 'Message received!', body: e.payload.message_content });
    });

    return () => {
      unlisten.then(f => f());
    }


  }, []);


  const handleChange = (
    event,
    newAlignment
  ) => {
    setMode(newAlignment);
  };


  const [connection, setConnection] = useState({});


  const darkTheme = createTheme({
    palette: {
      mode: 'dark',
    },
  });



  return (
    <ThemeProvider theme={darkTheme}>
      <CssBaseline />

      <Connection connection={connection} setConnection={setConnection} />
      <div className="container">
        <h1>Welcome to CipherChat!</h1>


        <p>CipherChat helps you keep your convos private 😎.</p>


        <Container>

          <TextField
            id="msg-input"
            onChange={(e) => {setMessage(e.currentTarget.value);processMessage();}}
            placeholder="Enter a message..."
            label="Message"
          />
          <TextField
            id="key-input"
            onChange={(e) => setKey(e.currentTarget.value)}
            placeholder="Enter a Cryptographic Key..."
            label="Password"
            type="password"
          />




          <Stack style={{ margin: 'auto', width: '30%', padding: '10px' }} spacing={2} direction="row">
            <Button type="submit" onClick={() => processMessage()}>Crypt</Button>
            <Button variant="outlined" onClick={() => copyToClip()}>Copy</Button>
            <Button variant="outlined" onClick={() => sendMsg()}>Send</Button>
            <Button variant="outlined" onClick={() => closeConnection()}>Connect</Button>
            <ToggleButtonGroup
              color="primary"
              value={mode}
              exclusive
              onChange={handleChange}
              aria-label="Platform"
            >
              <ToggleButton value="encrypt"><FaLock /></ToggleButton>
              <ToggleButton value="decrypt"><FaLockOpen /></ToggleButton>
            </ToggleButtonGroup>
          </Stack>
          <p>{output}</p>
          <br />
          <p>{animeFact.quote}</p>
        </Container>
        
      </div>
    </ThemeProvider>
  );
}

export default aesDash;