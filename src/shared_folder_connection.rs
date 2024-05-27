use std::ffi::OsStr;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use slint::SharedString;

use crate::AppInfo;
use crate::connection_error::ConnectionError;

pub struct SHFConnection {}
impl SHFConnection {
    pub fn connect(path: &str) -> ExitStatus {
        let folder = PathBuf::from(&path);
        // Network drive path, username, and password
        let final_path = folder.to_str().expect("Err").to_string();

        let command = format!(r"net use X: {} /user:Unattended User1",final_path);
        //self.command = &command;
        Command::new("cmd")
            .args(&["/C",&command])
            .status()
            .expect("Error: Failed connection")


    }

    pub fn list_files(path: &str) -> Result<Vec<AppInfo>, ConnectionError> {

        println!("list files in {}",path);

        // Read dir

        match fs::read_dir(Path::new(path)) {
            Ok(files) => {
                // Vec of names
                let mut file_names: Vec<AppInfo> = Vec::new();

                // Iter files
                for file in files {
                    // If file exists
                    if let Ok(ref file) = file {
                        // Get file type
                        let file_type = file.file_type().expect("Error: On get file type");

                        // Get file name
                        let file_name = file.file_name();

                        // Convert file name to string
                        let fname_string = file_name.to_str().expect("Error: On get file name").to_string();

                        // Check file is file
                        if file_type.is_file() {
                            // Check has extencion and witch extension
                            if let Some(ext) = Path::new(&file_name).extension().and_then(OsStr::to_str) {
                                // Filter exe and msi
                                if ext == "exe" || ext == "msi" {
                                    // Push
                                    let file_path: String = file.path().to_str().expect("Error: On get file path").to_string();
                                    println!("FILE LISTED: {}",file_path);
                                    file_names.push(
                                        AppInfo {
                                            name: SharedString::from(&fname_string),
                                            install: false,
                                            path: SharedString::from(SharedString::from(file_path)),
                                            ins_type: SharedString::from(ext)
                                        }
                                    );
                                }
                            }

                            // Recursive
                        } else if file_type.is_dir() {
                            // Get paths
                            let sub_path = PathBuf::from(path).join(&file_name);

                            // Get files
                            let sub_files = SHFConnection::list_files(&sub_path.to_str().unwrap()).expect("Error: On search files");

                            // Extend vec with the new other vec
                            file_names.extend(sub_files);
                        }
                    }
                }
                println!("LISTED FILES");
                Ok(file_names)
            }
            Err(e) => {
                println!("ERROR LIST FILES");
                Err(ConnectionError {
                message: format!("Error: On read dir {}: {}", path, e),
            })},
        }


    }
}
