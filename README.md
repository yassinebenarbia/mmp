# Manage My Passwords
> Use at your own risk
___
## About
This is my Password manager, it stores your password in a optionaly symetrically encrypted file along with their equivalent *Tag*, this tag could be used later to retrieve the password of your need.    

The can be will be encrypted with a key provided by the user using the `encrypt` subcommand, and decrypted by the exact same key using the `decrypt` subcommand.  
## Usage
- Create a password   
`mmp create <tag>`      
-- Example: `mmp create facebook` // creates a password by the tag "Facebook"
- Copy a password to the clipboard  
`mmp copy <tag>`  
-- Example: `mmp copy facebook` // copys the password with the tag "Facebook" to clipboard
- Delete a password  
`mmp delete <tag>`    
-- Example: `mmp delete facebook` // deletes the password with the tage "facebook" from the passwords file
- List all passwors with their respective tags  
`mmp list`    
-- Example: `mmp list` // self-evident
- Encrypt passwords file  
`mmp encrypt`    
-- Example: `mmp encrypt` // prompt you to insert an encryption key that will be used to encrypt all your passwords
- Decrypt passwords file  
`mmp decrypt`  
-- Example: `mmp encrypt` // prompt you to insert the decryption key to decrypt your passwords  
- Display help page for a specific subcommand  
`mmp help <subcommand>`   
-- example: `mmp help create` // displays the help page of the "create" subcommand

## _Notes_ 
> - All subcommands except `decrypt` can't be used unless the file is decrypted by the user.    
> - All passwords will be stored on the `~/.local/share/mmp/pwd.yaml`, if the file does not exist, it will be created.
