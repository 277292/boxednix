# boxedNix
Encrypts and automaticlly hashes sensitive parts of your Nix configuration.


## How it's works
- **boxedNix** opens or creates an [age](https://age-encryption.org/v1)-encrypted file.
- It launches your preferred editor with the decrypted contents of the file.
- Sensitive data (e.g. passwords) can be marked with keywords such as `bcrypt`, `sha512`, `psk`.
- On save:
  - Changes are re-encrypted into the source file.
  - A new `.nix` file is generated, with marked secrets replaced by their hashes.
- The generated `.nix` file can be imported as usual via `import ./.boxednix/<file>.nix`.

See [Usage](./README.md#-usage) for a more detailed example.

**boxedNix** is built on the [rage](https://github.com/str4d/rage) implementaion of *age* and the [rnix-parser](https://github.com/nix-community/rnix-parser).




## Motivation
The idea behind **boxedNix** is to lower the barrier to using a tool for secret management in the first place.
Tools like [sops-nix](https://github.com/Mic92/sops-nix) or [agenix](https://github.com/ryantm/agenix) are excellent at providing a strong security guarantess. However, convenience and security often get in each other’s way.
**boxedNix** aims to strike a balance: convenient to use, while still offering a reasonable level fo security.

> [!warning]
> **boxedNix** is in an early development stage and has not been thoroughly tested.
> Use it at your own risk. The author assumes no responsibility or liability for any damages or issues resulting from its use.




## 🚀 Installation

### Global
```bash
nix profile install github:277292/boxednix
```

### Flake
```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/<your-version>";
    boxednix = {
      url = "github:277292/boxednix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {self, nixpkgs, boxednix, ...}: {
    nixosConfigurations.janes-server = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./configuration.nix

        ({config, pkgs, ...}: {
          nixpkgs.overlays = [boxednix.overlays.default];
          environment.systemPackages = [pkgs.boxednix];
        })
      ];
    };
  };
}
```




## 📦 Usage

### 1️⃣  Create a boxedNix configuration
Navigate to the root of your Nix configuration:

```bash
cd /path/to/your/configuration
```

Create a new configuration:
```bash
bx create janes-system
```

This will:  
- Create a `boxednix.toml` file:  
  ```toml
  identity = "~/.config/boxednix/janes-system"

  [generate]
  dir = ".boxednix"
  gitignore = "Always"
  ```
- Generate an *age* key pair in `~/.config/boxednix/janes-system` (if it does not exist).  

💡 **Tip:** Use `-p` to protect the key with a passphrase


### 2️⃣ Create or edit an encrypted file

```bash
bx mail-accs.age
```

- If the file exists → it will be decrypted to a temporary file and opened in your default editor.  
- If it does not exist → it will open with a default template.  

#### Example encrypted config (`mail-accs.age`):
```nix
{
  bcrypt,
  sha512,
  psk
}: {
  mailserver.loginAccounts = {
    "jane@example.uk".hashedPassword = bcrypt "Jane's strong password";
  };
}
```

After saving, **boxedNix** will:
1. Encrypt the file back to `mail-accs.age`.
2. Generate `.boxednix/mail-accs.nix`:
```nix
{
  mailserver.loginAccounts = {
    "jane@example.uk".hashedPassword = "$2b$12$AweSl1qJwmUmskOaSqAd8Odq6o9TYnuINRDwbl3R6BFWAR6MKg4.K";
  };
}
```


### 3️⃣ Import the generated file into Nix
```nix
{config, ...}: {
  imports = [./.boxednix/mail-accs.nix];
  mailserver.enable = true;
}
```


### 4️⃣ Alternative: Writing attribute sets instead of modules
You don’t have to write full Nix modules. Instead, you can define attribute sets to organize related secrets or configurations.

#### Example (`keys.age`):
```nix
{
  bcrypt,
  sha512,
  psk
}: {
  serverA = {
    users.jane = sha512 "Another strong password for Jane";
  };
  serverB = {
    users.jane = sha512 "Not the same password";
  };
}
```

Import:
```nix
outputs = {self, nixpkgs, ...}: {
  nixosConfigurations.janes-serverA = nixpkgs.lib.nixosSystem {
    ...
    specialArgs = {
      boxed = import ./.boxednix/keys.nix;
    };
  };
}
```

Usage:
```nix
{config, boxed, ...}: {
  users.users.jane.hashedPassword = boxed.serverA.users.jane;
}
```




## 🛑 Known problems
- **boxedNix does not fully evaluate Nix code.** It only scans for keywords and hashes secrets accordingly.
- In general, boxedNix will not generate a file if there is a syntax error. However, if the syntax is valid but not in the expected form, a password may remain unhashed and appear in the generated file.

### Lists
If you use lists (although it's currently unclear why you would), ensure that the keyword and string are correctly wrapped in parentheses:
```nix
list = [
  (bcrypt "your password")
];
```
If you forget the parentheses:
```nix
list = [
  bcrypt "your password"
];
```
This will be taken literally in the generated file, and your password will not be hashed.

### Functions
```nix
let
  hash_sha512 = key: sha512 key;
in
  users.users.jane.hashedPassword = hash_sha512 "janes key";
```
Using functions like this will not work, and the key will appear in plaintext in the generated file.




## 🔒 Security notes
- Hashed secrets are stored in the Nix store.
- The generated `.nix` files are stored in a directory with a `.gitignore` that excludes its contents to prevent accidental commits. This is automatically created by **boxedNix** if configured accordingly, but you must ensure that the files do not accidentally get committed to your repository.

## 🙌 Contributing
Feel free to open an issue or submit a pull request if you encounter a problem or have an idea for improvement — contributions related to **boxedNix** are very welcome.




## 📄 License
boxedNix is licensed under the [GNU General Public License v3.0 only (GPL-3.0-only)](./LICENSES/GPL-3.0-only.txt). All dependencies dual-licensed under [MIT](https://opensource.org/licenses/MIT) OR [Unlicense](https://unlicense.org/) are used under the terms of the MIT license.
