import React from 'react';

import Dialog from '@mui/material/Dialog';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';
import DialogContentText from '@mui/material/DialogContentText';
import DialogActions from '@mui/material/DialogActions';

import Button from '@mui/material/Button';

import { TextField } from '@mui/material';

import { useRef } from 'react';

const ContactDialog = ({ setContactDialogOpen, contactDialogOpen, setContact, setChat }) => {

  const contactDialogUsername = useRef();

  return (
   
    <Dialog
        open={contactDialogOpen}
      >
        <DialogTitle>Choose Contact</DialogTitle>
        <DialogContent>
          <DialogContentText>
            Input the Username of the person you want to chat with 😎
          </DialogContentText>
          <TextField
            required
            margin="dense"
            id="name"
            name="name"
            label="Username"
            fullWidth
            variant="standard"
            inputRef={contactDialogUsername}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => {
            let val = contactDialogUsername.current.value;
            setChat(prevChat => {
              const newChat = { ...prevChat };
          
              if (!(val in newChat)) {
                newChat[val] = [];
              }
          
              return newChat;
            });
            setContact(val);
            setContactDialogOpen(false);
          }}>Ok</Button>
          <Button onClick={() => {setContactDialogOpen(false)}}>Close</Button>
        </DialogActions>
      </Dialog>

  );
};

export default ContactDialog;