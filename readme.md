# bad bonelab mod manager
cli app to manage bonelab mods

NOTE: I coded this on linux, and have not tested this on windows, it probably does not work on windows.
EXTRA NOTE: the code uses some bash shell commands, you might want to just run this in wsl2 if you are on windows

```sh
[diced@mangoes ~]$ bonelab-mod-manager -h
bonelab-mod-manager 0.1.0
Bonelab mod manager

USAGE:
    bonelab-mod-manager [FLAGS] [OPTIONS]

FLAGS:
    -h, --help                      Prints help information
    -i, --install-all-subscribed    
    -s, --subscribe-all             subscribe to all mods
    -u, --update-all                update all mods / does not do anything rn
    -V, --version                   Prints version information

OPTIONS:
    -a, --api-key <api-key>          your mod.io api key
    -e, --email <email>              email to log into mod.io
    -m, --mod-folder <mod-folder>    folder where bonelab mods are, usually something like
                                     /C:/users/steamuser/AppData/LocalLow/Stress Level Zero/BONELAB/Mods/
```

# help
- the first time you run this you will want to `-e` to login to mod.io

# example usage
```bash
bonelab-mod-manager -s "-iu"
```
- these flags will install any new mods you subscribed to and check for updates for every mod
- you should create a `~/.config/bonelab-mod-manager/` and make 2 files
    - `modio_api_key` and `modio_mod_folder`
    - these files let you not have to put `--mod-folder` and `--api-key` in the cli options

# install
```bash
cargo install --git https://github.com/aerits/bonelab-mod-manager
```
