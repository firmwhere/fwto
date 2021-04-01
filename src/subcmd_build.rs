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
            project: AptioProject { veb: None }, toolkit: AptioToolkit { cc32: None, cc64: None, ewdk: None, tools: None, pycmd: None }
        }
    }
    pub fn handler(&self, cfg_aptio_v: &Option<&Build>, no_clean: bool) {
        let veb = if let Some(veb) = &self.project.veb {
            veb
        } else {
            cfg_aptio_v.expect("ERR: aptio_v is neither given in cmdline or json").project.veb.as_ref().expect("ERR: veb is None in json.")
        };
        if !veb.is_file() {
            println!("ERR: invalid project veb: {:?}", veb);
            return
        }

        let cc32 = if let Some(cc32) = &self.toolkit.cc32 {
            cc32
        } else {
            cfg_aptio_v.expect("ERR: aptio_v is neither given in cmdline or json").toolkit.cc32.as_ref().expect("ERR: cc32 is None in json.")
        };
        if !cc32.join("cl.exe").is_file() {
            println!("ERR: invalid cc32 {:?}", cc32);
            return
        }

        let cc64 = if let Some(cc64) = &self.toolkit.cc64 {
            cc64
        } else {
            cfg_aptio_v.expect("ERR: aptio_v is neither given in cmdline or json").toolkit.cc64.as_ref().expect("ERR: cc64 is None in json.")
        };
        if !cc64.join("cl.exe").is_file() {
            println!("ERR: invalid cc64 {:?}", cc64);
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

        std::env::set_var("CCX86DIR"        , &cc32);
        std::env::set_var("CCX64DIR"        , &cc64);
        std::env::set_var("EWDK_DIR"        , &ewdk);
        std::env::set_var("TOOLS_DIR"       , &tools);
        std::env::set_var("PYTHON_COMMAND"  , &pycmd);
        let path = if let Some(path) = std::env::var_os("PATH") {
            String::from(tools.to_str().unwrap()) + ";" + path.to_str().unwrap()
        } else {
            String::from(tools.to_str().unwrap())
        };
        std::env::set_var("PATH"            , path);
        std::env::set_var("VEB"             , veb.file_stem().unwrap());

        let make_opts = if no_clean {
            String::from("all")
        } else {
            String::from("rebuild")
        };

        let cmd: (&str, &str) = if cfg!(target_os = "windows") { ("cmd", "/c") } else { ("sh", "-c") };
        std::process::Command::new(cmd.0).arg(cmd.1).arg("make.exe").arg(make_opts).status().unwrap();
    }
}
