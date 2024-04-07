<a name="readme-top"></a>

<br />
<div align="center">

  [![Discussions][discussions-shield]][discussions-url]
  [![Contributors][contributors-shield]][contributors-url]
  [![Forks][forks-shield]][forks-url]
  ![MIT + Apache-2.0 License][license-shield]

  <h2 align="center">fstools</h2>

  <p align="center">
    Tools for FROMSOFTWARE
    <br />
    <a href="https://github.com/soulsmods/fstools-rs/discussions/categories/bug-reports">Report Bug</a>
    ·
    <a href="https://github.com/soulsmods/fstools-rs/discussions/categories/ideas">Request Feature</a>
  </p>
</div>

- [About The Project](#about-the-project)
  - [Built With](#built-with)
- [Features](#features)
- [Getting Started](#getting-started)
  - [Dependencies](#dependencies)
  - [Building from Source](#building-from-source)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)
- [Acknowledgments](#acknowledgments)

<!-- ABOUT THE PROJECT -->
## About The Project

`fstools-rs` is a Rust-based toolkit for modding and analyzing FROMSOFTWARE games, supporting archive extraction, format conversion, and data analysis.

### Built With

* [Rust](https://rust-lang.org/)
* Many others...

<p align="right">(<a href="#readme-top">back to top</a>)</p>


## Features

- **Archive extraction**: Unpack DVDBNDs and BNDs from game data as fast as possible.
- **File analysis**: Offers tools for analysis of asset data structures, intended to aid in understanding underlying game mechanics and data relationships.


<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- GETTING STARTED -->
## Getting Started

### Dependencies

- Rust Programming Language: [Installation instructions](rust-install-instructions) are available at the official Rust language website.

### Building from Source

1. Clone the repository:
   ```shell
   git clone https://github.com/soulsmods/fstools-rs.git
   ```
2. Change into the repository directory:
   ```shell
   cd fstools-rs
   ```
3. Compile the project using Cargo:
   ```shell
   cargo build --release
   ```
   The resulting binaries are located in `target/release`.

## Usage

`fstools` is invoked via the command line, with each utility accessible through subcommands. For example, to recursively extract a BND within the DVDBNDs:

```shell
fstools-cli extract --recursive -o <destination_directory> [optional_dvdbnd_name]
```

Detailed documentation on subcommand parameters and options is available within the `--help` output.

<p align="right">(<a href="#readme-top">back to top</a>)</p>


<!-- CONTRIBUTING -->
## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

<!-- LICENSE -->
## License

Distributed under either the Apache Software License 2.0 or MIT License. See LICENSE-APACHE and LICENSE-MIT for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- CONTACT -->
## Contact

Project Link: [https://github.com/soulsmods/fstools-rs](https://github.com/soulsmods/fstools-rs)

Discussions Board: [https://github.com/soulsmods/fstools-rs/discussions](https://github.com/soulsmods/fstools-rs/discussions)

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

* [SoulsFormats](https://github.com/JKAnderson/SoulsFormats) - widely used and de-facto library for interacting with FROMSOFTWARE assets.
  
<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[buildtools-installer]: https://aka.ms/vs/17/release/vs_BuildTools.exe
[discussions-shield]: https://img.shields.io/github/discussions/soulsmods/fstools-rs
[discussions-url]: https://github.com/soulsmods/fstools-rs/discussions
[contributors-shield]: https://img.shields.io/github/contributors/soulsmods/fstools-rs.svg?style=flat
[contributors-url]: https://github.com/soulsmods/fstools-rs/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/soulsmods/fstools-rs.svg?style=flat
[forks-url]: https://github.com/soulsmods/fstools-rs/network/members
[stars-shield]: https://img.shields.io/github/stars/soulsmods/fstools-rs.svg?style=flat
[stars-url]: https://github.com/soulsmods/fstools-rs/stargazers
[issues-shield]: https://img.shields.io/github/issues/soulsmods/fstools-rs.svg?style=flat
[issues-url]: https://github.com/soulsmods/fstools-rs/issues
[license-shield]: https://img.shields.io/badge/license-MIT%2FApache--2.0-green?style=flat
[rust-install-instructions]: https://www.rust-lang.org/tools/install