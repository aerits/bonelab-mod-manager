use std::{
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    thread,
    time::Duration,
};

use indicatif::{ProgressBar, ProgressStyle};
use modio::filter::prelude::*;
use modio::{Credentials, DownloadAction, Modio, Result, auth::Token, types::id::Id};
use modio::{files::filters::Id as fid, mods::Mod, types::id::GameId};
use reqwest::Client;
use serde_json::{Value, json};
use structopt::StructOpt;

const BONELAB: u64 = 3809;

struct InstalledMod {
    path: String,
    mod_id: u64,
    file_id: u64,
}

/// Bonelab mod manager
#[derive(structopt::StructOpt)]
struct Opt {
    /// email to log into mod.io
    #[structopt(short, long)]
    email: Option<String>,
    /// your mod.io api key
    api_key: String,
    /// folder where bonelab mods are,
    /// usually something like /C:/users/steamuser/AppData/LocalLow/Stress Level Zero/BONELAB/Mods/
    mod_folder: PathBuf,
    /// subscribe to all mods
    #[structopt(short, long, name = "subscribe to all mods")]
    subscribe_all: bool,
    /// update all mods / does not do anything rn
    #[structopt(short, long, name = "update all mods")]
    update_all: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let path = opt.mod_folder
        .clone()
        .as_mut_os_str()
        .to_string_lossy()
        .into_owned();

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
        .filter(|x| x.ends_with(".manifest"))
        .map(|x| x.clone())
        .collect();

    let mut installed_mods = Vec::new();

    println!("Reading mods...");
    let pb = ProgressBar::new(mod_manifests.len() as u64);
    for manifest in mod_manifests {
        // println!("{}", manifest);
        let path = path.clone() + &manifest;
        let manifest = fs::read_to_string(&path)?;
        // println!("{}", manifest);
        let deserialized: Value = serde_json::from_str(&manifest)?;
        let mod_id: u64 =
            match serde_json::from_value(deserialized["objects"]["3"]["modId"].clone()) {
                Ok(x) => x,
                Err(_) => {
                    continue;
                } // mod not downloaded from modio
            };
        let file_id: u64 =
            serde_json::from_value(deserialized["objects"]["3"]["modfileId"].clone())?;
        let mod_ = InstalledMod {
            path,
            mod_id,
            file_id,
        };
        installed_mods.push(mod_);
        pb.inc(1);
    }
    pb.finish_and_clear();

    let mut modio = Modio::new(Credentials::new(opt.api_key))?;

    let access_token = fs::read_to_string("modio_access_token");
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
            let mut file = File::create("modio_access_token")?;
            file.write_all(token.value.as_bytes())?;
            modio = modio.with_token(token.clone());
        } else {
            panic!("could not login");
        }
    }
    let user = modio.user().current().await?;
    println!("logged in as {}", user.unwrap().username);

    if opt.subscribe_all {
        let mut subscribed_mods = match fs::read_to_string("modio_subscribed_mods") {
            Ok(x) => {x},
            Err(_) => {String::new()},
        };
        let installed_mods: Vec<&InstalledMod> = installed_mods.iter().filter(|x| {
            !subscribed_mods.contains(&x.path)
        }).collect();
        println!("subscribing to all installed mods...");
        let pb = ProgressBar::new(installed_mods.len() as u64);
        pb.set_style(ProgressStyle::with_template("[{bar}][time: {elapsed_precise}][eta: {eta_precise}] {msg}").unwrap());
        for mod_ in installed_mods {
            pb.inc(1);
            pb.set_message(format!("subscribing to {:?}", PathBuf::from(&mod_.path).file_name().unwrap()));
            let modref = modio.mod_(Id::new(BONELAB), Id::new(mod_.mod_id));
            let mut subscribed = false;
            let mut delay = 1;
            while !subscribed {
                match modref.clone().subscribe().await {
                Ok(_) => {
                    subscribed = true;
                    subscribed_mods += &(mod_.path.clone() + "\n");
                },
                Err(x) => {
                    pb.set_message(format!("subscribing to {:?}, error: {}", PathBuf::from(&mod_.path).file_name().unwrap(), x));
                    delay *= 2;
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
        println!("this option doesn't do anything yet");
    }

    Ok(())
}

async fn download_mod(modio: Modio) -> Result<(), Box<dyn std::error::Error>> {
    let modref = modio.mod_(Id::new(BONELAB), Id::new(4061166));
    modref.clone().subscribe().await?;
    let mod_ = modref.clone().get().await?;
    println!("{:?}", mod_.name);

    let filter = fid::asc();

    let file = modref
        .clone()
        .files()
        .search(filter)
        .first()
        .await
        .unwrap()
        .unwrap()
        .id;

    let action = DownloadAction::File {
        game_id: Id::new(3809),
        mod_id: Id::new(4061166),
        file_id: file,
    };
    modio
        .download(action)
        .await?
        .save_to_file(mod_.name + ".zip")
        .await?;
    Ok(())
}

fn prompt(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer.trim().to_string())
}
