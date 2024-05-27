
mod shared_folder_connection;
mod file_handler;
mod installer_types;
mod connection_error;
mod deposit;

use std::net::TcpStream;
use std::process::Command;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use futures::executor::block_on;
use lazy_static::lazy_static;
use slint::{CloseRequestResponse, ComponentHandle, VecModel};
use crate::{AppInfo, AppWindow};
use crate::deposit::Deposit;
use crate::file_handler::FileHandler;
use crate::shared_folder_connection::SHFConnection;



slint::include_modules!();
lazy_static! {
    static ref THREAD_POOL: rayon::ThreadPool = {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_cpus::get())
            .build()
            .unwrap()
    };
}


const ZIP: &'static [u8] = include_bytes!(r".\Setups\Setups.tar.gz");

async fn create_local_admin(name: String, pass: String) -> String {
    if name.is_empty() || pass.is_empty() {
        return "Campos vacíos".to_string();
    }

    println!("{} : {}", &name, &pass);
    let add_user_status = Command::new("net").args(&["user", "/add", &name, &pass]).status();
    let add_to_admin_group_status = Command::new("net").args(&["localgroup", "Administradores", &name, "/add"]).status();

    match (add_user_status, add_to_admin_group_status) {
        (Ok(add_status), Ok(add_group_status)) if add_status.success() && add_group_status.success() => {
            println!("Usuario creado y añadido al grupo de administradores correctamente.");
            "Creado".to_string()
        }
        _ => {
            println!("Error al ejecutar uno de los comandos.");
            "Fallo".to_string()
        }
    }
}
async fn search_local() -> Vec<AppInfo>  {
    FileHandler::search_local().await
}
async fn search_online() -> Vec<AppInfo> {
    FileHandler::search_online().await
}
fn is_internet_available() -> bool {
    TcpStream::connect("www.google.com:80").is_ok()
}
fn get_programs(shared_online_deposit: Arc<Mutex<Deposit>>, shared_local_deposit: Arc<Mutex<Deposit>>) -> Rc<VecModel<AppInfo>> {
    if crate::is_internet_available() {
        let deposit_guard = shared_online_deposit.lock().unwrap();
        let deposit_content_clone = deposit_guard.get_all();
        Rc::new(VecModel::from(deposit_content_clone))
    } else {
        let deposit_guard = shared_local_deposit.lock().unwrap();
        let deposit_content_clone = deposit_guard.get_all();
        Rc::new(VecModel::from(deposit_content_clone))
    }
}

fn create_admin(){}


#[tokio::main]
async fn main() {

    // Connection to folder
    SHFConnection::connect(r#"\\192.168.1.69\Software\Instaladores"#);

    // Create deposits to share app info vectors
    let shared_online_deposit = Arc::new(Mutex::new(Deposit::from(block_on(FileHandler::search_online()))));
    let shared_local_deposit = Arc::new(Mutex::new(Deposit::from(FileHandler::search_local())));

    // Add files local and files online
    {
        // Lock the local deposit and replace its content with the online content
        let mut online_deposit = shared_online_deposit.lock().unwrap();
        online_deposit.replace(block_on(FileHandler::search_online()));
    }

    // Create window
    let app = AppWindow::new().unwrap();

    // Get programs from shared deposits
    app.set_programs(get_programs(shared_online_deposit.clone(), shared_local_deposit.clone()).into());

    // On click research button
    app.on_research(move || {
        let shared_online_deposit = shared_online_deposit.clone();
        block_on(async move {
            let result = search_online().await;
            let mut online_deposit = shared_online_deposit.lock().await;
            online_deposit.replace(result.clone());
            Rc::new(VecModel::from(result)).into()
        }).into()
    });

    // Search local included files
    app.on_search_local(|| {
        block_on(async { Rc::new(VecModel::from(search_local().await)).into() }).into()
    });

    app.on_create_admin(|name, pass| block_on(create_local_admin(name.to_string(), pass.to_string())).into());
    app.on_check_internet(|| crate::is_internet_available());
    app.set_have_internet(crate::is_internet_available());

    app.on_install(move |to_install, unattended, online| {
        print!("TESTING");
    });

    app.window().on_close_requested(|| CloseRequestResponse::default());

    app.run().unwrap();

}