use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    process::Command,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use indicatif::{ProgressBar, ProgressStyle};
use modio::{
    Credentials, DownloadAction, Modio, Result, TargetPlatform, auth::Token, mods::filters::GameId,
    types::id::Id,
};
use modio::{files::filters::Id as fid, mods::Mod};
use modio::{filter::prelude::*, types::Timestamp};
use reqwest::Client;
use serde_json::{Value, json};
use structopt::StructOpt;

use crate::structs::{Isa, Manifest, ModListing, ModTarget, Object, Pallet, Reference, Root};

const BONELAB: u64 = 3809;
const TEMPLATE: &str = "[{bar}][time: {elapsed_precise}][eta: {eta_precise}] {msg}";

#[derive(Clone)]
struct InstalledMod {
    path: String,
    manifest: Manifest,
}

/// Bonelab mod manager
#[derive(structopt::StructOpt)]
struct Opt {
    /// email to log into mod.io
    #[structopt(short, long)]
    email: Option<String>,
    /// your mod.io api key
    #[structopt(short, long)]
    api_key: Option<String>,
    /// folder where bonelab mods are,
    /// usually something like /C:/users/steamuser/AppData/LocalLow/Stress Level Zero/BONELAB/Mods/
    #[structopt(short, long)]
    mod_folder: Option<PathBuf>,
    /// subscribe to all mods
    #[structopt(short, long, name = "subscribe to all mods")]
    subscribe_all: bool,
    /// update all mods / does not do anything rn
    #[structopt(short, long, name = "update all mods")]
    update_all: bool,
    #[structopt(short, long, name = "install subscribed mods")]
    install_all_subscribed: bool,
}

mod structs;

#[derive(Debug)]
struct BMMError(String);
impl std::fmt::Display for BMMError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for BMMError {}

