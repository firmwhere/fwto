/*++ @file

  Copyright ©2021 Liu Yi, liuyi28@lenovo.com

  This program is just made available under the terms and conditions of the
  MIT license: http://www.efikarl.com/mit-license.html

  THE PROGRAM IS DISTRIBUTED UNDER THE MIT LICENSE ON AN "AS IS" BASIS,
  WITHOUT WARRANTIES OR REPRESENTATIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED.
--*/

use structopt::StructOpt;

pub mod libs;
pub mod audk;
pub mod subcmd_ovrd;
pub mod subcmd_cbup;
pub mod subcmd_view;
pub mod subcmd_build;

#[derive(StructOpt, Debug)]
/// AptioV Codebase Upgrade Toolkit @liuyi28@lenovo.com
struct Opts {
    #[structopt(subcommand)]
    cmd             : Option<Command>,
    #[structopt(flatten)]
    project         : audk::Project,
    #[structopt(flatten)]
    oemovrd         : audk::StdOvrd,
    /// Current audk configuration file
    #[structopt(short, long)]
    audk_json       : Option<String>,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Override a file of AptioV codebase
    Ovrd {
        #[structopt(flatten)]
        ovrd        : subcmd_ovrd::Ovrd,
    },
    /// Extract diffs for codebase upgrade
    Cbup {
        #[structopt(flatten)]
        diff        : subcmd_cbup::Cbup,
    },
    /// Extract diffs for ws's two commits
    View {
        #[structopt(flatten)]
        diff        : subcmd_view::View,
    },
    /// Build the project code in anywhere
    Build {
        #[structopt(flatten)]
        build       : audk::AptioV,
        /// When --no-clean, build without clean
        #[structopt(long)]
        no_clean    : bool,
    },
}

fn main() {
    let opt = Opts::from_args();

    let audk_option = audk::Json::get(&opt.audk_json);
    if opt.cmd.is_none() {
        return
    }

    let mut workspace   = std::path::PathBuf::new();
    let mut cfg_aptio_v = None;
    let mut cfg_oemovrd = None;
    let mut cfg_ibvovrd = None;
    if let Some(audk) = audk_option {
        workspace   = audk.project.workspace.unwrap();
        cfg_aptio_v = audk.aptio_v;
        cfg_oemovrd = audk.oemovrd;
        cfg_ibvovrd = audk.ibvovrd;
    }

    if let Some(ws) = opt.project.workspace {
        workspace = ws;
    }
    let wp = workspace.parent().unwrap();
    let ws =&workspace;
    if !ws.join("MdePkg").is_dir() && ws.is_dir() {
        std::env::set_current_dir(ws).unwrap();
        setup_workspace(&cfg_aptio_v.as_ref());
    }
    if !ws.join("MdePkg").is_dir() && wp.is_dir() {
        std::env::set_current_dir(wp).unwrap();
        setup_workspace(&cfg_aptio_v.as_ref()); 
    }
    if !ws.join("MdePkg").is_dir() {
        println!("ERR: invalid workspace {:?}", ws);
        return
    }
    std::env::set_current_dir(ws).unwrap();

    match opt.cmd.as_ref().unwrap() {
        Command::Ovrd{ovrd} => {
            ovrd.handler(&opt.oemovrd, &cfg_oemovrd.as_ref(), &cfg_ibvovrd.as_ref());
        },
        Command::Cbup{diff} => {
            diff.handler(&opt.oemovrd, &cfg_oemovrd.as_ref(), &cfg_ibvovrd.as_ref());
        },
        Command::View{diff} => {
            diff.handler(&opt.oemovrd, &cfg_oemovrd.as_ref(), &cfg_ibvovrd.as_ref());
        },
        Command::Build{build, no_clean} => {
            build.handler(&cfg_aptio_v.as_ref(), *no_clean);
        },
    }
}

fn setup_workspace(cfg_aptio_v: &Option<&crate::audk::AptioV>) {
    //
    // scripts of build hooks
    //
    let scripts = if let Some(cfg_aptio_v) = cfg_aptio_v {
        if let Some(scripts) = cfg_aptio_v.scripts.as_ref() {
            Some(scripts)
        } else {
            None
        }
    } else {
        None
    };
    //
    // setup workspace hook
    //
    if let Some(scripts) = scripts {
        if let Some(work_space) = scripts.work_space.as_ref() {
            for i in work_space {
                let null_args = Vec::<String>::new();
                let null_farg =       String ::new();
                let args = if let Some(args) = i.args.as_ref() { args } else { &null_args };
                let farg = if let Some(farg) = i.farg.as_ref() { farg } else { &null_farg };
                if i.file.is_file() {
                    std::process::Command::new(&i.interpreter).args(args).arg(farg).arg(&i.file).status().unwrap();
                }
            }
        }
    }
}
