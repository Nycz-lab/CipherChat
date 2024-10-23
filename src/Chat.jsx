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

import { load } from '@tauri-apps/plugin-store';
import SHA256 from 'crypto-js/sha256';

import MuiAppBar from '@mui/material/AppBar';
import { styled, useTheme } from '@mui/material/styles';
import Toolbar from '@mui/material/Toolbar';
import IconButton from '@mui/material/IconButton';
import MenuIcon from '@mui/icons-material/Menu';

import Typography from '@mui/material/Typography';
import ChevronLeftIcon from '@mui/icons-material/ChevronLeft';
import ChevronRightIcon from '@mui/icons-material/ChevronRight';



const drawerWidth = 240;



function Chat({token, setToken, user, connection, setConnection}) {
  const [recipient, setRecipient] = useState("");
  const [message, setMessage] = useState("");

  const [chat, setChat] = useState({});
  const [contact, setContact] = useState("");

  const [contactDialogOpen, setContactDialogOpen] = useState(false);
  const [contactDialogUsername, setContactDialogUsername] = useState("");

  const [messagesLoaded, setMessagesLoaded] = useState(false);

  const theme = useTheme();
  const [open, setOpen] = React.useState(false);

  const handleDrawerOpen = () => {
    setOpen(true);
  };

  const handleDrawerClose = () => {
    setOpen(false);
  };

  

  async function loadMessagesStore(){

    const hash = SHA256(connection.host).toString();
    const messageStore = await load(`${hash}/${user}/messages.bin`, { autoSave: 0 });

    let messages = await messageStore.get("messages");
    if(messages !== null && messages !== undefined){

      setChat(messages);
    }

    setMessagesLoaded(true);

  }

  useEffect(() => {
    console.log("load message store")
    loadMessagesStore(); // This will always use latest value of count
}, []);
  


  useEffect(() => {
    storeMessages(chat); // This will always use latest value of count
}, [chat]);
  

  async function storeMessages(messages){
    if (!messagesLoaded){
      return;
    }
    const hash = SHA256(connection.host).toString();
    const messageStore = await load(`${hash}/${user}/messages.bin`, { autoSave: 0 });

    if(Object.keys(messages).length === 0){
      console.log("chat empty");
    }
    await messageStore.set("messages", messages);
    await messageStore.save();

    console.log("saved messages");
    
  }


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
  }

  async function closeChat(){
    let msgStruct = {
      timestamp: Math.floor(Date.now()/1000),
      auth: {
        action: "logout",
        user: '',
        password: '',
        message: ""
      },
      message_id: '',
      author: user,
      recipient: 'System'
    }

    invoke("logout", { auth: msgStruct });
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

    const unlisten = listen("msg", async (e) => {
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
      toast.error("Connection suddenly closed 😮!");
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

  const Main = styled('main', { shouldForwardProp: (prop) => prop !== 'open' })(
    ({ theme }) => ({
      flexGrow: 1,
      padding: theme.spacing(3),
      transition: theme.transitions.create('margin', {
        easing: theme.transitions.easing.sharp,
        duration: theme.transitions.duration.leavingScreen,
      }),
      marginLeft: `-${drawerWidth}px`,
      variants: [
        {
          props: ({ open }) => open,
          style: {
            transition: theme.transitions.create('margin', {
              easing: theme.transitions.easing.easeOut,
              duration: theme.transitions.duration.enteringScreen,
            }),
            marginLeft: 0,
          },
        },
      ],
    }),
  );
  
  const AppBar = styled(MuiAppBar, {
    shouldForwardProp: (prop) => prop !== 'open',
  })(({ theme }) => ({
    transition: theme.transitions.create(['margin', 'width'], {
      easing: theme.transitions.easing.sharp,
      duration: theme.transitions.duration.leavingScreen,
    }),
    variants: [
      {
        props: ({ open }) => open,
        style: {
          width: `calc(100% - ${drawerWidth}px)`,
          marginLeft: `${drawerWidth}px`,
          transition: theme.transitions.create(['margin', 'width'], {
            easing: theme.transitions.easing.easeOut,
            duration: theme.transitions.duration.enteringScreen,
          }),
        },
      },
    ],
  }));
  
  const DrawerHeader = styled('div')(({ theme }) => ({
    display: 'flex',
    alignItems: 'center',
    padding: theme.spacing(0, 1),
    // necessary for content to be below app bar
    ...theme.mixins.toolbar,
    justifyContent: 'flex-end',
  }));



  return (
    <ThemeProvider theme={darkTheme}>
      <Box sx={{ display: 'flex' }}>
      <CssBaseline />

      <AppBar position="fixed" open={open}>
        <Toolbar>
          <IconButton
            color="inherit"
            aria-label="open drawer"
            onClick={handleDrawerOpen}
            edge="start"
            sx={[
              {
                mr: 2,
              },
              open && { display: 'none' },
            ]}
          >
            <MenuIcon />
          </IconButton>
          <Typography variant="h6" noWrap component="div">
            {contact}
          </Typography>
        </Toolbar>
      </AppBar>

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
        variant="persistent"
        anchor="left"
        open={open}
        disableEnforceFocus
      >
        <DrawerHeader>
          <IconButton onClick={handleDrawerClose}>
            {theme.direction === 'ltr' ? <ChevronLeftIcon /> : <ChevronRightIcon />}
          </IconButton>
        </DrawerHeader>
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

      <Main open={open}>



      <Box
        component="main"
        sx={{ flexGrow: 1, bgcolor: 'background.default', p: 3 }}
      >
        <ChatComponent chat={chat} contact={contact} message={message} setMessage={setMessage}/>

            <BottomNavigation
              showLabels
            >
              <BottomNavigationAction onClick={() => sendMessage()} label="Send" icon={<SendIcon />}  />
              <BottomNavigationAction onClick={() => closeChat()} label="Close" icon={<CloseIcon />} />
            </BottomNavigation>
      </Box>

      </Main>

      </Box>
        
    </ThemeProvider>
  );
}

export default Chat;