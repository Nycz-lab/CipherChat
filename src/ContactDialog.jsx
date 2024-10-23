import React from 'react';

import Dialog from '@mui/material/Dialog';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';
import DialogContentText from '@mui/material/DialogContentText';
import DialogActions from '@mui/material/DialogActions';

import Button from '@mui/material/Button';

import { TextField } from '@mui/material';

const ContactDialog = ({ setContactDialogOpen, contactDialogOpen, contactDialogUsername, setContactDialogUsername, setContact, setChat }) => {

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
          <Button onClick={() => {setContactDialogOpen(false)}}>Close</Button>
        </DialogActions>
      </Dialog>

  );
};

export default ContactDialog;