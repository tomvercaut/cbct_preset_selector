# CBCT preset selector

## About

A small commandline based application to determine the preset of a cone beam CT (CBCT) 
by asking the user to input these parameters:
 - treatment machine 
 - pathology

The application filters the results from a CSV file containing 3 columns:
 - treament machine
 - pathology
 - CBCT preset
 
## Installation
### Pre-build binary
If you have a prebuild executable for the system you are working on, copy the executable to a directory of your preference.
For example, on Windows you can create a directory: `C:\Program Files\cbct_preset_selector` and copy the executable there. 

Add the installation directory to the user or system environment PATH variable. 
Doing this will enable you to launch the application from your favorite terminal. 

### Build from source
To build the application from source you need to install some prerequisites:
 - [Git](https://git-scm.com): version control system
 - [Rust](https://www.rust-lang.org): compiler to build the Rust code. 
 More information on how to install Rust can be found in the online book [The Rust Programming Language](https://doc.rust-lang.org/book)

When all prerequisites have been installed simply run the following commands in your terminal to install the application:
```shell script
git clone https://github.com/tomvercaut/cbct_preset_selector
cargo install --path cbct_preset_selector 
```
Add the installation directory [`default: $HOME/.cargo/bin`] to the user or system environment PATH variable for easy use on the commandline.

## Command-line options
```
USAGE:
    cbct_preset_selector.exe [CSV file]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <CSV file>    CSV table with the CBCT presets per pathology and linac.
                  If the argument is ommited the application will look for the CSV file [data.csv] in the user local
                  data folder.
                  - Windows: %USERPROFILE%\AppData\Local\cbct_preset_selector
                  - Linux: $HOME/.local/share/cbct_preset_selector
                  - macOS: $HOME/Library/Application Support/cbct_preset_selector

```
This is the output of `cbct_preset_selector -h`.


 ## License
 Copyright (c) 2020 The cbct_preset_selector developer(s)
 
`cbct_preset_selector` is distributed under the terms of both the MIT License and the Apache Licence 2.0. 
See the [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) files for license details.
