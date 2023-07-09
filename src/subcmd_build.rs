/*++ @file

  Copyright ©2021 Liu Yi, liuyi28@lenovo.com

  This program is just made available under the terms and conditions of the
  MIT license: http://www.efikarl.com/mit-license.html

  THE PROGRAM IS DISTRIBUTED UNDER THE MIT LICENSE ON AN "AS IS" BASIS,
  WITHOUT WARRANTIES OR REPRESENTATIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED.
--*/

use crate::audk::AptioV as Build;
use crate::audk::AptioProject;
use crate::audk::AptioToolkit;

impl Build {
    pub fn new() -> Self {
        Self {
            project: AptioProject { veb: None }, toolkit: AptioToolkit { ewdk: None, tools: None, pycmd: None }, scripts: None
        }
    }
    pub fn handler(&self, cfg_aptio_v: &Option<&Build>, no_clean: bool) {
        let cmd: (&str, &str) = if cfg!(target_os = "windows") { ("powershell", "-command") } else { ("sh", "-c") };
        // scripts of build hooks
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
        // fore_build hooks
        //
        if let Some(scripts) = scripts {
            if let Some(fore_build) = scripts.fore_build.as_ref() {
                for i in fore_build {
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

        let veb = if let Some(veb) = &self.project.veb {
            veb
        } else {
            cfg_aptio_v.expect("ERR: aptio_v is neither given in cmdline or json").project.veb.as_ref().expect("ERR: veb is None in json.")
        };
        if !veb.is_file() {
            println!("ERR: invalid project veb: {:?}", veb);
            return
        }

        let ewdk = if let Some(ewdk) = &self.toolkit.ewdk {
            ewdk
        } else {
            cfg_aptio_v.expect("ERR: aptio_v is neither given in cmdline or json").toolkit.ewdk.as_ref().expect("ERR: ewdk is None in json.")
        };
        if !ewdk.join("LaunchBuildEnv.cmd").is_file() {
            println!("ERR: invalid ewdk {:?}", ewdk);
            return
        }
        let tools = if let Some(tools) = &self.toolkit.tools {
            tools
        } else {
            cfg_aptio_v.expect("ERR: aptio_v is neither given in cmdline or json").toolkit.tools.as_ref().expect("ERR: tools is None in json.")
        };
        if !tools.join("Bin").is_dir() || !tools.join("make.exe").is_file() {
            println!("ERR: invalid tools {:?}", tools);
            return
        }
        let pycmd = if let Some(pycmd) = &self.toolkit.pycmd {
            pycmd
        } else {
            cfg_aptio_v.expect("ERR: aptio_v is neither given in cmdline or json").toolkit.pycmd.as_ref().expect("ERR: pycmd is None in json.")
        };
        if (!pycmd.is_file()) || (pycmd.file_name().unwrap() != "python.exe") {
            println!("ERR: invalid pycmd {:?}", pycmd);
            return
        }

        std::env::set_var(            "VEB" , veb.file_stem().unwrap()  );
        std::env::set_var(       "EWDK_DIR" , &ewdk                     );
        std::env::set_var(      "TOOLS_DIR" , &tools                    );
        std::env::set_var( "PYTHON_COMMAND" , &pycmd                    );
        let pyth = String::new() + pycmd.parent().unwrap().to_str().unwrap() + ";" + pycmd.parent().unwrap().join("Scripts").to_str().unwrap() + ";";
        let path = if let Some(path) = std::env::var_os("PATH") {
            String::new() + &pyth + tools.to_str().unwrap() + ";" + path.to_str().unwrap()
        } else {
            String::new() + &pyth + tools.to_str().unwrap()
        };
        std::env::set_var(           "PATH" , path                      );

        let make_opts = if no_clean {
            String::from("all")
        } else {
            String::from("rebuild")
        };
        std::process::Command::new(cmd.0).arg(cmd.1).arg("make").arg(make_opts).status().unwrap();
        //
        // post_build hooks
        //
        if let Some(scripts) = scripts {
            if let Some(post_build) = scripts.post_build.as_ref() {
                for i in post_build {
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
}
