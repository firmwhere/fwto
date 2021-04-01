/*++ @file

  Copyright ©2021 Liu Yi, liuyi28@lenovo.com

  This program is just made available under the terms and conditions of the
  MIT license: http://www.efikarl.com/mit-license.html

  THE PROGRAM IS DISTRIBUTED UNDER THE MIT LICENSE ON AN "AS IS" BASIS,
  WITHOUT WARRANTIES OR REPRESENTATIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED.
--*/

use std::fs;

fn force_rw<P: AsRef<std::path::Path>>(path: P) -> std::result::Result<(), std::io::Error> {
  if path.as_ref().is_file() {
    let mut perms = fs::metadata(&path)?.permissions();
    if perms.readonly() {
      perms.set_readonly(false); return fs::set_permissions(&path, perms);
    }
  }

  Ok(())
}

pub fn remove_file<P: AsRef<std::path::Path>>(path: P) -> std::result::Result<(), std::io::Error> {
  force_rw(&path)?;

  fs::remove_file(&path)
}

pub fn copy<P: AsRef<std::path::Path>, Q: AsRef<std::path::Path>>(from: P, to: Q) -> std::result::Result<u64, std::io::Error> {
  force_rw(&to)?;

  fs::copy(&from, &to)
}
