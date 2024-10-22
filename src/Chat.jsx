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
import Dialog from '@mui/material/Dialog';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';
import DialogContentText from '@mui/material/DialogContentText';
import DialogActions from '@mui/material/DialogActions';

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

import { ToastContainer, toast, Bounce } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';



function Chat({token, setToken, user, connection, setConnection}) {
  const [recipient, setRecipient] = useState("");
  const [message, setMessage] = useState("");

  const [chat, setChat] = useState({});
  const [contact, setContact] = useState("");

  const [contactDialogOpen, setContactDialogOpen] = useState(false);
  const [contactDialogUsername, setContactDialogUsername] = useState("");


  async function sendMessage(){

    if(contact === ""){
      toast.error("Recipient is empty...");
      return;
    }

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
      recipient: contact
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

  useEffect(() => {

    const unlisten = listen("msg", (e) => {
      if(e.payload.content !== null && e.payload.content !== undefined){
        tauri_toast({ title: 'Message received!', body: e.payload.content.cleartext });
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

      <Dialog
        open={contactDialogOpen}
      >
        <DialogTitle>Choose Contact</DialogTitle>
        <DialogContent>
          <DialogContentText>
            Input the Username of the person you want to chat with 😎
          </DialogContentText>
          <TextField
            autoFocus
            required
            margin="dense"
            id="name"
            name="name"
            label="Username"
            fullWidth
            variant="standard"
            onChange={(e) => {setContactDialogUsername(e.currentTarget.value)}}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => {
            setChat(prevChat => {
              const newChat = { ...prevChat };
          
              if (!(contactDialogUsername in newChat)) {
                newChat[contactDialogUsername] = [];
              }
          
              return newChat;
            });
            setContact(contactDialogUsername);
            setContactDialogOpen(false);
          }}>Ok</Button>
          <Button onClick={() => setContactDialogOpen(false)}>Close</Button>
        </DialogActions>
      </Dialog>

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
              <ListItemButton onClick={() => setContactDialogOpen(true)}>
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
        <ChatComponent chat={chat} contact={contact} setMessage={setMessage}/>

              <ToastContainer
              position="top-right"
              autoClose={5000}
              hideProgressBar={false}
              newestOnTop={false}
              closeOnClick
              rtl={false}
              pauseOnFocusLoss
              draggable
              pauseOnHover
              theme="dark"
              transition={Bounce}
              />

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