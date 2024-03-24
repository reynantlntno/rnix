use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Read;
use std::path::Path;
use std::io::Write;

use bcrypt::{hash, verify};

use fatfs::{Dir, FatType, FileSystem, FormatVolumeOptions, FsOptions, ReadWriteSeek};

const DISK_IMAGE_SIZE: u64 = 128 * 1024 * 1024; // 128 MB
// const ROOT_DIR: &str = "/";

// Function to clear the terminal
pub fn clear() {
    println!("{}[2J{}[1;1H", 27 as char, 27 as char);
}

pub fn setup(root_dir: &mut Dir<'_, File>) -> io::Result<()> {
    // Check if the root account already exists
    if root_dir.open_file("internal/root").is_err() {
        // Root account doesn't exist, create it
        let mut root_file = root_dir.create_file("internal/root")?;
        let hashed_password = hashp("iloveapple"); // Hash the default password
        let encrypted_account = format!("root:{}", hashed_password);
        let mut encrypted_bytes = encrypted_account.into_bytes();
        edcrypt(&mut encrypted_bytes); // Encrypt the data before writing
        root_file.write_all(&encrypted_bytes)?;
        println!("Root account created.");
    }

    // Now proceed with setting up user account
    println!("RNIX | Setting up user account:");
    print!("Enter user username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim();

    print!("Enter user password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();

    let mut users_file = match root_dir.open_file("internal/rnix") {
        Ok(file) => file,
        Err(_) => {
            let mut users_file = root_dir.create_file("internal/rnix")?;
            let hashed_password = hashp(password); // Hash the password
            let encrypted_account = format!("{}:{}", username, hashed_password);
            let mut encrypted_bytes = encrypted_account.into_bytes();
            edcrypt(&mut encrypted_bytes); // Encrypt the data before writing
            users_file.write_all(&encrypted_bytes)?;
            println!("User account created.");
            return Ok(());
        }
    };

    // Existing logic for reading and decrypting user accounts remains unchanged
    let mut contents = Vec::new();
    users_file.read_to_end(&mut contents)?;
    let mut decrypted_contents = contents.clone();
    edcrypt(&mut decrypted_contents);
    let decrypted_str = String::from_utf8(decrypted_contents).unwrap_or_default();
    let accounts: Vec<&str> = decrypted_str.trim().lines().collect();

    if !accounts.is_empty() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "No user accounts found",
        ))
    }
}


