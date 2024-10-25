import * as React from 'react';

import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import Box from '@mui/material/Box';

import { BottomNavigation, BottomNavigationAction } from '@mui/material';



import SendIcon from '@mui/icons-material/Send';
import CloseIcon from '@mui/icons-material/Close';
import AttachFileIcon from '@mui/icons-material/AttachFile';



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


import { styled, useTheme } from '@mui/material/styles';

import ContactDrawer from "./ContactDrawer";
import ContactDialog from "./ContactDialog";

import { open } from '@tauri-apps/plugin-dialog';
import { writeFile, readFile, BaseDirectory } from '@tauri-apps/plugin-fs';

import {getMimeTypeFromExtension} from "./util";

import { appDataDir } from '@tauri-apps/api/path';


const drawerWidth = 240;



function Chat({token, setToken, user, connection, setConnection}) {
  const [recipient, setRecipient] = useState("");
  // const [message, setMessage] = useState("");
  const messageRef = useRef();

  const [chat, setChat] = useState({});
  const [contact, setContact] = useState("");

  const [contactDialogOpen, setContactDialogOpen] = useState(false);
  const [contactDialogUsername, setContactDialogUsername] = useState("");

  const [messagesLoaded, setMessagesLoaded] = useState(false);

  const theme = useTheme();
  const [drawerOpen, setDrawerOpen] = useState(false);

  

  

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

  function Uint8ToString(u8a){
    var CHUNK_SZ = 0x8000;
    var c = [];
    for (var i=0; i < u8a.length; i+=CHUNK_SZ) {
      c.push(String.fromCharCode.apply(null, u8a.subarray(i, i+CHUNK_SZ)));
    }
    return c.join("");
  }

  async function attachFile(){
    const file = await open({
      multiple: false,
      directory: false,
    });

    var re = /(?:\.([^.]+))?$/;
    let filetype = re.exec(file)[1];

    let binary_data = await readFile(file);
    let b64 = btoa(Uint8ToString(binary_data));

    let mime_type = getMimeTypeFromExtension(filetype);

    let payload = {data: b64, mime_type: mime_type};
    let json = JSON.stringify(payload);

    let msgStruct = await sendPayload(json);

    let path = await appDataDir();
    const hash = SHA256(connection.host).toString();


    payload.data = `${path}/${hash}/${user}/${msgStruct.message_id}`;

    await writeFile(`${path}/${hash}/${user}/${msgStruct.message_id}`, binary_data);

    msgStruct.content.cleartext = payload;


    setChat(prevChat => {
      const newChat = { ...prevChat };
  
      if (msgStruct.recipient in newChat) {
        newChat[msgStruct.recipient].push(msgStruct);
      } else {
        newChat[msgStruct.recipient] = [msgStruct];
      }
  
      return newChat;
    });
  }

  async function sendPayload(data){
    let msgStruct = {
      content: {
        ciphertext: '',
        nonce: '',
        cleartext: data
      },
      timestamp: Math.floor(Date.now()/1000),
      auth: null,
      message_id: crypto.randomUUID(),
      author: user,
      recipient: contact
    }

    invoke("send_msg", { msg: msgStruct });
    msgStruct.author = "You";

    

    setContact(msgStruct.recipient);

    return msgStruct;
  }

  async function sendMessage(){

    if(contact === ""){
      toast.error("Recipient is empty...");
      return;
    }

    if(messageRef.current.value === ""){
      toast.error("Message cant be empty...");
      return;
    }

    let payload = {data: messageRef.current.value, mime_type: "text/plain"};
    let json = JSON.stringify(payload);

    let msgStruct = await sendPayload(json);
    msgStruct.content.cleartext = payload;

    setChat(prevChat => {
      const newChat = { ...prevChat };
  
      if (msgStruct.recipient in newChat) {
        newChat[msgStruct.recipient].push(msgStruct);
      } else {
        newChat[msgStruct.recipient] = [msgStruct];
      }
  
      return newChat;
    });

    messageRef.current.value = "";
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
        let json_data = e.payload.content.cleartext;

        // tauri_toast({ title: 'Message received!', body: e.payload.content.cleartext });
        // console.log(e);
        let msgStruct = e.payload;
        // msgStruct.content.cleartext = json_data;
        
        let payload = JSON.parse(json_data);

        if(payload.mime_type !== "text/plain"){

          toast.info("Message received 📷 ");
        }else{
          toast.info("Message received: 🖊️ " + payload.data);
        }

        if(payload.mime_type !== "text/plain"){
          let path = await appDataDir();
          const hash = SHA256(connection.host).toString();

          let u8_2 = new Uint8Array(atob(payload.data).split("").map(function(c) {
            return c.charCodeAt(0); }));
          let binary = Uint8Array.from(u8_2);

          payload.data = `${path}/${hash}/${user}/${msgStruct.message_id}`;

          await writeFile(`${path}/${hash}/${user}/${msgStruct.message_id}`, binary);

        }

        msgStruct.content.cleartext = payload;

        // TODO somehow this usestate gets called twice causing the receiver to get duplicate messages
        setChat(prevChat => {
          const newChat = { ...prevChat };
      
          if (msgStruct.author in newChat) {
            // TODO fix this temporary fix properly:
            if(!newChat[msgStruct.author].includes(msgStruct))
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

  useEffect(() => {
    const handleKeyPress = (event) => {
      if(contactDialogOpen){
        return;
      }
      if(event.key !== "Enter"){
        document.getElementById("chatTextbox").focus();
        return;
      }
      sendMessage();
    };

    // Register the keypress event
    window.addEventListener('keypress', handleKeyPress);

    // Clean up by removing the event listener on unmount
    return () => {
      window.removeEventListener('keypress', handleKeyPress);
    };
  }, [contact, contactDialogOpen]); // Empty dependency array to run only once



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
          props: ({ open }) => drawerOpen,
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
  




  return (
    <ThemeProvider theme={darkTheme}>
      <Box sx={{ display: 'flex' }}>
      <CssBaseline />

      

      <ContactDialog contactDialogOpen={contactDialogOpen} setContactDialogOpen={setContactDialogOpen} setChat={setChat} setContact={setContact}/>


      <ContactDrawer contact={contact} open={drawerOpen} setContact={setContact} setContactDialogOpen={setContactDialogOpen} setOpen={setDrawerOpen} chat={chat} />

      <Main open={drawerOpen}>


      <Box
        component="main"
        sx={{ flexGrow: 1, bgcolor: 'background.default', p: 3 }}
      >
        
        <ChatComponent chat={chat} contact={contact} message={messageRef}/>
        
            <BottomNavigation
              showLabels
            >
              <BottomNavigationAction onClick={() => sendMessage()} label="Send" icon={<SendIcon />}  />
              <BottomNavigationAction onClick={() => closeChat()} label="Close" icon={<CloseIcon />} />
              <BottomNavigationAction onClick={() => attachFile()} label="Attach" icon={<AttachFileIcon />} />
            </BottomNavigation>
      </Box>
      </Main>


      


      </Box>
        
    </ThemeProvider>
  );
}

export default Chat;