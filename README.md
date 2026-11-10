# Congularity

**Anti-singularity. Until all are one.**

*With this project, you can easily create NixOS images, virtual machines, or simply prepare `.nix` files for further use.*

## Status
🔜 Under development. Configurable parameters are the type of image to be built, the set of pre-installed packages, kernel versions (outcluding the last one in the list, it's a placeholder) and ability to manage users (by default - "user" with "user" unhashed password),  passwords themselves are not stored—only their hashes are used.
## Notes
  First, download the project. Then, select your desired build output and specify the destination directory (by default, it is the `RESULT` folder within the project). Click the "Build" button. That is all for now. If a `git` error occurs, run the following commands manually in the directory designated for the output:
  ```bash
  git init
  git add .
  ```
  
Running this requires git, GTK4 and Nix to be installed.

## NIX
  ```bash
  nix run github:ABYSSudo/congularity
  ```
(Not tested)

## Linux
```bash
git clone https://github.com/ABYSSudo/congularity.git
cd congularity
cargo run
```
## Windows
  Not supported lol.
