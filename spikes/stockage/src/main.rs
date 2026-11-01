//! Spike P0 — volumes et type de média sous Windows, **en lecture seule**.
//!
//! Binaire d'exploration jetable, hors du code de production (cahier des charges, OBL-006).
//! Il ne supprime rien et ne défragmente rien : il lit.
//!
//! Questions du spike :
//!  1. la propriété « pénalité de recherche » distingue-t-elle vraiment HDD, SSD et NVMe ?
//!  2. TRIM est-il exposé par le périphérique, et sous quelle forme ?
//!  3. quels volumes sont « fixes », et lesquels mentent (virtuels, réseau) ?

use std::ffi::c_void;

const GENERIC_READ: u32 = 0x8000_0000;
const FILE_SHARE_READ: u32 = 0x0000_0001;
const FILE_SHARE_WRITE: u32 = 0x0000_0002;
const OPEN_EXISTING: u32 = 3;
const INVALID_HANDLE_VALUE: isize = -1;

/// `IOCTL_STORAGE_QUERY_PROPERTY`
const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x002D_1400;
/// `StorageDeviceSeekPenaltyProperty`
const PROPERTY_SEEK_PENALTY: u32 = 7;
/// `StorageDeviceTrimProperty`
const PROPERTY_TRIM: u32 = 8;

const DRIVE_REMOVABLE: u32 = 2;
const DRIVE_FIXED: u32 = 3;
const DRIVE_REMOTE: u32 = 4;
const DRIVE_CDROM: u32 = 5;
const DRIVE_RAMDISK: u32 = 6;

#[repr(C)]
struct StoragePropertyQuery {
    property_id: u32,
    query_type: u32,
    /// `AdditionalParameters[1]` complété à l'alignement d'un `u32`.
    additional: [u8; 4],
}

/// `DEVICE_SEEK_PENALTY_DESCRIPTOR` et `DEVICE_TRIM_DESCRIPTOR` ont la même forme.
#[repr(C)]
#[derive(Default)]
struct DeviceDescriptor {
    version: u32,
    size: u32,
    enabled: u8,
    _padding: [u8; 3],
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetLogicalDriveStringsW(length: u32, buffer: *mut u16) -> u32;
    fn GetDriveTypeW(root: *const u16) -> u32;
    fn GetVolumeInformationW(
        root: *const u16,
        volume_name: *mut u16,
        volume_size: u32,
        serial: *mut u32,
        max_component: *mut u32,
        flags: *mut u32,
        filesystem: *mut u16,
        filesystem_size: u32,
    ) -> i32;
    fn GetDiskFreeSpaceExW(
        directory: *const u16,
        free_to_caller: *mut u64,
        total: *mut u64,
        total_free: *mut u64,
    ) -> i32;
    fn CreateFileW(
        name: *const u16,
        access: u32,
        share: u32,
        security: *mut c_void,
        disposition: u32,
        flags: u32,
        template: *mut c_void,
    ) -> *mut c_void;
    fn DeviceIoControl(
        device: *mut c_void,
        code: u32,
        input: *mut c_void,
        input_size: u32,
        output: *mut c_void,
        output_size: u32,
        returned: *mut u32,
        overlapped: *mut c_void,
    ) -> i32;
    fn CloseHandle(handle: *mut c_void) -> i32;
    fn GetLastError() -> u32;
}

fn main() {
    println!("{{\"tool\":\"spike-stockage\",\"os\":\"windows\",\"volumes\":[");
    let drives = logical_drives();
    for (index, drive) in drives.iter().enumerate() {
        let separator = if index + 1 == drives.len() { "" } else { "," };
        println!("{}{separator}", describe(drive));
    }
    println!("]}}");
}

/// Toutes les racines de volume (`C:\`), y compris celles sans lettre usuelle.
fn logical_drives() -> Vec<String> {
    let mut buffer = [0u16; 512];
    // SAFETY: tampon de taille fixe, longueur annoncée ; l'appel écrit au plus cette taille.
    let length = unsafe { GetLogicalDriveStringsW(buffer.len() as u32, buffer.as_mut_ptr()) };
    if length == 0 {
        return Vec::new();
    }
    let mut drives = Vec::new();
    let mut start = 0usize;
    for index in 0..length as usize {
        if buffer[index] == 0 {
            if index > start {
                drives.push(String::from_utf16_lossy(&buffer[start..index]));
            }
            start = index + 1;
        }
    }
    drives
}

