# Manage My Passwords
> Use at your own risk
___
## About
This is my Password manager, it stores your password in a optionaly symetrically encrypted file along with their equivalent *Tag* and some *Metadata*, this tag could be used later to retrieve the password of your need.    

passwords can be encrypted with a key provided by the user using the `encrypt` subcommand, and decrypted using the `decrypt` subcommand.
## Usage
If you have nix installed, you can try:
```nix
nix run --impure

```
and you'll get as output:

```
Personal Password Manager

Usage: mmp [OPTIONS] <COMMAND>

Commands:
  create   Creates a password given a tag
  list     List available passwords, won't work if passwords list is encrypted
  copy     Copys the password with the tag to clipboard
  get      Prints the password entry with tag that matches the closes to regex input
  delete   Deletes the password entry that matches the exact tag input
  encrypt  Encrypts a passowrds using key
  decrypt  Decrypts encrypted passowrds using key
  update   changes info (metadata/password) of an entry with the exact tag input
  help     Print this message or the help of the given subcommand(s)

Options:
      --pwd-path <PWD_PATH>  Path to passwords file [default: ~/.local/share/mmp/]
      --pwd-name <PWD_NAME>  Name of the passwords file [default: pwd.yaml]
  -h, --help                 Print help
  -V, --version              Print version
```
You can run `mmp [OPTION] --help` for more info about each option

## _Notes_ 
> - All subcommands except `decrypt` can't be used unless the file is decrypted by the user.    
> - All passwords will be stored on the `~/.local/share/mmp/pwd.yaml`, if the file does not exist, it will be created unless specified otherwise by the `--pwd-path` and `--pwd-name` options.
