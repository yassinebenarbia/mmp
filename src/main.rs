use std::fs::OpenOptions;
use std::io;
use std::env;
use std::path::Path;
use copypasta::{ClipboardContext, ClipboardProvider};
use rand::distributions::{Alphanumeric, DistString};
use sha2::{Sha256, Digest};
use termion::input::TermRead;
use std::fs;
use serde_yaml::{Mapping, Value};
use serde::{Serialize, Deserialize};
use orion::aead;
use std::io::prelude::*;


fn encrypt_pfile() {
    println!("Write your encryption key: ...");
    let KEY = TermRead::read_passwd(&mut io::stdin(), &mut io::stdout())
        .unwrap().unwrap();
    println!("KEY: {}", KEY);
    // hashing the key
    let mut hasher = Sha256::new();
    hasher.update(KEY);
    let hashed_key = hasher.finalize();
    // creating a secret key
    let secret_key = aead::SecretKey::from_slice(hashed_key.as_slice()).unwrap();
    // encrypting file content
    let content = read_pfile();
    let ciphertext = aead::seal(&secret_key, content.as_slice()).unwrap();
    // writing encrypted content to file 
    let mut pwds_path = std::env::var("HOME").unwrap();
    pwds_path.push_str("/.local/share/mmp/pwd.yaml");
    fs::write(pwds_path, ciphertext).unwrap();
    println!("Passwords file encrypted successfully");
}

fn decrypt_pfile() {
    // let mut KEY = String::new();
    println!("Write your decryption key: ");
    let KEY = TermRead::read_passwd(&mut io::stdin(), &mut io::stdout())
        .unwrap().unwrap();
    println!("KEY: {}", KEY);
    // io::stdin().read_line(&mut KEY).expect("failed to readline");
    // hashing the key
    let mut hasher = Sha256::new();
    hasher.update(KEY);
    let hashed_key = hasher.finalize();
    // reading file content
    let content = read_pfile();
    // creating a secret key
    let secret_key = aead::SecretKey::from_slice(hashed_key.as_slice()).unwrap();
    // decrypt file with key
    let decrypted_data = aead::open(&secret_key, &content);
    match decrypted_data {
        Ok(decrypted_data) => {
            println!("{:?}", decrypted_data);
            let decrypted_string: String = decrypted_data.iter().map(|&value| value as u8 as char).collect();
            println!("{}", decrypted_string.clone());
            write_pfile(decrypted_string);
            println!("Passwords file decrypted successfully");
        },
        Err(_) => {
            println!("Error: Couldn't decrypt file");
            println!("File propably alredy decrypted, try the 'list' option to check");
        },
    }
}

fn read_pfile() -> Vec<u8> {
    let mut home_path = std::env::var("HOME").unwrap();
    let path =  "/.local/share/mmp/";
    let file_name = "pwd.yaml";
    home_path.push_str(path);
    // if path does not exist
    if !Path::new(&home_path.clone()).exists() {
        fs::create_dir_all(&home_path.clone()).unwrap();
    } 
    home_path.push_str(file_name);
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(home_path.clone())
        .unwrap();
    let mut content = vec![]; 
    file.read_to_end(&mut content).unwrap();
    return content
}

fn write_pfile(content: String) {
    let mut home_path = std::env::var("HOME").unwrap();
    let path =  "/.local/share/mmp/";
    let file_name = "pwd.yaml";
    home_path.push_str(path);
    // if path does not exist
    if !Path::new(&home_path.clone()).exists() {
        fs::create_dir_all(&home_path.clone()).unwrap();
    } 
    home_path.push_str(file_name);
    // createing hte file if does not eixt
    OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(home_path.clone())
        .unwrap();
    // writing to the file
    // NOTE: using the ouput of OpenOptions to write to file 
    // results in an error.
    fs::write(home_path, content).unwrap();
}


fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    handle_args(args);
}