fn throw<T>(err: &str) -> Result<T, Box<dyn std::error::Error>> {
    Err(Box::new(BMMError(err.into())))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xdg_config_home = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        // Default to ~/.config if XDG_CONFIG_HOME is not set
        let home = env::var("HOME").unwrap();
        format!("{}/.config", home)
    });
    let opt = Opt::from_args();
    let path = opt
        .mod_folder
        .clone();
    let path = match path {
        Some(mut x) => x.as_mut_os_str()
        .to_string_lossy()
        .into_owned(),
        None => {
            match fs::read_to_string(xdg_config_home.clone() + "/bonelab-mod-manager/modio_folder") {
                Ok(x) => {x},
                Err(_) => {throw("Missing modio folder")?},
            }
        },
    };  

    println!("{}", path);

    let mut files = Vec::new();
    match fs::read_dir(&path) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let file_name = entry.file_name();
                        // println!("{}", file_name.to_string_lossy());
                        files.push(file_name.to_string_lossy().into_owned());
                    }
                    Err(e) => eprintln!("Error reading entry: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Error reading directory: {}", e),
    }

    let mod_manifests: Vec<String> = files
        .iter()
        .filter(|x| x.ends_with(".manifest") && !x.starts_with("SLZ"))
        .map(|x| x.clone())
        .collect();

    let mut installed_mods = Vec::new();

    println!("Reading mods...");
    let pb = ProgressBar::new(mod_manifests.len() as u64);
    for manifest in mod_manifests {
        // println!("{}", manifest);
        let path = path.clone() + &manifest;
        let manifest = fs::read_to_string(&path)?;
        let manifest: Manifest = serde_json::from_str(&manifest)?;
        let _target = match &manifest.objects.mod_target {
            Some(x) => x,
            None => {
                continue;
            }
        };
        installed_mods.push(InstalledMod { path, manifest });
        pb.inc(1);
    }
    pb.finish_and_clear();

    let mut modio = Modio::new(Credentials::new(match opt.api_key {
        Some(x) => {x},
        None => {match fs::read_to_string(xdg_config_home.clone() + "/bonelab-mod-manager/modio_api_key") {
            Ok(x) => {x},
            Err(_) => {throw("Missing modio api key")?},
        }},
    }))?;

    let access_token = fs::read_to_string(xdg_config_home.clone() + "/bonelab-mod-manager/modio_access_token");
    if let Ok(token) = access_token {
        println!("token found");
        let token = Token {
            value: token,
            expired_at: None,
        };
        modio = modio.with_token(token);
    } else {
        modio
            .auth()
            .request_code(match &opt.email {
                Some(x) => x,
                None => {
                    panic!("No email found")
                }
            })
            .await?;
        let code = prompt("security code: ")?;
        let creds = modio.auth().security_code(&code).await?;
        if let Some(token) = &creds.token {
            println!("Access token:\n{}", token.value);
            let mut file = File::create(xdg_config_home.clone() + "/bonelab-mod-manager/modio_access_token")?;
            file.write_all(token.value.as_bytes())?;
            modio = modio.with_token(token.clone());
        } else {
            panic!("could not login");
        }
    }
    let user = modio.user().current().await?;
    println!("logged in as {}", user.unwrap().username);

    if opt.subscribe_all {
        let mut subscribed_mods = match fs::read_to_string(xdg_config_home.clone() + "/bonelab-mod-manager/modio_subscribed_mods") {
            Ok(x) => x,
            Err(_) => String::new(),
        };
        let installed_mods: Vec<&InstalledMod> = installed_mods
            .iter()
            .filter(|x| !subscribed_mods.contains(&x.path))
            .collect();
        println!("subscribing to all installed mods...");
        let pb = ProgressBar::new(installed_mods.len() as u64);
        pb.set_style(ProgressStyle::with_template(TEMPLATE).unwrap());
        'baseloop: for mod_ in installed_mods {
            pb.inc(1);
            pb.set_message(format!(
                "subscribing to {:?}",
                PathBuf::from(&mod_.path).file_name().unwrap()
            ));
            let mod_id = match &mod_.manifest.objects.mod_target {
                Some(x) => x,
                None => {
                    panic!()
                }
            };
            let modref = modio.mod_(Id::new(BONELAB), Id::new(mod_id.modId));
            let mut subscribed = false;
            let mut delay = 0;
            while !subscribed {
                match modref.clone().subscribe().await {
                    Ok(_) => {
                        subscribed = true;
                        subscribed_mods += &(mod_.path.clone() + "\n");
                    }
                    Err(x) => {
                        pb.set_message(format!(
                            "subscribing to {:?}, error: {}",
                            PathBuf::from(&mod_.path).file_name().unwrap(),
                            x
                        ));
                        delay = 2 * delay + 1;
                    }
                };
                thread::sleep(Duration::new(delay, 0));
            }
        }
        pb.finish_and_clear();
        let mut file = File::create("modio_subscribed_mods")?;
        file.write_all(subscribed_mods.as_bytes())?;
    }
    if opt.update_all {
        // println!("this option doesn't do anything yet");
        println!("updating all installed mods...");
        let pb = ProgressBar::new(installed_mods.len() as u64);
        pb.set_style(ProgressStyle::with_template(TEMPLATE).unwrap());
        'baseloop: for mod_ in &installed_mods {
            pb.inc(1);
            pb.set_message(format!(
                "Updating {}",
                mod_.manifest.clone().objects.pallet.palletBarcode
            ));
            let target = match mod_.manifest.clone().objects.mod_target {
                Some(x) => x,
                None => {
                    continue;
                }
            };
            let installed_version: i64 = mod_.manifest.clone().objects.pallet.updateDate.parse()?;
            let modref = modio.mod_(Id::new(BONELAB), Id::new(target.modId));
            let mut online_mod = None;
            let mut delay = 0;
            while online_mod.is_none() {
                match modref.clone().get().await {
                    Ok(x) => online_mod = Some(x),
                    Err(x) => {
                        println!("Error: {}", x);
                        if !x.is_ratelimited() {
                            println!(
                                "skipped {}",
                                mod_.manifest.clone().objects.pallet.palletBarcode
                            );
                            continue 'baseloop;
                        }
                        delay = 2 * delay + 1;
                    }
                }
                thread::sleep(Duration::new(delay, 0));
            }

            let online_mod = online_mod.unwrap();
            let online_version = online_mod.date_updated.as_secs() * 1000;

            let files = modref.files();
            let filter = fid::asc();
            let files = files.search(filter).collect().await?;
            let files: Vec<&modio::files::File> = files
                .iter()
                .filter(|file| file.platforms[0].target == TargetPlatform::WINDOWS)
                .collect();

            if online_version <= installed_version {
                continue;
            }
            let mut new_manifest = mod_.manifest.clone();
            new_manifest.objects.pallet.updateDate = online_version.to_string();
            download_mod(
                &online_mod,
                &modio,
                path.clone(),
                PathBuf::from(path.clone()),
                Some(new_manifest),
                files.last().map(|v| &**v), // the highest file id is the latest modfile
                Some(mod_.manifest.objects.pallet.installedDate.parse().unwrap())
            )
            .await?;
        }
    }

    if opt.install_all_subscribed {
        println!("installing all new subscribed mods");
        let filter = GameId::_in(BONELAB).and(Name::asc());
        let query = modio.user().subscriptions(filter).collect().await?;
        let pb = ProgressBar::new(installed_mods.len() as u64);
        pb.set_style(ProgressStyle::with_template(TEMPLATE).unwrap());
        for mod_ in query.iter() {
            pb.inc(1);
            pb.set_message(format!("{}", mod_.name));
            let mut found = false;
            'inner: for i_mod_ in installed_mods.iter() {
                let target = match i_mod_.manifest.clone().objects.mod_target {
                    Some(x) => x,
                    None => continue 'inner,
                };
                if mod_.id == target.modId {
                    found = true;
                    break 'inner;
                }
            }
            if found {
                // println!("mod is installed");
            } else {
                // println!("{}", mod_.name);
                download_mod(mod_, &modio, path.clone(), PathBuf::from(path.clone()), None, None, None).await?;
            }
        }
        pb.finish();
    }
    Ok(())
}

