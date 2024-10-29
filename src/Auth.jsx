import * as React from 'react';

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Container from '@mui/material/Container';

import TextField from '@mui/material/TextField';
import MenuItem from '@mui/material/MenuItem';
import { Autocomplete } from '@mui/material';


import { ThemeProvider, createTheme } from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';

import ToggleButton from '@mui/material/ToggleButton';
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup';

import { FaLock, FaLockOpen } from "react-icons/fa";
import { emit, listen } from '@tauri-apps/api/event';

import Connection from "./Connection";

import { load } from '@tauri-apps/plugin-store';
import SHA256 from 'crypto-js/sha256';

import { ToastContainer, toast, Bounce } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';


function Auth({token, setToken, user, setUser, connection, setConnection}) {
  // const [user, setUser] = useState("");
  const [password, setPassword] = useState("");

  const wrongLoginHelp = ({ closeToast, toastProps }) => (
    <div>
      You cant login because your device is missing the users keybundle 🥲<br/>
      <button>Help</button>
      <button onClick={closeToast}>Close</button>
    </div>
  );

  async function login() {

    const hash = SHA256(connection.host).toString();
    const credentials = await load(`${hash}/credentials.bin`, { autoSave: 0 });

    if(!((await credentials.keys()).includes(user))){
      toast.error(wrongLoginHelp, {autoClose: false, closeOnClick: false});
      return;
    }
    
    let msgStruct = {
      timestamp: Math.floor(Date.now()/1000),
      auth: {
        action: "login",
        user: user,
        password: password,
        message: ""
      },
      message_id: '',
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
      message_id: '',
      author: '',
      recipient: '',
    }
    invoke("register", { auth: msgStruct });
  }

  async function tauri_toast(options) {
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
    const unlisten = listen("register_token", (e) => {
      toast.success("Successfully registered/logged in! 😎");
      // tauri_toast({ title: 'Successfully registered!', body: e.payload.message_content });
      setToken("the cake was a lie!");
    });

    return () => {
      unlisten.then(f => f());
    }


  }, []);
  useEffect(() => {
    const unlisten = listen("auth_failure", (e) => {
      toast.error("Authentication failure! 🥲 :" + e.payload.auth.message);
    });

    return () => {
      unlisten.then(f => f());
    }


  }, []);


  

  async function getLoginOptions(connection){
    const hash = SHA256(connection.host).toString();
    const credentials = await load(`${hash}/credentials.bin`, { autoSave: 0 });

    let options = [];

    let logins = await credentials.keys();

    logins.forEach(login => {
      options.push({name: login});
    });

    return options;

  }




  const darkTheme = createTheme({
    palette: {
      mode: 'dark',
    },
  });

  let [options, setOptions] = useState([]);

  getLoginOptions(connection).then((x) => {
    setOptions(x);
  })

  return (
    <ThemeProvider theme={darkTheme}>
      <CssBaseline />

      <Connection connection={connection} setConnection={setConnection} />
      <div className="container">
        <h1>Welcome to CipherChat!</h1>


        <p>CipherChat helps you keep your convos private 😎.</p>


        <Container>

          

            <Autocomplete
            // style={{width: "fit-content"}}
            options={options}
            getOptionLabel={(option) => option.name}
            style={{width: '100%'}}
            renderInput={(params) => (
              <TextField
              {...params}
                id="login-username"
                onChange={(e) => setUser(e.currentTarget.value)}
                placeholder="Enter a Username..."
                label="Username">
            </TextField>
            )}
            freeSolo
            onChange={(e, n) => setUser(n.name)}
          />

          
          <TextField
            id="login-password"
            onChange={(e) => setPassword(e.currentTarget.value)}
            style={{width: '100%'}}
            placeholder="Enter your Password..."
            label="Password"
            type="password"
          />




          <Stack style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', width: '100%', paddingTop: '10px'}} spacing={2} direction="row">
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