pub fn auwp(root_dir: &Dir<'_, File>, username: &str, password: &str) -> io::Result<bool> {
    // Check if the username is root
    if username == "root" {
        // Try to open the root account file
        let mut root_file = match root_dir.open_file("internal/root") {
            Ok(file) => file,
            Err(_) => return Ok(false), // Root account not found
        };

        let mut contents = Vec::new(); // Change to Vec<u8> to read bytes
        root_file.read_to_end(&mut contents)?; // Read encrypted bytes

        let mut decrypted_contents = contents.clone(); // Create a clone for decryption
        edcrypt(&mut decrypted_contents); // Decrypt the data

        let decrypted_str = String::from_utf8(decrypted_contents).unwrap_or_default();
        let parts: Vec<&str> = decrypted_str.split(':').collect();
        if parts.len() == 2 && parts[0].trim() == username {
            // Verify password using bcrypt constant-time comparison
            if verify(password, parts[1].trim()).unwrap_or(false) {
                return Ok(true);
            }
        }
    } else {
        // User is not root, proceed with the existing logic
        let mut users_file = match root_dir.open_file("internal/rnix") {
            Ok(file) => file,
            Err(_) => return Ok(false), // File not found, return false
        };

        let mut contents = Vec::new(); // Change to Vec<u8> to read bytes
        users_file.read_to_end(&mut contents)?; // Read encrypted bytes

        let mut decrypted_contents = contents.clone(); // Create a clone for decryption
        edcrypt(&mut decrypted_contents); // Decrypt the data

        let decrypted_str = String::from_utf8(decrypted_contents).unwrap_or_default();
        let accounts: Vec<&str> = decrypted_str.trim().lines().collect();

        for account in accounts {
            let parts: Vec<&str> = account.split(':').collect();
            if parts.len() == 2 && parts[0].trim() == username {
                // Verify password using bcrypt constant-time comparison
                if verify(password, parts[1].trim()).unwrap_or(false) {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}


// Function to securely hash passwords using bcrypt
pub fn hashp(password: &str) -> String {
    hash(password, bcrypt::DEFAULT_COST).expect("Failed to hash password")
}


pub fn ocdi(path: &str, size: u64) -> io::Result<File> {
    if Path::new(path).exists() {
        OpenOptions::new().read(true).write(true).open(path)
    } else {
        let file = File::create(path)?;
        file.set_len(size)?;
        Ok(file)
    }
}

pub fn dformatq(path: &str) -> io::Result<bool> {
    let file = OpenOptions::new().read(true).write(true).open(path)?;
    let options = FsOptions::new();
    match FileSystem::new(file, options) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub fn dformat(path: &str) -> io::Result<()> {
    if !dformatq(path)? {
        let mut file = OpenOptions::new().read(true).write(true).open(path)?;
        let volume_label = "RNIX";
        let mut volume_label_bytes = [b' '; 11];
        let label_bytes = volume_label.as_bytes();
        let len = label_bytes.len().min(11);
        volume_label_bytes[..len].copy_from_slice(&label_bytes[..len]);
        let format_options = FormatVolumeOptions::new()
            .fat_type(FatType::Fat32)
            .volume_label(volume_label_bytes);
        fatfs::format_volume(&mut file, format_options)?;
    }
    Ok(())
}

pub fn lsdisks(mounted_disks: &HashSet<String>) -> io::Result<()> {
    println!("Disks:");
    println!("  /dev/disk0 - Root Disk");
    if mounted_disks.is_empty() {
        println!("  No disks currently mounted.");
    } else {
        for disk in mounted_disks {
            let bsd_name = format!("/dev/{}", disk);
            println!("  {} - {}", disk, bsd_name);
        }
    }
    Ok(())
}

pub fn createdisk(disk_path: &str, disk_name: &str) -> io::Result<()> {
    let file = ocdi(disk_path, DISK_IMAGE_SIZE)?;
    dformat(disk_path)?;
    println!("Disk created: {}", disk_name);
    drop(file); // Ensure the file is closed
    Ok(())
}

pub fn mountdisk(disk_name: &str, mounted_disks: &mut HashSet<String>, root_dir: &mut Dir<'_, File>) -> io::Result<()> {
    match disk_name {
        "disk0" => println!("Rnix Terminal --> /dev/disk0 mounted as root"),
        "disk1" | "disk2" => {
            let disk_img = format!("{}.img", disk_name);

            // Create 'volumes/{disk_name}' directory if it doesn't exist
            let mount_point = format!("volumes/{}", disk_name);

            if !root_dir.open_dir(&mount_point).is_ok() {
                root_dir.create_dir(&mount_point)?;
            }

            if Path::new(&disk_img).exists() {
                if mounted_disks.contains(disk_name) {
                    println!("Disk {} is already mounted.", disk_name);
                } else {
                    println!("Disk {} mounted.", disk_name);
                    mounted_disks.insert(disk_name.to_string());

                    // Update current_dir after mounting
                    *root_dir = root_dir.open_dir(&mount_point)?;
                }
            } else {
                println!("Disk image {} does not exist.", disk_img);
            }
        }
        _ => println!("Invalid disk name. Only disk1 and disk2 can be mounted."),
    }
    Ok(())
}


pub fn umountdisk(disk_name: &str, mounted_disks: &mut HashSet<String>, _root_dir: &mut Dir<'_, File>) -> io::Result<()> {
    match disk_name {
        "disk0" => println!("Cannot unmount root disk."),
        "disk1" | "disk2" => {
            // Remove the disk from the mounted disks set
            if mounted_disks.contains(disk_name) {
                mounted_disks.remove(disk_name);
                println!("Disk {} unmounted.", disk_name);
            } else {
                println!("Disk {} is not currently mounted.", disk_name);
            }
        }
        _ => println!("Invalid disk name. Only disk1 and disk2 can be unmounted."),
    }
    Ok(())
}



pub fn mkdir<T: ReadWriteSeek>(parent_dir: &Dir<'_, T>, dir_name: &str) -> io::Result<()> {
    parent_dir.create_dir(dir_name)?;
    println!("Directory '{}' created.", dir_name);
    Ok(())
}

pub fn touch<T: ReadWriteSeek>(parent_dir: &Dir<'_, T>, file_name: &str) -> io::Result<()> {
    parent_dir.create_file(file_name)?;
    println!("File '{}' created.", file_name);
    Ok(())
}

pub fn cd<T: ReadWriteSeek>(
    current_dir: &mut Dir<'_, T>,
    new_dir_name: &str,
    current_dir_path: &mut String,
    suppress_message: bool,
) -> io::Result<()> {
    match current_dir.open_dir(new_dir_name) {
        Ok(new_dir) => {
            *current_dir = new_dir;
            *current_dir_path = format!("{}/{}", current_dir_path, new_dir_name);
            if !suppress_message {
                println!("Changed directory to '{}'.", new_dir_name);
            }
            Ok(())
        }
        Err(_) => {
            if !suppress_message {
                println!("Directory '{}' not found.", new_dir_name);
            }
            Err(io::Error::new(io::ErrorKind::NotFound, "Directory not found"))
        }
    }
}


pub fn rmfile<T: ReadWriteSeek>(current_dir: &mut Dir<'_, T>, file_name: &str) -> io::Result<()> {
    current_dir.remove(file_name)?;
    println!("File '{}' removed.", file_name);
    Ok(())
}

pub fn rmdir<T: ReadWriteSeek>(current_dir: &mut Dir<'_, T>, dir_name: &str) -> io::Result<()> {
    current_dir.remove(dir_name)?;
    println!("Directory '{}' removed.", dir_name);
    Ok(())
}

pub fn rename<T: ReadWriteSeek>(
    current_dir: &mut Dir<'_, T>,
    src_path: &str,
    dst_dir: &Dir<'_, T>,
    dst_path: &str,
) -> io::Result<()> {
    let mut src_full_path = String::from("internal/");
    src_full_path.push_str(src_path);

    let mut dst_full_path = String::from("internal/");
    dst_full_path.push_str(dst_path);

    // Validate source and destination paths
    if !src_full_path.starts_with("internal/") || !dst_full_path.starts_with("internal/") {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid source or destination path"));
    }

    current_dir.rename(src_path, dst_dir, dst_path)?;
    println!("Renamed '{}' to '{}'.", src_path, dst_path);
    Ok(())
}


pub fn cp<T: ReadWriteSeek>(
    src_dir: &Dir<'_, T>,
    src_file_name: &str,
    dst_dir: &mut Dir<'_, T>,
    dst_file_name: &str,
) -> io::Result<()> {
    let mut src_file = src_dir.open_file(src_file_name)?;
    let mut dst_file = dst_dir.create_file(dst_file_name)?;

    io::copy(&mut src_file, &mut dst_file)?;
    println!("File '{}' copied to '{}'.", src_file_name, dst_file_name);

    Ok(())
}




pub fn edit<T: ReadWriteSeek>(
    current_dir: &mut Dir<'_, T>,
    file_name: &str,
) -> io::Result<()> {
    // Check if the file exists
    if current_dir.open_file(file_name).is_err() {
        println!("File '{}' not found.", file_name);
        return Ok(());
    }

    // Open the file for reading and writing
    let mut file = current_dir.open_file(file_name)?;

    // Read the existing contents of the file
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Print the contents for editing
    println!("Editing file '{}':", file_name);
    println!("---------------------------");
    println!("{}", contents);
    println!("---------------------------");

    // Prompt the user to enter new contents
    println!("Enter new contents below. Press Ctrl+D (Ctrl+Z on Windows) to save and exit.");
    let mut new_contents = String::new();
    io::stdin().read_to_string(&mut new_contents)?;

    // Truncate the file to remove existing contents
    file.truncate()?;

    // Write the new contents to the file
    file.write_all(new_contents.as_bytes())?;

    println!("File '{}' has been updated.", file_name);

    Ok(())
}

// Function to securely encrypt data
pub fn edcrypt(data: &mut [u8]) {
    let key: [u8; 9] = [7, 19, 4, 1, 3, 6, 11, 5, 2]; // Example of a more secure key
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % key.len()];
    }
}


use std::fmt;

// Struct to hold RNIX version information
pub struct RnixVersion {
    version: &'static str,
}

impl fmt::Display for RnixVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RNIX Version: {}", self.version)
    }
}

// Function to get RNIX version information
pub fn get_rnix_version() -> RnixVersion {
    RnixVersion {
        version: "1.0.0", // Update with the actual version
    }
}

// Struct to hold RNIX API version information
pub struct RnixApiVersion {
    version: &'static str,
}

impl fmt::Display for RnixApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RNIX API Version: {}", self.version)
    }
}

