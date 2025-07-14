# bad bonelab mod manager
cli app to manage bonelab mods

```sh
[diced@mangoes ~]$ bonelab-mod-manager -h
bonelab-mod-manager 0.1.0
Bonelab mod manager

USAGE:
    bonelab-mod-manager [FLAGS] [OPTIONS] <api-key> <mod-folder>

FLAGS:
    -h, --help             Prints help information
    -s, --subscribe-all    subscribe to all mods
    -u, --update-all       update all mods / does not do anything rn
    -V, --version          Prints version information

OPTIONS:
    -e, --email <email>    email to log into mod.io

ARGS:
    <api-key>       your mod.io api key
    <mod-folder>    folder where bonelab mods are, usually something like
                    /C:/users/steamuser/AppData/LocalLow/Stress Level Zero/BONELAB/Mods/
```

# help
- the first time you run this you will want to `-e` to login to mod.io

# example usage
```bash
bonelab-mod-manager -s "$(cat modio_api_key)" "$(cat modio_folder)"
```
- `$(...)` is a bash-ism to place the output of a command there
- this is because its a pain to type it every time

# install
```bash
cargo install --git https://github.com/aerits/bonelab-mod-manager
```