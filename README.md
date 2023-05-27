# Plutonium Manager

This project aims to help you manage and initialise a [Plutonium](https://plutonium.pw) dedicated server for many games. For a Pterodactyl egg, please visit [this](https://github.com/Stefanuk12/Pterodactyl/blob/master/eggs/games/egg-plutonium.json).

# Usage

```
Manage and create a Plutonium dedicated server

Usage: plutonium-manager.exe [OPTIONS]

Options:
  -s, --server <path>     Install server files to a given path
  -c, --config <path>     Install server config to a given path
  -i, --iw4m <path>       Install IW4M Admin to a given path
  -l, --iw4m-log <path>   Install IW4M Admin (log server) to a given path
  -p, --plutonium <path>  Install Plutonium to a given path
  -r, --rcon <path>       Install a RCON client to a given path
  -e, --engine <game>     Specify the game version (must be provided if not only installing plutonium) [possible values: t6, t5, t4, iw5]
  -h, --help              Print help
  -V, --version           Print version
```