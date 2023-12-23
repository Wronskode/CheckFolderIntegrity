// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use std::{fs, io};
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![verify])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}   

#[tauri::command]
async fn verify(mut folder1: String, mut folder2: String, secure: bool) -> Value {
    let now = Instant::now();
    let is_unix = cfg!(unix);
    match folder1.chars().last() {
        Some(k) => {
            if k != '/' {
                if is_unix {
                    folder1.push('/');
                }
            }
            else if k != '\\' {
                if !is_unix {
                    folder1.push('\\');
                }
            }
        },
        None => ()
    }
    match folder2.chars().last() {
        Some(k) => {
            if k != '/' {
                if is_unix {
                    folder2.push('/');
                }
            }
            else if k != '\\' {
                if !is_unix {
                    folder1.push('\\');
                }
            }
        },
        None => ()
    }
    let excluded_folders = Arc::new(Mutex::new(HashSet::new()));
    // excluded_folders
    //     .lock()
    //     .unwrap()
    //     .insert("System Volume Information".to_string()); // Add here excluded folders like that
    let t1 = {
        let excluded_folders = excluded_folders.clone();
        thread::spawn(move || {
            check_folder(folder1.clone().into(), folder1.len(), secure, excluded_folders)
        })
    };
    let t2 = {
        let excluded_folders = excluded_folders.clone();
        thread::spawn(move || {
            check_folder(folder2.clone().into(), folder2.len(), secure, excluded_folders)
        })
    };
    let f1 = match t1.join() {
        Ok(hm) => hm,
        Err(_) => HashMap::new()
    };
    let f2 = match t2.join() {
        Ok(hm) => hm,
        Err(_) => HashMap::new()
    };

    // TODO : Optimize this part
    let mut files_only_in_f1: Vec<&str> = vec![];
    let mut files_only_in_f2: Vec<&str> = vec![];
    let mut diff_files = HashSet::new();
    let mut all_files = HashSet::new();
    let mut files_in_f1 = vec![];
    let mut files_in_f2 = vec![];
    for element in &f1 {
        files_in_f1.push(element.0);
        if !f2.contains_key(element.0) {
            files_only_in_f1.push(element.0.to_str().unwrap());
        }
        else {
            if element.1 != f2.get(element.0).unwrap() {
                diff_files.insert(element.0.to_str().unwrap());
            }
        }
        all_files.insert(element.0);
    }
    for element in &f2 {
        files_in_f2.push(element.0);
        if !f1.contains_key(element.0) {
            files_only_in_f2.push(element.0.to_str().unwrap());
        }
        else {
            if element.1 != f1.get(element.0).unwrap() {
                diff_files.insert(element.0.to_str().unwrap());
            }
        }
        all_files.insert(element.0);
    }
    let result = json!({
        "only folder1": files_only_in_f1,
        "only folder2": files_only_in_f2,
        "different files": diff_files,
        "Length of folder1": f1.len(),
        "Length of folder2": f2.len(),
        "time": format!("{:?}", now.elapsed()),
        "all files": all_files,
        "f1_files": files_in_f1,
        "f2_files": files_in_f2,
        "excluded folders": *excluded_folders
    });
    return result;
}

fn check_folder(path: PathBuf, len: usize, secure: bool, excluded_folders: Arc<Mutex<HashSet<String>>>) -> HashMap<PathBuf, String> {
    let mut files_hashs = HashMap::new();
    let folder = match fs::read_dir(&path) {
        Err(_) => {
            println!("Unable to open this directory {:?}\n(verify permissions)", path);
            excluded_folders.lock().unwrap().insert(path.to_str().unwrap().to_string());
            return files_hashs;
        },
        Ok(folder) => folder
    };
    for file in folder {
        let f = match file {
            Err(_) => continue,
            Ok(dir) => dir
        };
        let file_type = match f.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue
        };
        if file_type.is_file() {
            let relative_path: String = match f.path().to_str() {
                Some(x) => x.chars().skip(len).collect(),
                None => continue
            };
            if secure {
                let digest = hash_file(f.path());
                match digest {
                    Ok(digest) => files_hashs.insert(relative_path.into(), digest),
                    Err(_) => continue
                };
            } else {
                let metadatas = match f.metadata() {
                    Ok(meta) => meta,
                    Err(_) => continue
                };
                let size = metadatas.len().to_string();
                let date = match metadatas.modified() {
                    Ok(date) => date,
                    Err(_) => continue
                };
                let date_str = format!("{:?}", date);
                files_hashs.insert(relative_path.into(), date_str + " " + &size);
            }
        } else if file_type.is_dir() {
            // To skip excluded folders
            if  excluded_folders
                .lock()
                .unwrap()
                .contains(&f.file_name().to_str().unwrap().to_string())
            {
                return files_hashs;
            }
            // Idk if create a thread here is a good idea
            let excluded_folders = excluded_folders.clone();
            let h = 
            match thread::spawn(move || check_folder(f.path(), len, secure, excluded_folders)).join() {
                Ok(hash_map) => hash_map,
                Err(_) => continue
            };
            // Replace h by that if you think it's a bad idea to create a thread
            // let h = check_folder(f.path(), len, secure, excluded_folders);
            for element in h {
                files_hashs.insert(element.0, element.1);
            }
        }
    }
    files_hashs
}

fn hash_file(path: PathBuf) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).unwrap();
    let digest = hasher.finalize();
    Ok(format!("{:x}", digest))
}
