import * as React from 'react';

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';

import Button from '@mui/material/Button';
import Container from '@mui/material/Container';
import Stack from '@mui/material/Stack';
import Drawer from '@mui/material/Drawer';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemText from '@mui/material/ListItemText';
import Box from '@mui/material/Box';
import Paper from '@mui/material/Paper';
import { BottomNavigation, BottomNavigationAction } from '@mui/material';
import ListItemIcon from '@mui/material/ListItemIcon';

import SendIcon from '@mui/icons-material/Send';
import CloseIcon from '@mui/icons-material/Close';

import PersonIcon from '@mui/icons-material/Person';
import AddIcon from '@mui/icons-material/Add';

import TextField from '@mui/material/TextField';

import { ThemeProvider, createTheme } from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';

import { emit, listen } from '@tauri-apps/api/event';

import ChatComponent from "./ChatComponent";
import { Person } from '@mui/icons-material';


function Chat({token, setToken, user, connection, setConnection}) {
  const [recipient, setRecipient] = useState("");
  const [message, setMessage] = useState("");

  const [chat, setChat] = useState({});
  const [contact, setContact] = useState("");


  async function sendMessage(){
    let msgStruct = {
      content: {
        ciphertext: '',
        nonce: '',
        cleartext: message
      },
      timestamp: Math.floor(Date.now()/1000),
      auth: null,
      message_id: '',
      author: user,
      recipient: recipient
    }

    invoke("send_msg", { msg: msgStruct });
    msgStruct.author = "You";

    setChat(prevChat => {
      const newChat = { ...prevChat };
  
      if (msgStruct.recipient in newChat) {
        newChat[msgStruct.recipient].push(msgStruct);
      } else {
        newChat[msgStruct.recipient] = [msgStruct];
      }
  
      return newChat;
    });

    setContact(msgStruct.recipient);
    // setChat(chat => [...chat, msgStruct]);
  }

  async function closeChat(){
    setToken("");
    
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
      if(e.payload.content !== null && e.payload.content !== undefined){
        toast({ title: 'Message received!', body: e.payload.content.cleartext });
        let msgStruct = e.payload;
        setChat(prevChat => {
          const newChat = { ...prevChat };
      
          if (msgStruct.author in newChat) {
            newChat[msgStruct.author].push(msgStruct);
          } else {
            newChat[msgStruct.author] = [msgStruct];
          }
      
          return newChat;
        });
      }
      
    });


    return () => {
      unlisten.then(f => f());
    }


  }, []);

  useEffect(() => {
    const unlisten = listen("connection_closed", (e) => {
      setToken("");
      setConnection({});
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
      <Box sx={{ display: 'flex' }}>
      <CssBaseline />

      <Drawer
        sx={{
          width: 240,
          flexShrink: 0,
          '& .MuiDrawer-paper': {
            width: 240,
            boxSizing: 'border-box',
          },
        }}
        variant="permanent"
        anchor="left"
      >
        <List>
            <ListItem key="New">
              <ListItemButton>
                <ListItemIcon>
                  <AddIcon />
                </ListItemIcon>
                <ListItemText primary="New" />
              </ListItemButton>
            </ListItem>
          {Object.keys(chat).map((contact, index) => (
            <ListItem key={contact}>
              <ListItemButton onClick={() => setContact(contact)}>
                <ListItemIcon>
                  <PersonIcon />
                </ListItemIcon>
                <ListItemText primary={contact} />
              </ListItemButton>
            </ListItem>
          ))}
        </List>
      </Drawer>






      <Box
        component="main"
        sx={{ flexGrow: 1, bgcolor: 'background.default', p: 3 }}
      >
        <ChatComponent chat={chat} contact={contact}/>

          

        {/* <TextField
            id="recipient"
            onChange={(e) => {setRecipient(e.currentTarget.value);}}
            placeholder="Enter a Username..."
            label="Recipient"
          /> */}


{/* 
          <Stack style={{ margin: 'auto', width: '30%', padding: '10px' }} spacing={2} direction="row">
            <Button variant="outlined" onClick={() => sendMessage()}>Send</Button>
            <Button variant="outlined" onClick={() => closeChat()}>Close</Button>
          </Stack> */}

            <BottomNavigation
              showLabels
            >
              <BottomNavigationAction onClick={() => sendMessage()} label="Send" icon={<SendIcon />}  />
              <BottomNavigationAction onClick={() => closeChat()} label="Close" icon={<CloseIcon />} />
            </BottomNavigation>
      </Box>

      </Box>
        
    </ThemeProvider>
  );
}

export default Chat;