use std::{fs, io, thread};
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::rc::Rc;
use std::sync::mpsc;
use flate2::read::GzDecoder;

use futures::task::SpawnExt;
use lazy_static::lazy_static;
use num_cpus;
use rayon::prelude::*;
use slint::SharedString;


use crate::{AppInfo, ZIP};
use crate::installer_types::InstallerType;
use crate::shared_folder_connection::SHFConnection;

macro_rules! path_buf_to_string {
    ($p:expr) => {
        $p.to_string_lossy().into_owned()
    };
}
macro_rules! string_os_to_string {
    ($p:expr) => {
        $p.unwrap().to_string_lossy().into_owned()
    };
}

lazy_static! {
    static ref APPDATA_PATH: PathBuf = FileHandler::get_appdata();
}

pub struct FileHandler {
}
impl FileHandler {

    pub fn get_appdata() -> PathBuf {
        dirs::data_dir().expect("Error: On get appdata folder")
    }

    // Unzip compressed files
    pub fn unzip_locals(x: &[u8]) -> io::Result<()> {
        let mut gzip = GzDecoder::new(x);
        let mut gz_buffer = Vec::new();
        gzip.read_to_end(&mut gz_buffer)?;

        let appdata = Self::get_appdata();
        let mut tar_file_path = appdata.clone();
        tar_file_path.push("programs.tar");
        let mut tar_file = fs::File::create(&tar_file_path)?;

        tar_file.write_all(&gz_buffer)?;

        drop(tar_file);

        let mut tar_data = tar::Archive::new(fs::File::open(&tar_file_path)?);

        for file in tar_data.entries()? {
            let mut file = file?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            let path = appdata.join(file.path().unwrap().file_name().unwrap().to_string_lossy().to_string());
            println!("{:?}", path.to_string_lossy().to_string());
            let mut file = fs::File::create(&path)?;
            file.write_all(&buffer)?;
        }
        Ok(())
    }

    // Search files and convert in instances of AppInfo
    pub async fn search_online() -> Vec<AppInfo> {
        match SHFConnection::list_files(r"\\192.168.1.69\Software\Instaladores") {
            Ok(files) =>
                {
                    files
                },
            Err(e) =>
                {
                    println!("ERROR");
                    FileHandler::search_local()
                }
        }
    }

    pub async fn search_local() -> Vec<AppInfo>  {
        let mut apps = vec![
            AppInfo {name: SharedString::from("Chrome Setup.msi"), install: false, path: SharedString::from("local"), ins_type: SharedString::from("msi")},
            AppInfo {name: SharedString::from("LibreOffice 7.4.1 x64.msi"), install: false, path: SharedString::from("local"), ins_type: SharedString::from("exe") },
            AppInfo {name: SharedString::from("Adobe Reader x64.exe"), install: false, path: SharedString::from("local"), ins_type: SharedString::from("msi") },
            AppInfo {name: SharedString::from("7z2404-x64.msi"), install: false, path: SharedString::from("local"), ins_type: SharedString::from("msi")}
        ];

        let mut appdata = PathBuf::from(
            &APPDATA_PATH.to_str().expect("Error: Appdata to string")
        );

        for mut a in &mut apps {
            let mut path = appdata.clone();
            path.push(&a.name.to_string());
            a.path = SharedString::from(
                path.to_str().expect("Error: Converting path to string").to_string()
            );

        }
        apps
    }

    pub async fn write(info: &[u8], name: &str) {
        let mut appdata = PathBuf::from(&APPDATA_PATH.to_str().expect("Error: Appdata to string"));
        appdata.push(name);
        let mut buf_writer = BufWriter::new(
            fs::File::create(appdata)
                .expect("Failed to create output file")
        );

        buf_writer.write_all(info).unwrap()
    }

    // Give PathBuf of all files
    fn collect(to_install: &Vec<AppInfo>) -> Vec<AppInfo> {
        let mut apps: Vec<AppInfo> = Vec::new();
        for app in to_install {
            if app.install {
                let p = app.path.to_string();
                apps.push(app.clone());
            }
        }
        apps
    }

