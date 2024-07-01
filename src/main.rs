use std::{collections::HashMap, env};
use copypasta::{ClipboardContext, ClipboardProvider};
use rand::distributions::{Alphanumeric, DistString};
use std::fs;
use serde_yaml::{Deserializer, Mapping, Sequence, Serializer, Value};
use serde::{Serialize, Deserialize};


fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    handle_args(args);
}
fn default_password() -> Value {
    let map = Mapping::new();
    return Value::Sequence(vec![]);
}

fn default_path() -> String {
    String::new()
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct Format {
    #[serde(default = "default_password")]
    passwords: Value,
    #[serde(default = "default_path")]
    path: String
}

fn create(tag: &String) {
    let string = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    let f = fs::read("pwd.yaml").unwrap();
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
                        return;
                    }
                }
            }
            let mut map = Mapping::new();
            map.insert(Value::String(tag.to_owned()), Value::String(string));
            sequence.push(Value::Mapping(map));
            let format = Format {
                passwords: Value::Sequence(sequence),
                path: things.path 
            };
            let string_format = serde_yaml::to_string(&format).unwrap();
            fs::write("pwd.yaml", string_format).unwrap();
        },
        _ => {
            println!("Error: Unexpected passwords list format!");
        }
    }
}

fn copy(tag: &String){
    let f = fs::read("pwd.yaml").unwrap();
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
        _ => todo!(),
    }
}

fn delete(tag: &String) {
    let f = fs::read("pwd.yaml").unwrap();
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
                    path: things.path 
                };
                let string_format = serde_yaml::to_string(&format).unwrap();
                fs::write("pwd.yaml", string_format).unwrap();
            }else {
                println!("Error: Tag does not exist!");
                println!("The provided tag '{}' does not exist, try another one!", tag.clone());
            }
        },
        _ => {
            println!("Error: Unexpected passwords list format!");
        }
    }
}
fn list(){
    let f = fs::read("pwd.yaml").unwrap();
    let things: Format = serde_yaml::from_slice(&f).unwrap_or(Format::default());
    if things.path.is_empty() {
        println!("No path provided!")
    }else {
            println!("path: {}", things.path)
    }
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
        _ => {}
    }
}

fn handle_args(args: Vec<String>) {
    let actions = vec![
        String::from("create"),
        String::from("copy"),
        String::from("delete"),
        String::from("list"),
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
            _ => {}
        };
    }else {
        if action.is_empty() {
            println!("Error: Argument not provided!");
            println!("Expected one of {:?}, but got {}", actions, "none");
            return;
        }else {
            println!("Error: Uncorrect argument!");
            println!("Expected one of {:?}, but got {}", actions, action);
            return;
        }
        return;
    }
}
mod Test {
    use copypasta::{ClipboardContext, ClipboardProvider};
    #[test]
    fn test1() {
        let mut ctx = ClipboardContext::new().unwrap();

        let msg = "Hello, world!";
        ctx.set_contents(msg.to_owned()).unwrap();

        let content = ctx.get_contents().unwrap();

        println!("{}", content);

    }
}