fn describe(root: &str) -> String {
    let wide = to_wide(root);
    // SAFETY: `wide` est une chaîne UTF-16 terminée par NUL, valide le temps de l'appel.
    let drive_type = unsafe { GetDriveTypeW(wide.as_ptr()) };

    let mut filesystem = [0u16; 64];
    let mut serial = 0u32;
    let mut max_component = 0u32;
    let mut flags = 0u32;
    // SAFETY: tous les pointeurs désignent des tampons de la taille annoncée.
    let ok = unsafe {
        GetVolumeInformationW(
            wide.as_ptr(),
            std::ptr::null_mut(),
            0,
            &mut serial,
            &mut max_component,
            &mut flags,
            filesystem.as_mut_ptr(),
            filesystem.len() as u32,
        )
    };
    let filesystem = if ok != 0 {
        String::from_utf16_lossy(&filesystem)
            .trim_end_matches('\0')
            .to_owned()
    } else {
        String::new()
    };

    let mut free_to_caller = 0u64;
    let mut total = 0u64;
    let mut total_free = 0u64;
    // SAFETY: trois pointeurs vers des `u64` initialisés.
    let space_ok = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free_to_caller,
            &mut total,
            &mut total_free,
        )
    };
    let (total, total_free) = if space_ok != 0 {
        (total, total_free)
    } else {
        (0, 0)
    };

    let seek = query_descriptor(root, PROPERTY_SEEK_PENALTY);
    let trim = query_descriptor(root, PROPERTY_TRIM);

    format!(
        "{{\"root\":{root:?},\"type\":\"{}\",\"filesystem\":{filesystem:?},\
         \"total\":{total},\"free\":{total_free},\
         \"incurs_seek_penalty\":{},\"trim_enabled\":{}}}",
        kind(drive_type),
        seek.map_or("null".to_owned(), |value| format!("{value}")),
        trim.map_or("null".to_owned(), |value| format!("{value}"))
    )
}

/// Interroge une propriété du périphérique (`seek penalty`, `TRIM`).
///
/// Rend `None` quand l'appel échoue : un volume peut refuser l'ouverture (`\\.\X:` demande
/// parfois des droits), et ce refus est une **information** — il ne se transforme pas en
/// « SSD » par défaut.
fn query_descriptor(root: &str, property: u32) -> Option<bool> {
    let device = format!(r"\\.\{}", root.trim_end_matches('\\'));
    let wide = to_wide(&device);
    // SAFETY: chaîne UTF-16 terminée par NUL.
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            // `0` : on demande l'ouverture sans droit d'accès, seulement pour interroger le
            // périphérique. Avec `GENERIC_READ`, un volume système refuse l'ouverture à un
            // utilisateur non administrateur (erreur 5) — c'est le premier piège du spike.
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        )
    };
    if handle as isize == INVALID_HANDLE_VALUE {
        // SAFETY: aucun argument ; lit le code d'erreur du fil courant.
        let code = unsafe { GetLastError() };
        eprintln!("CreateFile {device} -> erreur {code}");
        return None;
    }

    let mut query = StoragePropertyQuery {
        property_id: property,
        query_type: 0,
        additional: [0; 4],
    };
    let mut descriptor = DeviceDescriptor::default();
    let mut returned = 0u32;
    // SAFETY: les deux tampons sont de la taille annoncée, et le handle est valide jusqu'à
    // `CloseHandle`.
    let ok = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_STORAGE_QUERY_PROPERTY,
            (&mut query as *mut StoragePropertyQuery).cast(),
            size_of::<StoragePropertyQuery>() as u32,
            (&mut descriptor as *mut DeviceDescriptor).cast(),
            size_of::<DeviceDescriptor>() as u32,
            &mut returned,
            std::ptr::null_mut(),
        )
    };
    // SAFETY: le handle provient de `CreateFileW` et n'est plus utilisé après.
    unsafe { CloseHandle(handle) };

    if ok == 0 {
        // SAFETY: aucun argument ; lit le code d'erreur du fil courant.
        let code = unsafe { GetLastError() };
        eprintln!("DeviceIoControl {device} (propriété {property}) -> erreur {code}");
        return None;
    }
    Some(descriptor.enabled != 0)
}

fn kind(drive_type: u32) -> &'static str {
    match drive_type {
        DRIVE_REMOVABLE => "amovible",
        DRIVE_FIXED => "fixe",
        DRIVE_REMOTE => "réseau",
        DRIVE_CDROM => "optique",
        DRIVE_RAMDISK => "ramdisk",
        _ => "inconnu",
    }
}

fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
