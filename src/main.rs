// TODO: move from this dogshit format to using struct
// TODO: use clap
// TODO: support regex
// TODO: support X11 users with `clipboard-rs`
use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand};
use orion::aead;
use rand::distributions::{Alphanumeric, DistString};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use termion::input::TermRead;

fn main() {
    App::new(Cli::parse()).run();
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct Metadata {
    date: Option<SystemTime>,
    text: Option<String>,
    url: Option<String>,
}

impl From<&CreateArgs> for Metadata {
    fn from(value: &CreateArgs) -> Self {
        Self {
            url: value.url.clone(),
            text: value.text.clone(),
            date: if value.with_date {
                Some(SystemTime::now())
            } else {
                None
            },
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct PasswordEntry {
    password: String,
    metadata: Metadata,
}

impl PasswordEntry {
    fn new(password: String, metadata: Metadata) -> Self {
        Self { password, metadata }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct Format {
    passwords: HashMap<String, PasswordEntry>,
}

impl Format {
    fn exist(&self, entry: &String) -> bool {
        self.passwords.get(entry).is_some()
    }
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
struct CreateArgs {
    #[arg(value_name = "TAG")]
    tag: String,
    #[arg(value_name = "date", long, action = clap::ArgAction::SetTrue)]
    with_date: bool,
    #[arg(value_name = "text", long)]
    text: Option<String>,
    #[arg(value_name = "url", long)]
    url: Option<String>,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
struct ListArgs {
    #[arg(long, action = clap::ArgAction::SetTrue)]
    with_date: bool,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    with_text: bool,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    with_url: bool,
}

impl ListArgs {
    fn with_date(&self) -> bool {
        self.with_date
    }
    fn with_text(&self) -> bool {
        self.with_text
    }
    fn with_url(&self) -> bool {
        self.with_url
    }
}

impl Default for ListArgs {
    fn default() -> Self {
        Self {
            with_date: Default::default(),
            with_text: Default::default(),
            with_url: Default::default(),
        }
    }
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
struct GetArgs {
    #[arg(value_name = "TAG")]
    tag: String,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    with_date: bool,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    with_text: bool,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    with_url: bool,
}

impl GetArgs {
    fn with_date(&self) -> bool {
        self.with_date
    }
    fn with_text(&self) -> bool {
        self.with_text
    }
    fn with_url(&self) -> bool {
        self.with_url
    }
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
struct DeleteArgs {
    #[arg(value_name = "TAG")]
    tag: String,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
struct CopyArgs {
    #[arg(value_name = "TAG")]
    tag: String,
    #[arg(required = false, long)]
    _tag: bool,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
#[group(multiple = false)]
struct EncryptArgs {
    #[arg(required = false, long)]
    key: Option<String>,
    #[arg(required = false, long)]
    key_file: Option<String>,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
#[group(multiple = false)]
struct DecryptArgs {
    #[arg(required = false, long)]
    key: Option<String>,
    #[arg(required = false, long)]
    key_file: Option<String>,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
#[group(multiple = true)]
struct UpdateArgs {
    #[arg(value_name = "TAG")]
    tag: String,
    #[arg(long)]
    with_date: bool,
    #[arg(long)]
    text: Option<String>,
    #[arg(long)]
    url: Option<String>,
    #[arg(long)]
    with_password: Option<String>,
}

#[derive(Debug, Parser)]
#[command(name = "mmp")]
#[command(about = "Personal Password Manager", long_about = None, version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(required = false, long, default_value_t = String::from("~/.local/share/mmp/"))]
    pwd_path: String,
    #[arg(required = false, long, default_value_t = String::from("pwd.yaml"))]
    pwd_name: String,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = false)]
    Create(CreateArgs),
    #[command(arg_required_else_help = false)]
    List(ListArgs),
    #[command(arg_required_else_help = false)]
    Copy(CopyArgs),
    #[command(arg_required_else_help = false)]
    Get(GetArgs),
    #[command(arg_required_else_help = false)]
    Delete(DeleteArgs),
    #[command(arg_required_else_help = false)]
    Encrypt(EncryptArgs),
    #[command(arg_required_else_help = false)]
    Decrypt(DecryptArgs),
    #[command(arg_required_else_help = false)]
    Update(UpdateArgs),
}

struct App {
    cli: Cli,
}

impl App {
    fn new(cli: Cli) -> Self {
        Self { cli }
    }

    fn create(&self) {
        if let Commands::Create(args) = &self.cli.command {
            {
                let mut file = self.read_with_format();
                if file.passwords.get(&args.tag).is_some() {
                    eprintln!("Error: Already existing tag!");
                    eprintln!(
                        "The provided tag '{}' already exist, try another one!",
                        args.tag.clone()
                    );
                    return;
                }

                let password = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
                let metadata = Metadata::from(args);
                file.passwords
                    .insert(args.tag.clone(), PasswordEntry::new(password, metadata));
                self.write(&file);
            }
        }
    }

    fn list(&self) {
        if let Commands::List(args) = &self.cli.command {
            let file = self.read_with_format();
            if file.passwords.is_empty() {
                println!("No passwords generated!");
                return;
            }

            for (name, entry) in file.passwords {
                println!("name: {}", name);
                println!("  password : {}", entry.password); // never show full?
                let metadata = entry.metadata;
                if let Some(date) = metadata.date
                    && args.with_date()
                {
                    let datetime: DateTime<Utc> = date.into();
                    println!("  date : {}", datetime.to_rfc2822());
                }
                if let Some(url) = metadata.url
                    && args.with_url()
                {
                    println!("  url : {}", url);
                }
                if let Some(text) = metadata.text
                    && args.with_text()
                {
                    println!("  text: {}", text);
                }
                println!();
            }
        }
    }

    fn run(&self) {
        match &self.cli.command {
            Commands::Create(_) => self.create(),
            Commands::List(_) => self.list(),
            Commands::Get(_) => self.get(),
            Commands::Delete(_) => self.delete(),
            Commands::Copy(_) => self.copy(),
            Commands::Encrypt(_) => self.encrypt(),
            Commands::Decrypt(_) => self.decrypt(),
            Commands::Update(_) => self.update(),
        }
    }

    fn get(&self) {
        if let Commands::Get(args) = &self.cli.command {
            let file = self.read_with_format();
            if file.passwords.is_empty() {
                println!("No passwords generated!");
                return;
            }
            let re = Regex::new(&args.tag).unwrap();
            if let Some((name, entry)) = file
                .passwords
                .iter()
                .filter(|(k, _)| re.is_match(k))
                .min_by_key(|(k, _)| k.len())
            {
                println!("name: {}", name);
                println!("  password : {}", entry.password); // never show full?
                let metadata = &entry.metadata;
                if let Some(date) = metadata.date
                    && args.with_date()
                {
                    let datetime: DateTime<Utc> = date.into();
                    println!("  date : {}", datetime.to_rfc2822());
                }
                if let Some(url) = &metadata.url
                    && args.with_url()
                {
                    println!("  url : {}", url);
                }
                if let Some(text) = &metadata.text
                    && args.with_text()
                {
                    println!("  text: {}", text);
                }
                println!();
            }
        }
    }

    fn delete(&self) {
        if let Commands::Delete(args) = &self.cli.command {
            if args.tag.is_empty() {
                eprintln!("Error: Missing tag!");
                eprintln!("Expected a tag name after 'mmp delete' , but got None");
                return;
            }
            let mut file = self.read_with_format();
            if !file.exist(&args.tag) {
                eprintln!("Error: Tag does not exist!");
                eprintln!("Entry `{}` does not exist", args.tag);
                return;
            }
            file.passwords.remove(&args.tag);
            self.write(&file);
        }
    }

    fn copy(&self) {
        if let Commands::Copy(args) = &self.cli.command {
            if args.tag.is_empty() {
                eprintln!("Error: Missing tag!");
                eprintln!("Expected a tag name after 'mmp delete' , but got None");
                return;
            }
            let file = self.read_with_format();
            if let Some(entry) = file.passwords.get(&args.tag) {
                let target = if args._tag {
                    args.tag.clone()
                } else {
                    entry.password.clone()
                };
                let mut command = Command::new("wl-copy").arg(target).spawn().unwrap();
                let status = command.wait().unwrap();
                if status.success() {
                    println!("Password copied successfully to clipboard!");
                } else {
                    eprintln!("Error: Couldn't copy password to clipboard");
                    eprintln!("Tip: Consier copying the password manually from the terminal");
                }
            } else {
                eprintln!("Error: Tag does not exist!");
                eprintln!("Entry `{}` does not exist", args.tag);
                return;
            }
        }
    }

    fn encrypt(&self) {
        if let Commands::Encrypt(args) = &self.cli.command {
            let mut buffer = String::new();
            let key = if let Some(key) = &args.key {
                key.clone()
            } else if let Some(key_file) = &args.key_file {
                std::fs::read_to_string(key_file).unwrap()
            } else if atty::isnt(atty::Stream::Stdin) {
                io::stdin().read_to_string(&mut buffer).unwrap();
                buffer
            } else {
                println!("Input your encryption key:");
                TermRead::read_passwd(&mut io::stdin(), &mut io::stdout())
                    .unwrap()
                    .unwrap()
            };
            println!("{}", key);

            // hashing the key
            let mut hasher = Sha256::new();
            hasher.update(key);
            let hashed_key = hasher.finalize();
            // creating a secret key
            let secret_key = aead::SecretKey::from_slice(hashed_key.as_slice()).unwrap();
            // encrypting file content
            let content = self.read();
            let ciphertext = aead::seal(&secret_key, content.as_slice()).unwrap();
            // writing encrypted content to file
            let mut pwds_path = std::env::var("HOME").unwrap();
            // TODO: change this to something defined in self instead
            pwds_path.push_str("/.local/share/mmp/pwd.yaml");
            fs::write(pwds_path, ciphertext).unwrap();
            println!("Passwords file encrypted successfully");
        }
    }

    fn decrypt(&self) {
        if let Commands::Decrypt(args) = &self.cli.command {
            let mut buffer = String::new();
            let key = if let Some(key) = &args.key {
                key.clone()
            } else if let Some(key_file) = &args.key_file {
                std::fs::read_to_string(key_file).unwrap()
            } else if atty::isnt(atty::Stream::Stdin) {
                io::stdin().read_to_string(&mut buffer).unwrap();
                buffer
            } else {
                println!("Input your encryption key:");
                TermRead::read_passwd(&mut io::stdin(), &mut io::stdout())
                    .unwrap()
                    .unwrap()
            };

            // hashing the key
            let mut hasher = Sha256::new();
            hasher.update(key);
            let hashed_key = hasher.finalize();
            // reading file content
            let content = self.read();
            // creating a secret key
            let secret_key = aead::SecretKey::from_slice(hashed_key.as_slice()).unwrap();
            // decrypt file with key
            let decrypted_data = aead::open(&secret_key, &content);
            match decrypted_data {
                Ok(decrypted_data) => {
                    let decrypted_string: String = decrypted_data
                        .iter()
                        .map(|&value| value as u8 as char)
                        .collect();
                    self.write_str(&decrypted_string);
                    println!("Passwords file decrypted successfully");
                }
                Err(_) => {
                    println!("Error: Couldn't decrypt file");
                    println!("File propably alredy decrypted, try the 'list' option to check");
                }
            }
        }
    }

    fn join_dir_name<P: AsRef<Path>, N: AsRef<Path>>(dir: P, name: N) -> PathBuf {
        let mut p = PathBuf::from(dir.as_ref());
        p.push(name);
        p
    }

    fn dirs_home() -> Option<PathBuf> {
        if let Ok(home) = std::env::var("HOME") {
            return Some(PathBuf::from(home));
        }
        None
    }

    fn expand_tilde(path: &str) -> PathBuf {
        if let Some(stripped) = path.strip_prefix("~/") {
            if let Some(home) = Self::dirs_home() {
                return home.join(stripped);
            }
        }
        PathBuf::from(path)
    }

    fn read(&self) -> Vec<u8> {
        let dir = Self::expand_tilde(&self.cli.pwd_path.clone());
        let file_name = self.cli.pwd_name.clone();
        let path = Self::join_dir_name(&dir, &file_name);
        let _ = std::fs::create_dir_all(&path);

        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&path)
            .unwrap();
        let mut content = vec![];
        file.read_to_end(&mut content).unwrap();
        content
    }

    fn read_with_format(&self) -> Format {
        serde_yaml::from_slice(&self.read()).unwrap()
    }

    fn write_str(&self, content: &str) {
        let dir = Self::expand_tilde(&self.cli.pwd_path.clone());
        let file_name = self.cli.pwd_name.clone();
        let path = Self::join_dir_name(&dir, &file_name);
        let _ = std::fs::create_dir_all(&path);
        // createing hte file if does not eixt
        OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&path)
            .unwrap();
        // writing to the file
        fs::write(&path, content).unwrap();
    }

    fn write(&self, content: &Format) {
        let new_file_content = serde_yaml::to_string(&content).unwrap();
        self.write_str(&new_file_content);
    }

    fn update(&self) {
        if let Commands::Update(args) = &self.cli.command {
            let mut format = self.read_with_format();
            if let Some(pwd) = format.passwords.get_mut(&args.tag) {
                if let Some(text) = &args.text {
                    pwd.metadata.text = Some(text.clone());
                }
                if let Some(url) = &args.url {
                    pwd.metadata.url = Some(url.clone());
                }
                if args.with_date {
                    pwd.metadata.date = Some(SystemTime::now());
                }
                if let Some(password) = &args.with_password {
                    pwd.password = password.clone();
                } else if atty::isnt(atty::Stream::Stdin) {
                    let mut buffer = String::new();
                    io::stdin().read_to_string(&mut buffer).unwrap();
                    pwd.password = buffer;
                }
            }
            self.write(&format);
        }
    }
}
