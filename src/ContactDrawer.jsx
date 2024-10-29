import React from 'react';

import Toolbar from '@mui/material/Toolbar';
import IconButton from '@mui/material/IconButton';
import MenuIcon from '@mui/icons-material/Menu';

import Typography from '@mui/material/Typography';
import ChevronLeftIcon from '@mui/icons-material/ChevronLeft';
import ChevronRightIcon from '@mui/icons-material/ChevronRight';

import Container from '@mui/material/Container';
import Stack from '@mui/material/Stack';
import Drawer from '@mui/material/Drawer';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemText from '@mui/material/ListItemText';

import { styled, useTheme } from '@mui/material/styles';
import MuiAppBar from '@mui/material/AppBar';

import PersonIcon from '@mui/icons-material/Person';
import AddIcon from '@mui/icons-material/Add';

import ListItemIcon from '@mui/material/ListItemIcon';
import Paper from '@mui/material/Paper';


const ContactDrawer = ({ open, setOpen, contact, setContact, setContactDialogOpen, chat }) => {
  
  const theme = useTheme();
    const drawerWidth = 240;

  const handleDrawerOpen = () => {
    setOpen(true);
  };

  const handleDrawerClose = () => {
    setOpen(false);
  };

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
    

    <><AppBar position="fixed" open={open}>
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
      </AppBar><Drawer
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
          </Drawer></>
    
  );
};

export default ContactDrawer;