/*++ @file

  Copyright ©2021 Liu Yi, liuyi28@lenovo.com

  This program is just made available under the terms and conditions of the
  MIT license: http://www.efikarl.com/mit-license.html

  THE PROGRAM IS DISTRIBUTED UNDER THE MIT LICENSE ON AN "AS IS" BASIS,
  WITHOUT WARRANTIES OR REPRESENTATIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED.
--*/

use std::fs;
use std::io::prelude::*;
use structopt::StructOpt;
use serde;

pub const FWTO_WS: &str = "0.fwto";

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Json {
    pub project         : Project,
    pub ibvovrd         : Option<StdOvrd>,
    pub oemovrd         : Option<StdOvrd>,
    pub aptio_v         : Option<AptioV>,
}

impl Json {
    pub fn get(conf: &Option<String>) -> Option<Json> {
        let mut audk_json: Option<Json> = None;

        let pcfg = std::env::current_exe().unwrap().parent().unwrap().join(".fwto");
        if !pcfg.is_dir() {
            fs::create_dir_all(&pcfg).unwrap();
        }
        let fsel = pcfg.join(".audk.default");
        let conf_from_default = if fsel.is_file() {
            String::from_utf8(fs::read(&fsel).unwrap()).unwrap()
        } else {
            String::from("default.json")
        };

        let conf = if let Some(conf) = conf {
            conf
        } else {
            &conf_from_default
        };
        println!("---------------------------");
        println!("INF: current configuration: {:?}", conf);
        println!("---------------------------");

        let fcfg = pcfg.join(conf);
        if fcfg.is_file() {
            audk_json = Some(serde_json::from_reader(
                fs::OpenOptions::new().read(true).open(fcfg).unwrap()
            ).expect(&format!("ERR: invalid format of {}", conf)));
            // once success, update default audk default configuration to .audk.default
            let mut fsel = fs::OpenOptions::new().create(true).write(true).truncate(true).open(&fsel).unwrap();
            fsel.write(conf.as_bytes()).unwrap();
        } else {
            println!("WRN: no audk configuration: {:?}", conf);
        }
        audk_json
    }
}

#[derive(Debug, Clone, StructOpt, serde::Deserialize)]
pub struct Project {
    /// Workspace of UEFI Development Kit
    #[structopt(short, long, parse(from_os_str))]
    pub workspace       : Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, StructOpt, serde::Deserialize)]
pub struct StdOvrd {
    /// Project cif file
    #[structopt(short, long, parse(from_os_str))]
    pub cif             : Option<std::path::PathBuf>,

    /// Destination where overrides are in
    #[structopt(short, long, parse(from_os_str))]
    pub dst             : Option<std::path::PathBuf>,

    /// Destination where originals are in
    #[structopt(short, long, parse(from_os_str))]
    pub org             : Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, StructOpt, serde::Deserialize)]
pub struct AptioV {
    #[structopt(flatten)]
    pub project         : AptioProject,
    #[structopt(flatten)]
    pub toolkit         : AptioToolkit,
    #[structopt(skip)]
    pub scripts         : Option<Scripts>,
}

#[derive(Debug, Clone, StructOpt, serde::Deserialize)]
pub struct AptioProject {
    /// Visual eBios of AMI project
    #[structopt(short, long, parse(from_os_str))]
    pub veb             : Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, StructOpt, serde::Deserialize)]
pub struct AptioToolkit {
    /// Path to Enterprise WDK
    #[structopt(short, long, parse(from_os_str))]
    pub ewdk            : Option<std::path::PathBuf>,
    /// Path to BuildTools of AptioV
    #[structopt(short, long, parse(from_os_str))]
    pub tools           : Option<std::path::PathBuf>,
    /// Path of PYTHON_COMMAND
    #[structopt(short, long, parse(from_os_str))]
    pub pycmd           : Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Scripts {
    pub fore_build      : Option<Vec<ScriptsDesc>>,
    pub post_build      : Option<Vec<ScriptsDesc>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ScriptsDesc {
    pub interpreter     : std::path::PathBuf,
    pub opts            : Option<String>,
    pub file            : std::path::PathBuf,
}
