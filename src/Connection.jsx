import * as React from 'react';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import Dialog from '@mui/material/Dialog';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import DialogContentText from '@mui/material/DialogContentText';
import DialogTitle from '@mui/material/DialogTitle';
import { exit, relaunch } from '@tauri-apps/plugin-process';
import { invoke } from "@tauri-apps/api/core";
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';

import {useEffect, useState} from "react";
import { emit, listen } from '@tauri-apps/api/event';

import { ToastContainer, toast, Bounce } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';

export default function Connection({connection, setConnection}) {
  const [url, setUrl] = useState("");

  const port = 9999;

  async function connectToUrl(){
    try{
        
        let status = await invoke("connect_via_url", { url: `wss://${url}:${port}` });
        setConnection(status);
        // console.log(status);
        // toast({ title: 'Connection successful', body: `Connection to wss://${url}:${port} has been successful!` });
        toast.info("Connection successful! 😄");
      }catch(err){
        // toast({ title: 'Connection Error', body: `No connection to wss://${url}:${port}!` });
        toast.error("Connection error! 🥲 :" + err);
        // console.error(err);
    }

    //"ws://127.0.0.1:9999"
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
  async function quitApp(){
    await exit(0);
  }


  useEffect(() => {
    const unlisten = listen("connection_closed", (e) => {
      toast.error("Connection suddenly closed 😮!");
      setConnection({});
    });

    return () => {
      unlisten.then(f => f());
    }


  }, []);


  return (
    <div>
      <Dialog open={!Object.keys(connection).length}>
        <DialogTitle>Connect to Homeserver</DialogTitle>
        <DialogContent>
          <DialogContentText>
            We need to connect you to a Homeserver in order to route your messages correctly 😄
          </DialogContentText>
          <TextField
            autoFocus
            margin="dense"
            id="name"
            label="Websocket URL"
            fullWidth
            variant="standard"
            onChange={(e) => {setUrl(e.currentTarget.value)}}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={quitApp}>Quit</Button>
          <Button onClick={connectToUrl}>Connect</Button>
        </DialogActions>
      </Dialog>
    </div>
  );
}