async fn download_mod(
    mod_: &Mod,
    modio: &Modio,
    path: String,
    mod_folder: PathBuf,
    manifest: Option<Manifest>,
    modfile: Option<&modio::files::File>,
    installed_date: Option<u128>,
) -> Result<(), Box<dyn std::error::Error>> {
    let modfile = match modfile {
        Some(x) => x,
        None => {
            let modfile = &mod_.modfile;
            let modfile = match modfile {
                Some(x) => x,
                None => {
                    println!("no modfile");
                    return Ok(());
                }
            };
            modfile
        }
    };
    let xdg_cache_home = env::var("XDG_CACHE_HOME").unwrap_or_else(|_| {
        // Default to ~/.cache if XDG_CACHE_HOME is not set
        let home = env::var("HOME").unwrap();
        format!("{}/.cache", home)
    });

    let action = DownloadAction::File {
        game_id: Id::new(BONELAB),
        mod_id: Id::new(mod_.id.into()),
        file_id: Id::new(modfile.id.into()),
    };
    let _output = Command::new("mkdir")
        .arg(xdg_cache_home.clone() + "/bonelab-mod-manager")
        .output() // Execute the command
        .expect("Failed to execute command: mkdir");
    modio
        .download(action)
        .await?
        .save_to_file(format!("{}/bonelab-mod-manager/{}.zip", &xdg_cache_home, mod_.name.clone()))
        .await?;

    // shell commands to unzip and then figure out the barcode and pallet name and catalog name
    let _output = Command::new("unzip")
        .args([
            "".to_string() + &xdg_cache_home + "/bonelab-mod-manager/" + &mod_.name.clone() + "",
            "-d".into(),
            path.clone() + "/" + &mod_.name,
        ])
        .output() // Execute the command
        .expect("Failed to execute command: unzip");
    let barcode = Command::new("ls")
        .arg(path.clone() + "/" + &mod_.name)
        .output()?
        .stdout;
    let barcode = String::from_utf8(barcode)?;
    let zip_path = path.clone() + "/" + &mod_.name + "/" + &barcode;
    let zip_contents = Command::new("ls")
        .arg("".to_owned() + &zip_path.trim() + "")
        .output()?;
    let zip_contents = zip_contents.stdout;
    let zip_contents = String::from_utf8(zip_contents)?;
    let mut jsons = Vec::new();
    for line in zip_contents.lines() {
        if line.ends_with(".json") {
            jsons.push(line);
        }
    }
    assert_eq!(jsons.len(), 2);
    if !jsons[0].ends_with("pallet.json") {
        jsons.reverse();
    }

    let mani = match manifest {
        Some(x) => x,
        None => make_manifest(mod_, modfile, &barcode, jsons[0].trim(), jsons[1].trim(), installed_date),
    };
    let mani_str = serde_json::to_string_pretty(&mani)?;
    let mut save_path = mod_folder;
    save_path.push(mani.clone().objects.pallet.palletBarcode + ".manifest");
    let mut file = File::create(save_path)?;
    file.write_all(mani_str.as_bytes())?;

    let _output = Command::new("mv")
        .arg(path.clone() + "/" + &mod_.name + "/" + &barcode.trim())
        .arg(path.clone() + "/")
        .output()?;
    let _output = Command::new("rmdir")
        .arg(path.clone() + "/" + &mod_.name)
        .output()?;
    Ok(())
}

