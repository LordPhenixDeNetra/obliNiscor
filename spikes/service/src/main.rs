//! Spike P0 — canal nommé Windows et descripteur de sécurité (OBL-008).
//!
//! Binaire d'exploration jetable, hors du code de production. Il répond à trois questions :
//!
//!  1. un canal nommé peut-il être créé avec une **ACL explicite** plutôt qu'héritée ?
//!  2. un client du même utilisateur s'y connecte-t-il ?
//!  3. un client visé par une entrée **refus** est-il réellement rejeté, et avec quel code ?
//!
//! Le canal porte un nom unique et disparaît à la fermeture du dernier handle : rien ne reste
//! sur le système. Aucune donnée n'est écrite.

use std::ffi::c_void;

type Handle = *mut c_void;
const INVALID_HANDLE_VALUE: isize = -1;

const PIPE_ACCESS_DUPLEX: u32 = 0x0000_0003;
const PIPE_TYPE_MESSAGE: u32 = 0x0000_0004;
const PIPE_READMODE_MESSAGE: u32 = 0x0000_0002;
const PIPE_WAIT: u32 = 0x0000_0000;
const GENERIC_READ: u32 = 0x8000_0000;
const GENERIC_WRITE: u32 = 0x4000_0000;
const OPEN_EXISTING: u32 = 3;

/// SDDL : propriétaire du fichier-objet (`OW`) seul autorisé en accès générique.
const SDDL_OWNER_ONLY: &str = "D:P(A;;GA;;;OW)";
/// SDDL : refus de tout accès à `Everyone` (`WD`).
const SDDL_DENY_EVERYONE: &str = "D:(D;;GA;;;WD)";
/// SDDL réaliste d'un service : le système et l'utilisateur interactif, rien d'autre.
///
/// C'est la forme que devra utiliser `obliniscord` : le service tourne en `LocalSystem`, donc
/// `OW` (le propriétaire) désigne **le système**, et l'utilisateur serait exclu. Il faut nommer
/// explicitement `SY` et `IU`.
const SDDL_SERVICE_AND_USER: &str = "D:P(A;;GA;;;SY)(A;;GA;;;IU)";

#[repr(C)]
struct SecurityAttributes {
    n_length: u32,
    security_descriptor: *mut c_void,
    inherit_handle: i32,
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
        sddl: *const u16,
        revision: u32,
        descriptor: *mut *mut c_void,
        size: *mut u32,
    ) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateNamedPipeW(
        name: *const u16,
        open_mode: u32,
        pipe_mode: u32,
        max_instances: u32,
        out_buffer: u32,
        in_buffer: u32,
        timeout: u32,
        attributes: *mut SecurityAttributes,
    ) -> Handle;
    fn CreateFileW(
        name: *const u16,
        access: u32,
        share: u32,
        security: *mut c_void,
        disposition: u32,
        flags: u32,
        template: *mut c_void,
    ) -> Handle;
    fn CloseHandle(handle: Handle) -> i32;
    fn GetLastError() -> u32;
    fn LocalFree(memory: *mut c_void) -> *mut c_void;
}

fn main() {
    // Nom unique : deux exécutions simultanées ne se marchent pas dessus.
    let stem = format!("obliniscor-spike-{}", std::process::id());
    println!("{{\"tool\":\"spike-service\",\"os\":\"windows\",\"experiments\":[");

    let allowed = experiment(&format!("{stem}-allowed"), SDDL_OWNER_ONLY, "propriétaire seul");
    println!("{allowed},");
    let denied = experiment(&format!("{stem}-denied"), SDDL_DENY_EVERYONE, "refus à Everyone");
    println!("{denied},");
    let service = experiment(
        &format!("{stem}-service"),
        SDDL_SERVICE_AND_USER,
        "système et utilisateur",
    );
    println!("{service}");

    println!("]}}");
}

/// Crée un canal avec le descripteur donné, tente de s'y connecter, puis nettoie.
fn experiment(name: &str, sddl: &str, label: &str) -> String {
    let descriptor = match security_descriptor(sddl) {
        Some(descriptor) => descriptor,
        None => {
            return format!(
                "{{\"case\":{label:?},\"error\":\"SDDL refusée (code {})\"}}",
                // SAFETY: aucun argument.
                unsafe { GetLastError() }
            );
        }
    };

    let mut attributes = SecurityAttributes {
        n_length: size_of::<SecurityAttributes>() as u32,
        security_descriptor: descriptor,
        inherit_handle: 0,
    };
    let pipe_name = format!(r"\\.\pipe\{name}");
    let wide_pipe = to_wide(&pipe_name);

    // SAFETY: `wide_pipe` est une chaîne UTF-16 terminée par NUL, `attributes` vit pendant
    // l'appel et son descripteur est valide.
    let server = unsafe {
        CreateNamedPipeW(
            wide_pipe.as_ptr(),
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
            1,
            4096,
            4096,
            0,
            &mut attributes,
        )
    };
    // SAFETY: le descripteur vient de la conversion SDDL et n'est plus utilisé par nous.
    unsafe { LocalFree(descriptor) };

    let created = server as isize != INVALID_HANDLE_VALUE;
    if !created {
        // SAFETY: aucun argument.
        let code = unsafe { GetLastError() };
        return format!(
            "{{\"case\":{label:?},\"sddl\":{sddl:?},\"created\":false,\"create_error\":{code}}}"
        );
    }

    // Le client ouvre le même nom, avec les droits d'écriture et de lecture.
    let client = unsafe {
        CreateFileW(
            wide_pipe.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            0,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        )
    };
    let connected = client as isize != INVALID_HANDLE_VALUE;
    // SAFETY: aucun argument ; lit le code d'erreur du fil courant.
    let code = unsafe { GetLastError() };

    if connected {
        // SAFETY: handle obtenu par CreateFileW, plus utilisé ensuite.
        unsafe { CloseHandle(client) };
    }
    // SAFETY: handle obtenu par CreateNamedPipeW, plus utilisé ensuite.
    unsafe { CloseHandle(server) };

    format!(
        "{{\"case\":{label:?},\"sddl\":{sddl:?},\"created\":true,\
         \"client_connected\":{connected},\"last_error\":{code}}}"
    )
}

/// Construit un descripteur de sécurité depuis une chaîne SDDL.
fn security_descriptor(sddl: &str) -> Option<*mut c_void> {
    let wide = to_wide(sddl);
    let mut descriptor: *mut c_void = std::ptr::null_mut();
    let mut size = 0u32;
    // SAFETY: `wide` est terminée par NUL ; `descriptor` et `size` reçoivent le résultat.
    let ok = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            wide.as_ptr(),
            1,
            &mut descriptor,
            &mut size,
        )
    };
    (ok != 0 && !descriptor.is_null()).then_some(descriptor)
}

fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
