# Rnix

**Rnix** is a Rust program that simulates a UNIX-like operating system. It provides functionalities for managing a filesystem, user authentication, disk operations, and other related tasks.

## Features

- **Filesystem operations:** Create directories, create files, remove files, remove directories, rename files/directories, change directories, list mounted disks, create disk images, mount/unmount disks, etc.
- **User authentication:** Set up user accounts (including root account) and authenticate users with hashed passwords using bcrypt.
- **Disk operations:** Open/create disk images, check if disks are formatted, format disks, read disk images, etc.
- **Utility functions:** Clear terminal, hash passwords, encrypt data, reset root disk, etc.
- **Version information:** Get RNIX and RNIX API version information.
- **Editing and displaying disk contents.**

## Installation

To use **Rnix**, you'll need Rust installed on your system. You can install Rust by following the instructions on the [official Rust website](https://www.rust-lang.org/tools/install).

Once Rust is installed, you can clone the **Rnix** repository and build the project:

```bash
git clone https://github.com/reynantlntno/rnix.git
cd rnix
cargo build --release
```

## Usage

After building the project, you can run the Rnix program by executing the binary file generated in the `target/release` directory:

```bash
./target/release/rnix
```

This will start the Rnix program, allowing you to interact with the simulated operating system through a command-line interface.

## License

Rnix is licensed under the MIT License. See the `LICENSE` file for details.

## Contributing

Contributions to Rnix are welcome! If you encounter any issues or have ideas for improvements, feel free to open an issue or submit a pull request on the [GitHub repository](https://github.com/reynantlntno/rnix).

## Acknowledgements

Rnix is inspired by UNIX-like operating systems and built with the help of various Rust libraries and tools.

## Contact

For any inquiries or feedback, you can reach out to the project maintainer:

- **Reynan Tolentino**
  - GitHub: [reynantlntno](https://github.com/reynantlntno)
  - Email: knots-osier-0m@icloud.com
