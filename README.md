# Congularity

**Anti-singularity. Until all are one.**

*With this project, you can easily create NixOS images, virtual machines, or simply prepare `.nix` files for further use.*

## Status
🔜 Under development. Currently for testing purposes only. The only configurable parameters are the type of image to be built and the set of pre-installed packages.
## Notes
  First, download the project. Then, select your desired build output and specify the destination directory (by default, it is the `RESULT` folder within the project). Click the "Build" button. That is all for now.

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