// Function to get RNIX API version information
pub fn get_rnix_api_version() -> RnixApiVersion {
    RnixApiVersion {
        version: "1.0.0", // Update with the actual version
    }
}

pub fn resetroot(root_dir: &mut Dir<'_, File>, current_username: &str) -> io::Result<()> {
    // Check if the user is root
    if current_username != "root" {
        println!("Only root can execute resetroot command.");
        return Ok(());
    }

    // Debug information
    println!("Resetting root disk...");
    
    // Remove the setup_completed.flag file
    match rmfile(root_dir, "internal/setup_completed.flag") {
        Ok(_) => println!("setup_completed.flag removed."),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            println!("setup_completed.flag not found. Skipping...");
        }
        Err(err) => return Err(err),
    }

    // Remove the rnix directory
    match rmfile(root_dir, "internal/rnix") {
        Ok(_) => println!("rnix file removed."),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            println!("rnix file not found. Skipping...");
        }
        Err(err) => return Err(err),
    }

    // Print completion message
    println!("Root disk reset complete. Please restart the program.");

    Ok(())
}


use std::io::Cursor;


// Function to read the contents of a virtual disk image
pub fn readdisk(path: &str) -> io::Result<Cursor<Vec<u8>>> {
    // Open the disk image file
    let mut file = std::fs::File::open(path)?;

    // Read the entire contents of the file into a vector
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Create a cursor from the buffer
    let cursor = Cursor::new(buffer);

    Ok(cursor)
}

// Function to display the files and directories inside a virtual disk image
pub fn displaydisk(path: &str) -> io::Result<()> {
    println!("Contents of disk image '{}':", path);

    // Read the disk image into a cursor
    let image_data = readdisk(path)?;

    // Create a file system object from the cursor
    let options = FsOptions::new();
    let fs = match fatfs::FileSystem::new(image_data, options) {
        Ok(fs) => fs,
        Err(_) => {
            println!("Failed to parse file system.");
            return Ok(());
        }
    };

    // Print the list of files and directories
    for entry in fs.root_dir().iter() {
        match entry {
            Ok(entry) => {
                println!("{}", entry.file_name());
            }
            Err(_) => {
                println!("Error reading entry");
            }
        }
    }

    Ok(())
}