fn make_manifest(
    mod_: &Mod,
    modfile: &modio::files::File,
    barcode: &str,
    pallet_name: &str,
    catalog_name: &str,
    installed_date: Option<u128>,
) -> Manifest {
    let barcode = barcode.trim();
    let pallet_name = pallet_name.trim();
    let catalog_name = catalog_name.trim();
    let time_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let installed_date = match installed_date {
        Some(x) => {x},
        None => {time_now},
    };
    let mut targets = HashMap::new();
    targets.insert(
        "pc".to_string(),
        Reference {
            reference: "3".into(),
            type_: "mod-target-modio#0".into(),
        },
    );
    let manifest = Manifest {
        version: 2,
        root: Root {
            reference: "1".into(),
            type_: "pallet-manifest#0".into(),
        },
        objects: Object {
            pallet: Pallet {
                palletBarcode: barcode.into(),
                palletPath: format!(
                    "C:/users/steamuser/AppData/LocalLow/Stress Level Zero/BONELAB/Mods/{}/{}",
                    barcode, pallet_name
                ),
                catalogPath: format!(
                    "C:/users/steamuser/AppData/LocalLow/Stress Level Zero/BONELAB/Mods/{}/{}",
                    barcode, catalog_name
                ),
                version: modfile.version.clone(),
                installedDate: installed_date.to_string(),
                updateDate: time_now.to_string(),
                modListing: Some(Reference {
                    reference: "2".into(),
                    type_: "mod-listing#0".into(),
                }),
                active: true,
                isa: Isa {
                    type_: "pallet-manifest#0".into(),
                },
            },
            mod_listing: Some(ModListing {
                barcode: barcode.into(),
                title: Some(mod_.name.clone()),
                description: mod_.description_plaintext.clone(),
                author: Some(mod_.submitted_by.username.clone()),
                version: modfile.version.clone(),
                thumbnailUrl: Some(mod_.logo.thumb_320x180.to_string()),
                targets: targets,
                isa: Isa {
                    type_: "mod-listing#0".into(),
                },
            }),
            mod_target: Some(ModTarget {
                thumbnailOverride: None,
                gameId: mod_.game_id.into(),
                modId: mod_.id.into(),
                modfileId: modfile.id.into(),
                isa: Isa {
                    type_: "mod-target-modio#0".into(),
                },
            }),
        },
    };
    return manifest;
}

fn prompt(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer.trim().to_string())
}
