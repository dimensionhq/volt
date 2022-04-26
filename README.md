<p align="center">
  <img src="https://github.com/dimensionhq/volt/blob/master/assets/volt-transparent-bg.png?raw=true" />
</p>

<h1 align="center">Volt</h1>
<h4 align="center">Rapid, reliable and robust Javascript package management.</h4>
<br>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.0.1--alpha-c6b5ff"> <img src="https://img.shields.io/github/license/dimensionhq/volt?color=75ff73"> <img src="https://img.shields.io/tokei/lines/github/dimensionhq/volt?color=%23ffb5f5"> <img src="https://img.shields.io/github/languages/top/dimensionhq/volt?color=b5f0ff"> <img src="https://img.shields.io/github/languages/code-size/dimensionhq/volt?color=%235e6cff&label=size">
</p>
<br>

<img src="https://user-images.githubusercontent.com/63039748/122814035-b9696280-d2e4-11eb-8157-67a49f03190d.png">

WARNING: Volt is still in the development stage and is not ready for use!

**Rapid**: Volt is incredibly fast and powerful.

**Reliable**: Volt is built to be reliable and dependable.

**Robust**: Volt works with low resource usage.

**Important**: Volt is still in the alpha stage of development, and is not ready for use in production or development environments.
<br>

# :zap: Installation

We don't have an official release of Volt yet, however, if you would like to give it a try, feel free to follow the steps below to build from source.
<br>

## Build From Source

Prerequisites: **Git**, **Rust Toolchain**

### Minimum Supported Rust Version (MSRV)

Rust 1.58

### Steps

1. Clone the github repository using the Github CLI.

```powershell
git clone https://github.com/dimensionhq/volt
```

2. Change to the `volt` directory.

```powershell
cd volt
```

3. Run a compiled and optimized build

```
cargo run --release -- --help
# you should see a help menu from Volt
```

<br>

## :test_tube: Testing

First, make sure you [**Build From Source**](https://github.com/dimensionhq/volt/#build-from-source).

Run this command to run the tests for volt.

```powershell
cargo test
```

<br>

## :clap: Supporters

[![Stargazers repo roster for @dimensionhq/volt](https://reporoster.com/stars/dimensionhq/volt)](https://github.com/dimensionhq/volt/stargazers)

[![Forkers repo roster for @dimensionhq/volt](https://reporoster.com/forks/dimensionhq/volt)](https://github.com/dimensionhq/volt/network/members)

<br>

## :hammer: Build Status

| Feature  | Build Status |
| -------- | ------------ |
| Add      | 🏗️           |
| Audit    | ❌           |
| Cache    | ❌           |
| Check    | ❌           |
| Clone    | 🏗️           |
| Compress | 🏗️           |
| Create   | 🏗️           |
| Deploy   | 🏗️           |
| Fix      | ❌           |
| Help     | 🏗️           |
| Info     | ❌           |
| Init     | 🏗️           |
| Install  | 🏗️           |
| List     | 🏗️           |
| Login    | 🏗️           |
| Logout   | ❌           |
| Migrate  | 🏗️           |
| Mod      | ❌           |
| Outdated | ❌           |
| Owner    | ❌           |
| Ping     | 🏗️           |
| Publish  | ❌           |
| Remove   | ❌           |
| Run      | 🏗️           |
| Search   | ❌           |
| Set      | ❌           |
| Stat     | ❌           |
| Tag      | ❌           |
| Team     | ❌           |
| Update   | ❌           |
| Watch    | 🏗️           |

<br>

## Built With

[Rust](https://www.rust-lang.org/)

[External Libraries](https://github.com/dimensionhq/volt/blob/dev/CREDITS.md)

## Versioning

We use [semver](https://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/dimensionhq/volt/tags).

## License

This project is licensed under Apache-2.0 - see the [LICENSE.md](LICENSE) file for details.
