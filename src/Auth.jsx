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

import ToggleButton from '@mui/material/ToggleButton';
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup';

import { FaLock, FaLockOpen } from "react-icons/fa";
import { emit, listen } from '@tauri-apps/api/event';

import Connection from "./Connection";


function Auth({token, setToken, user, setUser}) {
  // const [user, setUser] = useState("");
  const [password, setPassword] = useState("");


  async function login() {
    let msgStruct = {
      timestamp: Math.floor(Date.now()/1000),
      auth: {
        action: "login",
        user: user,
        password: password,
        message: ""
      },
      token: '',
      author: '',
      recipient: ''
    }
    invoke("login", { auth: msgStruct });
  }

  async function register() {
    let msgStruct = {
      timestamp: Math.floor(Date.now()/1000),
      auth: {
        action: "register",
        user: user,
        password: password,
        message: ""
      },
      token: '',
      author: '',
      recipient: '',
    }
    invoke("register", { auth: msgStruct });
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

  async function closeConnection(){ //TODO buggy apparently
    console.log("closing conn");
    let x = await invoke("close_conn", {});
    console.log(x);
  }


  useEffect(() => {

    // const unlisten = listen("msg", (e) => {
    //   console.log(e);
    //   toast({ title: 'Message received!', body: e.payload.message_content });
    // });

    const unlisten = listen("register_token", (e) => {
      console.log(e);
      toast({ title: 'Register Token!', body: e.payload.message_content });
      setToken(e.payload.token);
    });

    return () => {
      unlisten.then(f => f());
    }


  }, []);


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
            id="login-username"
            onChange={(e) => {setUser(e.currentTarget.value);}}
            placeholder="Enter a Username..."
            label="Username"
          />
          <TextField
            id="login-password"
            onChange={(e) => setPassword(e.currentTarget.value)}
            placeholder="Enter your Password..."
            label="Password"
            type="password"
          />




          <Stack style={{ margin: 'auto', width: '30%', padding: '10px' }} spacing={2} direction="row">
            <Button variant="outlined" onClick={() => login()}>Login</Button>
            <Button variant="outlined" onClick={() => register()}>Register</Button>
            <Button variant="outlined" onClick={() => closeConnection()}>Connect</Button>
          </Stack>
        </Container>
        
      </div>
    </ThemeProvider>
  );
}

export default Auth;