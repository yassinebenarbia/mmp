# My Password Manager
> Use at your own risk
___
## About
This is my Password manager, it stores your password in a optionaly symetrically encrypted file along with their equivalent *Tag*, this tag could be used later to retrieve the password of your need.    

The can be will be encrypted with a key provided by the user using the `encrypt` subcommand, and decrypted by the exact same key using the `decrypt` subcommand.  
## Usage
- Create a password   
`mmp create <tag>`    
- Copy a password to the clipboard  
`mmp copy <tag>`
- Delete a password  
`mmp delete <tag>`  
- List all passwors with their respective tags  
`mmp list`  
- Encrypt passwords file  
`mmp encrypt`  
- Decrypt passwords file  
`mmp decrypt`  
- Display help page for a specific subcommand  
`mmp help <subcommand>`    

## _Notes_ 
> - All subcommands except `decrypt` can't be used unless the file is decrypted by the user.    
> - All passwords will be stored on the `~/.local/share/mmp/pwd.yaml`, if the file does not exist, it will be created.