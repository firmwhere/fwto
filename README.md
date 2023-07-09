## fwto.exe

```powershell
fwto 0.5.5
AptioV Codebase Upgrade Toolkit @liuyi28@lenovo.com

USAGE:
    fwto.exe [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --audk-json <audk-json>    Current audk configuration file
    -c, --cif <cif>                Project cif file
    -d, --dst <dst>                Destination where overrides are in
    -o, --org <org>                Destination where originals are in
    -w, --workspace <workspace>    Workspace of UEFI Development Kit

SUBCOMMANDS:
    build    Build the project code in anywhere
    cbup     Extract diffs for codebase upgrade
    help     Prints this message or the help of the given subcommand(s)
    ovrd     Override a file of AptioV codebase
    view     Extract diffs for code two commits
```

### Json configuration support

- With json configuration, arguments of `fwto.exe` can be left out. If there is, argument will override json configuration.
- Put it under the sub-folder (`.fwto`) where `fwto.exe` is.

```json
{
    "project": {
        "workspace": "The absolute path to EDKII workspace"
    },
    "ibvovrd": {
        "cif": "The relative path to IBV's *.cif",
        "dst": "The relative path to IBV's OVERRIDE"
    },
    "oemovrd": {
        "cif": "The relative path to <project>.cif file",
        "dst": "The relative path to <project> OVERRIDE",
        "org": "The relative path to <project> Original"
    },
    "aptio_v": {
        "project": {
            "veb"   : "<project>.veb"
        },
        "toolkit": {
            "ewdk"  : "The absolute path to EWDK, in where LaunchBuildEnv.cmd is",
            "tools" : "The absolute path to BuildTools of Aptio_x.x_TOOLS_xx",
            "pycmd" : "The absolute path to python.exe file"
        }
    }
}
```

- `ibvovrd` is optional, please remove `ibvovrd` if it is not in use.

```powershell
❯ # Suppose current dir is in where fwto.exe is,
❯ #   and there is "default.json", the path should be: ".fwto\default.json" to make it work
❯ 
❯ # In fact, the default configuration is .fwto\default.json,
❯ #   if change it to another json under .fwto, run:
❯ fwto.exe -a <json_file_name>.json
```

## Usage: fwto.exe-ovrd

### Command help

```powershell
fwto.exe-ovrd 0.5.5
Override a file of AptioV codebase

USAGE:
    fwto.exe ovrd [FLAGS] --src <src>

FLAGS:
        --clean       Clean files from override
    -h, --help        Prints help information
        --skip-org    Skip original of override
    -V, --version     Prints version information

OPTIONS:
    -s, --src <src>    File to be override
```

### Command example

```powershell
❯ # Override a file example without json configuration support:
❯ # ` below is just linebreak in powershell please ignore it
❯ fwto.exe `
          -w E:\Wv2\code                                        `
          -c LenovoPlatformPkg\OverrideRC\OverrideRC.cif        `
          -d LenovoPlatformPkg\OverrideRC\OVERRIDE              `
          -o Original                                           `
          ovrd -s MdeModulePkg\Core\Dxe\DxeMain.inf
```

```powershell
❯ # Override a file example with json configuration support:
❯ fwto.exe ovrd -s MdeModulePkg\Core\Dxe\DxeMain.inf
❯ # Clean the override of a file:
❯ fwto.exe ovrd -s MdeModulePkg\Core\Dxe\DxeMain.inf --clean
```

## Usage: fwto.exe-cbup

### Command help

```powershell
fwto.exe-cbup 0.5.5
Extract diffs for codebase upgrade

USAGE:
    fwto.exe cbup [FLAGS] --commit <commit>

FLAGS:
    -h, --help       Prints help information
        --pure       Extract codebase-pure module or package diffs
    -V, --version    Prints version information

OPTIONS:
    -c, --commit <commit>    Commit to be extract
```

```ini
# fwto.exe cbup -c <commit>
[input]
a git commit with only codebase changes.

[output]
#.1 extract diff of that we also override ones
dir         = <workspace>/0.fwto/cbup/{R!|R75}
#.2 auto-update overrides, for example move the right file to orginal. So the reset work are just:
    # merge conflict of 1st step
    # paste merge result and replace file in project OVERRIDE

[dir]: 
base.old    : old source files of codebase
base.new    : new source files of codebase
ovrd        : source files we override, and to merge
```

### Command example

```powershell
❯ # This is example of codebase upgrade with aptio-v style override:
❯ 
❯ # step.1: create one commit with only codebase changes
❯ # step.2: run:
❯ fwto.exe cbup -c <commits>
❯ # step.2.1: merge conflict of step.2 by compare tool
❯ # step.2.2: paste merge result and replace file in project OVERRIDE
❯ # step.3: merge veb changes of codebase to <project>.veb
❯ # step.4: build and have a try, in most case, it should work well
```

## Usage: fwto.exe-view

### Command help

```powershell
fwto.exe-view 0.5.5
Extract diffs for ovrd-code review

USAGE:
    fwto.exe view [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -n, --new <new>    New commit for diff [default: HEAD]
    -o, --old <old>    Old commit for diff
```

```ini
# fwto.exe cbup -c <commit>
[input]
a git commit of project code.

[output]
dir         = <workspace>/0.fwto/view

[dir]: 
old         : old source files
new         : new source files
```

### Command example

```powershell
❯ fwto.exe cbup -c <commits>
```

## Usage: fwto.exe-build

### Command help

```powershell
fwto.exe-build 0.5.5
Build the project code in anywhere

USAGE:
    fwto.exe build [FLAGS] [OPTIONS]

FLAGS:
    -h, --help        Prints help information
        --no-clean    When --no-clean, build without clean
    -V, --version     Prints version information

OPTIONS:
    -e, --ewdk <ewdk>      Path to Enterprise WDK
    -p, --pycmd <pycmd>    Path of PYTHON_COMMAND
    -t, --tools <tools>    Path to BuildTools of AptioV
    -v, --veb <veb>        Visual eBios of AMI project
```

- Support build hooks, just by setup `scripts` into `aptio_v` in json configuration:

```json
    "aptio_v": {
        "scripts": {
            "work_space":  [
                {
                    "interpreter": "ruby",
                    "file": "script_relative_path_to_workspace-or-absolute_path.rb"
                }
            ],
            "fore_build": [
                {
                    "interpreter": "python",
                    "file": "script_relative_path_to_workspace-or-absolute_path.py"
                }
            ],
            "post_build": [
                {
                    "interpreter": "cmd",
                    "farg": "/c",
                    "file": "script_relative_path_to_workspace-or-absolute_path.bat"
                },
                {
                    "interpreter": "powershell",
                    "farg": "-file",
                    "file": "script_relative_path_to_workspace-or-absolute_path.ps1"
                }
            ]
        }
    }
```

### Command example

```ini
❯ fwto.exe build
```
