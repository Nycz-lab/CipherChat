use log::info;
use socket::{Socket, SocketFuncs};
use tauri::WebviewWindow;
use tauri::{Manager, Window};
use tauri_plugin_store::StoreBuilder;
use tauri_plugin_store::StoreExt;
use util::{ConnectionInfo, MsgPayload};

use tokio::sync::Mutex;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod crypt;
mod socket;
pub mod util;
mod x3dh;
mod xxxdh;

pub use util::Error;
use x3dh::get_keybundle;

lazy_static::lazy_static! {
  static ref SOCKET: Mutex<Option<Box<Socket>>> = Mutex::new(None);
  static ref MAIN_WINDOW: Mutex<Option<WebviewWindow>> = Mutex::new(None);
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

#[tauri::command]
async fn encrypt(key: &str, txt: &str) -> Result<String, util::Error> {
    let encrypted = crypt::encrypt(key.to_string(), txt.to_string())?;
    Ok(encrypted)
}

#[tauri::command]
async fn decrypt(key: &str, txt: &str) -> Result<String, util::Error> {
    let decrypted = crypt::decrypt(key.to_string(), txt.to_string())?;
    Ok(decrypted)
}

#[tauri::command]
async fn send_msg(msg: MsgPayload) -> Result<(), util::Error> {
    let mut socket_lock = SOCKET.lock().await;
    if let Some(socket) = socket_lock.as_mut() {
        socket.send_msg(msg).await?;
    } else {
        // Handle the case when the Option is None
        error!("Socket not initialized.");
    }
    Ok(())
}

#[tauri::command]
async fn login(auth: MsgPayload) -> Result<(), util::Error> {
    let mut socket_lock = SOCKET.lock().await;
    if let Some(socket) = socket_lock.as_mut() {
        socket.login(auth).await?;
    } else {
        // Handle the case when the Option is None
        error!("Socket not initialized.");
    }
    Ok(())
}

#[tauri::command]
async fn register(auth: MsgPayload, app_handle: tauri::AppHandle) -> Result<(), util::Error> {
    let mut socket_lock = SOCKET.lock().await;
    if let Some(socket) = socket_lock.as_mut() {
        let bundle = get_keybundle(app_handle, auth.clone());
        socket.register(auth.clone(), bundle).await?;
    } else {
        // Handle the case when the Option is None
        info!("Socket not initialized.");
    }
    Ok(())
}

#[tauri::command]
async fn send_enc_msg(key: &str, mut msg: MsgPayload) -> Result<(), util::Error> {
    msg.content = encrypt(key, &msg.content).await?;
    send_msg(msg).await?;
    Ok(())
}

#[tauri::command]
async fn connect_via_url(url: String) -> Result<util::ConnectionInfo, util::Error> {
    init_conn(url.to_string()).await?;
    let mut stream_type = "not defined";
    let socket_lock = SOCKET.lock().await;
    if let Some(socket) = socket_lock.as_ref() {
        stream_type = socket.stream_type.as_str();
    }
    Ok(ConnectionInfo {
        host: url.to_string(),
        stream_type: stream_type.to_string(),
    })
}

async fn init_conn(url: String) -> Result<(), Error> {
    info!("initiating Connection");
    let mut win1 = MAIN_WINDOW.lock().await;
    let win = win1.as_mut();
    let res = match win{
        Some(v) => v,
        None => {
            info!("error window not available yet");
            return Ok(());
        },
    };
    let mut socket = Socket::new(res.clone(), url).await?;
    socket.recv_msg().await;
    *SOCKET.lock().await = Some(socket);
    Ok(())
}

#[tauri::command]
async fn close_conn() -> Result<(), Error> {
    info!("closing...");
    let mut socket_lock = SOCKET.lock().await;
    if let Some(socket) = socket_lock.as_mut() {
        socket.close().await?;
        let socket_value = socket_lock.take().unwrap();
        drop(socket_value);
        *socket_lock = None;
    }

    Ok(())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

    pretty_env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app|{
            let win = app.get_webview_window("main").unwrap();
            
            // let mut store = StoreBuilder::new(app.handle(), "credentials.bin".parse()?).build();
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async {
                let mut main_window = MAIN_WINDOW.lock().await; // Locking the mutex
                *main_window = Some(win); // Assigning the new window
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            encrypt,
            decrypt,
            send_msg,
            send_enc_msg,
            connect_via_url,
            close_conn,
            login,
            register
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
