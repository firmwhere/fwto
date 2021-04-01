/*++ @file

  Copyright ©2021 Liu Yi, liuyi28@lenovo.com

  This program is just made available under the terms and conditions of the
  MIT license: http://www.efikarl.com/mit-license.html

  THE PROGRAM IS DISTRIBUTED UNDER THE MIT LICENSE ON AN "AS IS" BASIS,
  WITHOUT WARRANTIES OR REPRESENTATIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED.
--*/

use std::fs;
use path_slash::PathBufExt;

pub fn show_no_format(commit: &String, name_only: bool, find_renames: &str, diff_filter: &str, show_dst: &Vec<&Option<&std::path::PathBuf>>, exclude_show_dst: bool) -> std::process::Output {
    let cmd: (&str, &str) = if cfg!(target_os = "windows") { ("cmd", "/c") } else { ("sh", "-c") };
    let show_name = if name_only { "--name-only" } else { "--name-status" };
    let mut show_files_or_not = false;
    for path in show_dst {
        if let Some(path) = path {
            if path.is_dir() {
                show_files_or_not = true;
            }
        }
    };
    let mut show_files = if show_files_or_not {
        String::new() + " " + "--" + " "
    } else {
        String::new()
    };
    if show_files_or_not {
        for path in show_dst {
            if let Some(path) = path {
                if path.is_dir() {
                    show_files = show_files + if exclude_show_dst { ":!:" } else { "" } + &path.to_slash().unwrap() + " ";
                }
            }
        };
    }
    let gitcmd = String::from(r#"git show --format="#) + " " + show_name + " " + "--find-renames=" + find_renames + " " + "--diff-filter=" + diff_filter + " " + commit + &show_files;
    std::process::Command::new(cmd.0).arg(cmd.1).arg(gitcmd).output().unwrap()
}

pub fn create_file_from(commit: &String, fsrc: &std::path::PathBuf, fdst: &std::path::PathBuf, rcommit: Option<&str>) {
    let cmd: (&str, &str) = if cfg!(target_os = "windows") { ("cmd", "/c") } else { ("sh", "-c") };
    let output = std::process::Command::new(cmd.0).arg(cmd.1).arg("git show").arg(String::from(commit) + if let Some(r) = rcommit { r } else { "" } + ":" + &fsrc.to_slash().unwrap()).output().unwrap();
    if output.status.success() {
        let fdst_parent = fdst.parent().unwrap();
        if !fdst_parent.is_dir() {
            fs::create_dir_all(&fdst_parent).unwrap();
        }
        fs::write(&fdst, output.stdout).unwrap();
    } else {
        println!("create_file_from_git.0: fsrc: {:?}", fsrc);
    }
}

pub fn revert_no_commit(commit: &String) {
    let cmd: (&str, &str) = if cfg!(target_os = "windows") { ("cmd", "/c") } else { ("sh", "-c") };
    let output = std::process::Command::new(cmd.0).arg(cmd.1).arg("git revert --no-commit").arg(commit).output().unwrap();
    if !output.status.success() {
        println!("revert_with_no_commit.1: {:#?}", output);
    }
}

pub fn reset_hard_and_clean_xfd(commit: &String) {
    let cmd: (&str, &str) = if cfg!(target_os = "windows") { ("cmd", "/c") } else { ("sh", "-c") };
    let output = std::process::Command::new(cmd.0).arg(cmd.1).arg("git reset --hard").arg(commit).output().unwrap();
    if !output.status.success() {
        println!("reset_hard_and_clean_xfd.1: {:#?}", output);
    }
    let output = std::process::Command::new(cmd.0).arg(cmd.1).arg("git clean   -xfd").output().unwrap();
    if !output.status.success() {
        println!("reset_hard_and_clean_xfd.2: {:#?}", output);
    }
}
