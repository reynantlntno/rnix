mod libs;

use std::process::Stdio;
use std::{collections::HashSet, process::Command};
use std::io;
use std::io::prelude::*;

use fatfs::{FileSystem, FsOptions};

use libs::*;

const DISK_IMAGE_SIZE: u64 = 128 * 1024 * 1024; // 128 MB
const DISK_PATH: &str = "rnix.img";

fn main() -> Result<(), io::Error> {
    clear();

    let file = ocdi(DISK_PATH, DISK_IMAGE_SIZE)?;

    dformat(DISK_PATH)?;

    let mut mounted_disks: HashSet<String> = HashSet::new();

    let options = FsOptions::new();
    let fs = FileSystem::new(file, options)?;

    let mut root_dir = fs.root_dir();

    // Create 'internal' directory if it doesn't exist
    if !root_dir.open_dir("internal").is_ok() {
        root_dir.create_dir("internal")?;
    }

    // Create 'bin' directory inside 'internal' if it doesn't exist
    let internal_dir = root_dir.open_dir("internal")?;
    if !internal_dir.open_dir("bin").is_ok() {
        internal_dir.create_dir("bin")?;
    }

    // Create 'home' directory if it doesn't exist
    if !root_dir.open_dir("home").is_ok() {
        root_dir.create_dir("home")?;
    }

    // Create 'volumes' directory if it doesn't exist
    if !root_dir.open_dir("volumes").is_ok() {
        root_dir.create_dir("volumes")?;
    }

    // Check if the setup flag file exists
    let setup_flag_path = "internal/setup_completed.flag";
    if root_dir.open_file(setup_flag_path).is_err() {
        // Setup accounts and create the flag file
        setup(&mut root_dir)?;
        // Create the setup flag file
        root_dir.create_file(setup_flag_path)?;
    }

    loop {
        print!("-----------------\nRNIX | LogIn\n-----------------\nEnter username: ");
        io::stdout().flush()?;
        let mut current_username = String::new();
        io::stdin().read_line(&mut current_username)?;
        let current_username = current_username.trim();

        print!("Enter password: ");
        io::stdout().flush()?;
        let mut password = String::new();
        io::stdin().read_line(&mut password)?;
        let password = password.trim();

        if !auwp(&root_dir, current_username, password)? {
            println!("Invalid username or password. Please try again.");
            continue;
        }
        clear();
        mountdisk("disk0", &mut mounted_disks, &mut root_dir)?;

        let mut current_dir_path = String::new();

        // Change current directory to home directory without printing the message
        cd(&mut root_dir, "home", &mut current_dir_path, true)?;

        // If the current user is root, change the working directory to the root directory ("/")
        if current_username == "root" {
            root_dir = fs.root_dir();
        }

        let mut current_dir_path = "/".to_string();

        loop {
            print!("{}(rnix) > ", current_username);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let mut args = input.trim().split_whitespace();
            let command = args.next().unwrap_or("");

            if input.starts_with("run ") {
                let command_parts: Vec<&str> = input.split_whitespace().collect();
                if command_parts.len() < 2 {
                    println!("Usage: run <executable_name>");
                    continue;
                }
                let executable_name = command_parts[1];

                match Command::new(executable_name)
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn()
                {
                    Ok(_) => println!("Executing {}...", executable_name),
                    Err(err) => eprintln!("Error executing {}: {}", executable_name, err),
                }
            } else if command == "sudo" {
                // Prompt for password
                print!("Password: ");
                io::stdout().flush()?;
                let mut input_password = String::new();
                io::stdin().read_line(&mut input_password)?;
                let password = input_password.trim();

                // Authenticate user with password
                if !auwp(&root_dir, current_username, password)? {
                    println!("Incorrect password. Access denied.");
                    continue;
                }

                // Continue with executing the specified command and its arguments
                let sudo_command = match args.next() {
                    Some(cmd) => cmd,
                    None => {
                        println!("Usage: sudo [command]");
                        continue;
                    }
                };

                match sudo_command {
                    "mount" => {
                        let disk_name = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Usage: mount <disk_name>");
                                continue;
                            }
                        };
                        mountdisk(disk_name, &mut mounted_disks, &mut root_dir)?;
                    }
                    "umount" => {
                        let disk_name = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Usage: sudo umount <disk_name>");
                                continue;
                            }
                        };
                        umountdisk(disk_name, &mut mounted_disks, &mut root_dir)?;
                    }
                    _ => {
                        println!("Unknown command. Type 'help' for available commands.");
                    }
                }
            } else {
                // Warn if the command requires sudo
                let require_sudo = match command {
                    "mount" | "umount" => true,
                    _ => false,
                };

                if require_sudo {
                    println!(
                        "This command requires sudo privileges. Use 'sudo {}' to run this command.",
                        command
                    );
                    continue;
                }

                let _ = match command {
                    "listdisks" => lsdisks(&mounted_disks),
                    "createdisk" => {
                        let disk_name = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Usage: createdisk <disk_name>");
                                continue;
                            }
                        };
                        createdisk(&format!("{}.img", disk_name), disk_name)
                    }
                    "mount" => Ok({
                        let disk_name = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Usage: mount <disk_name>");
                                continue;
                            }
                        };
                        mountdisk(disk_name, &mut mounted_disks, &mut root_dir)?;
                    }),
                    "umount" => Ok({
                        let disk_name = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Usage: umount <disk_name>");
                                continue;
                            }
                        };
                        umountdisk(disk_name, &mut mounted_disks, &mut root_dir)?;
                    }),
                    "mkdir" => {
                        let dir_name = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Usage: mkdir <directory_name>");
                                continue;
                            }
                        };
                        mkdir(&root_dir, dir_name)
                    }
                    "touch" => {
                        let file_name = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Usage: touch <file_name>");
                                continue;
                            }
                        };
                        touch(&root_dir, file_name)
                    }
                    "rm" => {
                        let item_name = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Usage: rm <file_or_directory>");
                                continue;
                            }
                        };
                        if root_dir.open_file(item_name).is_ok() {
                            rmfile(&mut root_dir, item_name)
                        } else if root_dir.open_dir(item_name).is_ok() {
                            rmdir(&mut root_dir, item_name)
                        } else {
                            println!("Item '{}' not found.", item_name);
                            Ok(()) // Return Ok(()) to continue execution
                        }
                    }
                    "mv" => {
                        let src_path = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Usage: mv <source_path> <destination_directory> <destination_path>");
                                continue;
                            }
                        };
                        let dst_dir_name = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Usage: mv <source_path> <destination_directory> <destination_path>");
                                continue;
                            }
                        };
                        let dst_path = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Usage: mv <source_path> <destination_directory> <destination_path>");
                                continue;
                            }
                        };
                        let dst_dir = match root_dir.open_dir(dst_dir_name) {
                            Ok(dir) => dir,
                            Err(_) => {
                                println!("Destination directory '{}' not found.", dst_dir_name);
                                continue;
                            }
                        };
                        rename(&mut root_dir, src_path, &dst_dir, dst_path)
                    }
                    "cp" => {
                        let src_path = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Usage: cp <source_path> <destination_directory> <destination_file>");
                                continue;
                            }
                        };
                        let dst_dir_name = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Usage: cp <source_path> <destination_directory> <destination_file>");
                                continue;
                            }
                        };
                        let dst_file_name = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Usage: cp <source_path> <destination_directory> <destination_file>");
                                continue;
                            }
                        };
                        let mut dst_dir = match root_dir.open_dir(dst_dir_name) {
                            Ok(dir) => dir,
                            Err(_) => {
                                println!("Destination directory '{}' not found.", dst_dir_name);
                                continue;
                            }
                        };
                        cp(&root_dir, src_path, &mut dst_dir, dst_file_name)
                    }
                    "ls" => {
                        let dir_name = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Contents of current directory:");
                                for entry in root_dir.iter() {
                                    match entry {
                                        Ok(entry) => println!("{}", entry.file_name()),
                                        Err(_) => println!("Error reading entry"),
                                    }
                                }
                                continue;
                            }
                        };
                        match root_dir.open_dir(dir_name) {
                            Ok(dir) => {
                                println!("Contents of directory '{}':", dir_name);
                                for entry in dir.iter() {
                                    match entry {
                                        Ok(entry) => println!("{}", entry.file_name()),
                                        Err(_) => println!("Error reading entry"),
                                    }
                                }
                                Ok(()) // Return Ok(()) to continue execution
                            }
                            Err(_) => {
                                println!("Directory '{}' not found.", dir_name);
                                Ok(()) // Return Ok(()) to continue execution
                            }
                        }
                    }
                    "clear" => {
                        clear();
                        Ok(()) // Return Ok(()) to continue execution
                    }
                    "cd" => Ok({
                        let new_dir_name = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Usage: cd <directory_name>");
                                continue;
                            }
                        };
                        cd(&mut root_dir, new_dir_name, &mut current_dir_path, false)?;
                    }),
                    "whoami" => {
                        println!("{}", current_username);
                        Ok(()) // Return Ok(()) to continue execution
                    }
                    "readdisk" => Ok({
                        let disk_path = match args.next() {
                            Some(path) => path,
                            None => {
                                println!("Usage: readdisk <disk_path>");
                                continue;
                            }
                        };
                        match displaydisk(disk_path) {
                            Ok(_) => {}
                            Err(err) => println!("Error reading disk image: {}", err),
                        }
                    }),
                    "help" => {
                        println!("Available commands:");
                        println!("  listdisks - List mounted disks");
                        println!("  createdisk <disk_name> - Create a new disk image");
                        println!("  mount <disk_name> - Mount a disk");
                        println!("  umount <disk_name> - Unmount a disk");
                        println!("  mkdir <directory_name> - Create a new directory");
                        println!("  touch <file_name> - Create a new file");
                        println!("  rm <file_or_directory> - Remove a file or directory");
                        println!("  mv <source_path> <destination_directory> <destination_path> - Move or rename a file or directory");
                        println!("  cp <source_path> <destination_directory> <destination_file> - Copy a file");
                        println!("  ls - List contents of a directory");
                        println!("  clear - Clear the terminal");
                        println!("  whoami - Display current user");
                        println!("  exit - Exit the program");
                        Ok(()) // Return Ok(()) to continue execution
                    }
                    "version" => {
                        println!("{}", get_rnix_version());
                        println!("{}", get_rnix_api_version());
                        Ok(())
                    }
                    "resetroot" => Ok(resetroot(&mut root_dir, current_username)?),

                    "edit" => {
                        let file_name = match args.next() {
                            Some(name) => name,
                            None => {
                                println!("Usage: edit <file_name>");
                                continue;
                            }
                        };
                        edit(&mut root_dir, file_name)
                    }
                    "pwd" => {
                        println!("{}", current_dir_path);
                        Ok(())
                    }
                    "exit" => {
                        // Call the exit executable
                        let _ = std::process::Command::new("internal/bin")
                            .status()
                            .expect("Failed to execute 'exit' command.");
                        return Ok(());
                    }
                    _ => {
                        println!("Unknown command. Type 'help' for available commands.");
                        Ok(()) // Return Ok(()) to continue execution
                    }
                };
            }
        }
    }
}