    // Make a command to install and put in a Vec to return after Copy ALl files in Appdata.
    fn download(to_copy: &Vec<AppInfo>) {
        let (tx, rx) = mpsc::channel();

        to_copy.iter().for_each(|app| {
            let path = PathBuf::from(app.path.to_string());
            println!("FILE PATH TO DOWNLOAD {}",&path.to_str().to_owned().unwrap().to_string());
            let file_name = match path.file_name() {
                Some(name) => name.to_string_lossy().into_owned(),
                None => return,
            };

            let appdata = path_buf_to_string!(APPDATA_PATH);
            let cloned = path.clone();

            println!("Downloading {} {}", path_buf_to_string!(path), appdata);

            let tx = tx.clone();
            rayon::spawn(move || {
                FileHandler::copy(cloned.to_str().unwrap(), &appdata, &file_name);
                tx.send(()).expect("Error: On send end signal");
            });
        });

        for _ in 0..to_copy.len() {
            rx.recv().expect("Error: On receive end signal");
        }
    }

    fn copy(path: &str, to: &str, file_name: &str) {

        let final_path = PathBuf::from(to).join(file_name);

        let mut buf_reader = BufReader::new(
            fs::File::open(path).expect("Failed to open source file")
        );

        let mut buf_writer = BufWriter::new(
            fs::File::create(final_path)
                .expect("Failed to create output file")
        );

        let mut buf = vec![];
        let content = buf_reader.read_to_end(&mut buf).unwrap();
        buf_writer.write_all(&buf).unwrap();

        buf_writer.flush().expect("Failed to flush data to output file");

    }

    fn handle_installer (installer_type: InstallerType, path_buf: PathBuf, unattended: &bool) {
        let command = format!("{}", path_buf_to_string!(path_buf));

        fn handle_msi(command: &str, unattended: &bool) {
            // Construct the PowerShell command
            let mut cmd = Command::new("powershell");
            cmd.arg("-Command");

            // Build the command string to start the MSI
            let msi_command = format!("Start-Process '{}' -ArgumentList '/quiet', '/passive'", command);
            cmd.arg(&msi_command);

            // Execute the command
            let mut ins = cmd
                .stdout(Stdio::inherit()) // Redirect stdout of the spawned process to the parent process
                .stderr(Stdio::inherit()) // Redirect stderr of the spawned process to the parent process
                .spawn()
                .expect("Failed to execute command");

            // Wait for the command to finish
            let status = ins.wait().expect("Failed to wait for command");
            println!("Command exited with: {}", status);
        }

        fn handle_exe(command: &str) {
            let mut cmd = Command::new("cmd");
            let mut real = command.to_string();


            if command.contains("libreoffice") || command.contains("Libreoffice") {
                cmd.arg("/C").arg(&real).arg("/passive");
            } else if command.contains("Chrome") || command.contains("chrome") {
                cmd.arg("/C").arg(&real).arg("/install");
            } else if command.contains("Adobe Reader") || command.contains("adobe Reader") || command.contains("Adobe reader") {
                cmd.arg("/C").arg(&real).arg("/sAll").arg("/rs").arg("/msi").arg("EULA_ACCEPT=YES");
            } else if command.contains("7z") || command.contains("7Z") || command.contains("7zip") || command.contains("7Zip") {
                cmd.arg("/C").arg(&real).arg("/S");
            }

            println!("COMMAND: {}",&real);

            let mut ins = cmd
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to execute command");

            // Wait for the command to finish
            let status = ins.wait().expect("Failed to wait for command");
            println!("Command exited with: {}", status);
        };

        let thr = if installer_type == InstallerType::MSI { handle_msi(&command, unattended) } else { handle_exe(&command) };
        let handle = thread::spawn(move || thr);
        handle.join().unwrap();

    }

    // Execute all files
    pub(crate) async fn install(to_execute: Vec<AppInfo>, unattended: bool, online: bool) {

        let to_download = FileHandler::collect(&to_execute);
        if online {
            println!("ONLINE");
            FileHandler::download(&to_download);
        } else {
            println!("NO ONLINE");

        }

        let mut all_status: Vec<ExitStatus> = Vec::new();

        for app in to_execute {
            let file_name = app.name.to_string();
            println!("FILENAME: {}",&file_name);

            let appdata = PathBuf::from(&APPDATA_PATH.to_str().expect("Error: Appdata to string"));
            let app_new_path = appdata.join(file_name);

            println!("APPNEWPATH: {}",app_new_path.to_str().unwrap().to_string());

            let ext = match app_new_path.extension().and_then(|ext| ext.to_str()) {
                Some("msi") => InstallerType::MSI,
                _ => InstallerType::EXE,
            };

            FileHandler::handle_installer(ext, app_new_path, &unattended)



    }

}
}