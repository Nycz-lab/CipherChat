import * as React from 'react';

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';


import { ThemeProvider, createTheme } from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';

import { emit, listen } from '@tauri-apps/api/event';

import Auth from "./Auth";
import Chat from "./Chat";


function App() {
  const [token, setToken] = useState("");
  const [user, setUser] = useState("");



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

    // const unlisten = listen("msg", (e) => {
    //   console.log(e);
    //   toast({ title: 'Message received!', body: e.payload.message_content });
    // });


    // return () => {
    //   unlisten.then(f => f());
    // }


  }, []);



  const darkTheme = createTheme({
    palette: {
      mode: 'dark',
    },
  });



  return (
    token ? <Chat token={token} setToken={setToken} user={user}/>  : <Auth token={token} setToken={setToken} user={user} setUser={setUser}/>
  );
}

export default App;