fn default_password() -> Value {
    return Value::Sequence(vec![]);
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct Format {
    #[serde(default = "default_password")]
    passwords: Value,
}

/// creates a new password linked with the tag
fn create(tag: &String) {
    let f = read_pfile();
    let things: Format = serde_yaml::from_slice(&f).unwrap_or(Format::default());
    match things.passwords {
        Value::Sequence(mut sequence) => {
            // checkinf if key already exist
            for element in sequence.iter() {
                match element.to_owned() {
                    Value::Mapping(map) => {
                        if map.get(tag.clone()) != None {
                            println!("Error: Already existing tag!");
                            println!("The provided tag '{}' already exist, try another one!", tag.clone());
                            return;
                        }
                    },
                    _ => {
                        println!("Error: Unexpected passwords list format!");
                        println!("Check if file is encrypted, or file format convention is not adhered");
                        return;
                    }
                }
            }
            let mut map = Mapping::new();
            let password = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
            map.insert(Value::String(tag.to_owned()), Value::String(password));
            sequence.push(Value::Mapping(map));
            let format = Format {
                passwords: Value::Sequence(sequence),
            };
            let pfile_content = serde_yaml::to_string(&format).unwrap();
            write_pfile(pfile_content);
        },
        _ => {
            println!("Error: Unexpected passwords list format!");
            println!("Check if file is encrypted, or file format convention is not adhered");
        }
    }
}

/// copy the password that is linked with the tag to clipboard
fn copy(tag: &String) {
    let f = read_pfile();
    let things: Format = serde_yaml::from_slice(&f).unwrap_or(Format::default());
    match things.passwords {
        Value::Sequence(sequence) => {
            for sequence_elem in sequence {
                match sequence_elem {
                    Value::Mapping(map) => {
                        let pwd_opt = map.get(tag);
                        if let Some(pwd) = pwd_opt {
                            if let Value::String(pwd) = pwd {
                                let mut ctx = ClipboardContext::new().unwrap();
                                ctx.set_contents(pwd.to_owned()).unwrap();
                                let _ = ctx.get_contents().unwrap();
                                println!("Password moved to system clipboard");
                                return;
                            }else {
                                println!("Error: Unkown password format!");
                                return;
                            }
                        }
                    },
                    _ => todo!(),
                }
            }
            println!("Error: Password key does not exist!");
            return;
        },
        _ => {
            println!("Error: Unexpected passwords list format!");
            println!("Check if file is encrypted, or file format convention is not adhered");
        },
    }
}

/// delete the password related with the tag
fn delete(tag: &String) {
    let f = read_pfile();
    let mut found = false;
    let things: Format = serde_yaml::from_slice(&f).unwrap_or(Format::default());
    match things.passwords {
        Value::Sequence(mut sequence) => {
            sequence.retain(|v| { match v {
                    Value::Mapping(map) => {
                        if let Some(_) = map.get(tag) {
                            found = true;
                            return false;
                        }else {
                            return true;
                        }
                    }
                    _ => {
                        return false;
                    }
                }
            });
            if found {
                let format = Format {
                    passwords: Value::Sequence(sequence),
                };
                let pfile_content = serde_yaml::to_string(&format).unwrap();
                write_pfile(pfile_content);
            }else {
                println!("Error: Tag does not exist!");
                println!("The provided tag '{}' does not exist, try another one!", tag.clone());
            }
        },
        _ => {
            println!("Error: Unexpected passwords list format!");
            println!("Check if file is encrypted, or file format convention is not adhered");
        }
    }
}

/// list all passwords with their tags
fn list() {
    let f = read_pfile();
    let things: Format = serde_yaml::from_slice(&f).unwrap_or(Format::default());
    match things.passwords {
        Value::Sequence(s) => {
            if s.is_empty() {
                println!("No passwords generated!");
            }
            for i in s.iter() {
                match i {
                    Value::Mapping(m) => {
                        for v in m {
                            match v.0 {
                                Value::String(s) => {
                                    print!("{}: ", s);
                                },
                                _ => {}
                            }
                            match v.1 {
                                Value::String(s) => {
                                    println!("{:?}", s);
                                },
                                _ => {}
                            }
                        }
                    },
                    _ => {}
                }

            }
        },
        _ => {
            println!("Error: Unexpected passwords list format!");
            println!("Check if file is encrypted, or file format convention is not adhered");
        }
    }
}

fn help(tag: &String) {
    let actions = vec![
        String::from("create"),
        String::from("copy"),
        String::from("delete"),
        String::from("list"),
        String::from("encrypt"),
        String::from("decrypt"),
        String::from("help"),
    ];

    if actions.contains(tag) {
        match tag.as_str() {
            "create" => {
                println!("Creates a password linked with a tag that can be retrived later");
                println!("Schema: `mmp create <Tag>`");
            }
            "copy" => {
                println!("Copys the password linked with the tag to your clipboard");
                println!("Schema: `mmp copy <Tag>`");
            }
            "delete" => {
                println!("Deletes the password related with the tag");
                println!("Schema: `mmp delete <Tag>`");
            }
            "list" => {
                println!("List all saved passwords with their related tags");
                println!("Schema: `mmp list`");
            }
            "encrypt" => {
                println!("Encrypt the file that contains all your saved password");
                println!("Schema: `mmp encrypt`");
                println!("Note: All other actions wont work after using it, except `decrypt`");
            }
            "decrypt" => {
                println!("Decrypt the file that contains all your saved passwords");
                println!("Schema: `mmp delete <Tag>`");
            }
            "help" => {
                println!("Are you for real :|");
            }
            _ => {
                println!("Tag does not exist!");
                println!("Schema: `mmp help <subcommand>`");
                println!("Try one of the following options: {:?}", actions);
            }
        }
    }else {
        println!("Tag does not exist!");
        println!("Schema: `mmp help <subcommand>`");
        println!("Try one of the following options: {:?}", actions);
    }
}

fn handle_args(args: Vec<String>) {
    let actions = vec![
        String::from("create"),
        String::from("copy"),
        String::from("delete"),
        String::from("list"),
        String::from("encrypt"),
        String::from("decrypt"),
        String::from("help"),
    ];
    let empty = String::from("");
    let action = args.get(0).unwrap_or(&empty);

    if actions.contains(&action) {
        match action.as_str() {
            "create" => {
                let tag = args.get(1).unwrap_or(&empty);
                if tag.is_empty() {
                    println!("Error: Missing tag!");
                    println!("Expected a tag name after 'mmp create' , but got None");
                    return;
                }
                create(tag);
            },
            "copy" => {
                let tag = args.get(1).unwrap_or(&empty);
                if tag.is_empty() {
                    println!("Error: Missing tag!");
                    println!("Expected a tag name after 'mmp copy' , but got None");
                    return;
                }
                copy(tag);
            },
            "delete" => {
                let tag = args.get(1).unwrap_or(&empty);
                if tag.is_empty() {
                    println!("Error: Missing tag!");
                    println!("Expected a tag name after 'mmp delete' , but got None");
                    return;
                }
                delete(tag);
            },
            "list" => {
                list();
            }
            "encrypt" => {
                encrypt_pfile();
            }
            "decrypt" => {
                decrypt_pfile();
            }
            "help" => {
                let tag = args.get(1).unwrap_or(&empty);
                help(tag);
            },
            _ => {}
        };
    }else {
        if action.is_empty() {
            println!("Error: Argument not provided!");
            println!("Expected one of {:?}, but got {}", actions, "none");
            println!("Try `mmp help` to learn how to use `mmp`");
            return;
        }else {
            println!("Error: Uncorrect argument!");
            println!("Expected one of {:?}, but got {}", actions, action);
            println!("Try `mmp help` to learn how to use `mmp`");
            return;
        }
    }
}
