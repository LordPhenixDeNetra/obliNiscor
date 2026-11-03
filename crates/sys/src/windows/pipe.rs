//! Canal nommé Windows : le transport du service et de ses clients.
//!
//! Le canal porte des **trames** de `obliniscor-proto` (longueur `u32` little-endian puis JSON).
//! Ce module ne connaît pas le protocole : il lit et écrit des octets, et refuse une annonce
//! au-delà de la borne qu'on lui donne. La validation du contenu reste au-dessus.
//!
//! Le descripteur de sécurité vient de l'ADR et du spike OBL-008 : `SY` (système local) pour le
//! service et `IU` (utilisateur interactif) pour la GUI et la CLI. `OW` désignerait le système
//! quand le service tourne en `LocalSystem`, et l'utilisateur serait exclu — c'est l'erreur que
//! le spike a mesurée avant qu'elle n'arrive.
//!
//! Toutes les fonctions bloquent : c'est un service, il attend ses clients.

use obliniscor_platform::PlatformError;
use std::ffi::c_void;

type Handle = *mut c_void;
const INVALID_HANDLE_VALUE: isize = -1;

const PIPE_ACCESS_DUPLEX: u32 = 0x0000_0003;
const PIPE_TYPE_BYTE: u32 = 0x0000_0000;
const PIPE_READMODE_BYTE: u32 = 0x0000_0000;
const PIPE_WAIT: u32 = 0x0000_0000;
const GENERIC_READ: u32 = 0x8000_0000;
const GENERIC_WRITE: u32 = 0x4000_0000;
const OPEN_EXISTING: u32 = 3;
const ERROR_PIPE_CONNECTED: u32 = 535;
const ERROR_BROKEN_PIPE: u32 = 109;

/// Descripteur du canal : le service et l'utilisateur interactif, rien d'autre.
pub const SDDL_SERVICE_AND_USER: &str = "D:P(A;;GA;;;SY)(A;;GA;;;IU)";

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
    fn ConnectNamedPipe(handle: Handle, overlapped: *mut c_void) -> i32;
    fn DisconnectNamedPipe(handle: Handle) -> i32;
    fn ReadFile(
        handle: Handle,
        buffer: *mut c_void,
        to_read: u32,
        read: *mut u32,
        overlapped: *mut c_void,
    ) -> i32;
    fn WriteFile(
        handle: Handle,
        buffer: *const c_void,
        to_write: u32,
        written: *mut u32,
        overlapped: *mut c_void,
    ) -> i32;
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

/// Côté serveur : une instance de canal, acceptant un client à la fois.
pub struct PipeServer {
    handle: Handle,
}

// SAFETY: le handle est possédé par la structure, et le canal nommé est utilisable depuis un
// autre fil (c'est même son usage : le service bloque sur l'acceptation pendant que la boucle
// principale vit sa vie).
unsafe impl Send for PipeServer {}

impl PipeServer {
    /// Crée une instance de canal nommé avec le descripteur du service.
    ///
    /// `name` est le nom court (`obliniscord`), sans le préfixe `\\.\pipe\`.
    pub fn create(name: &str) -> Result<Self, PlatformError> {
        let mut descriptor: *mut c_void = std::ptr::null_mut();
        let mut size = 0u32;
        let sddl = to_wide(SDDL_SERVICE_AND_USER);
        // SAFETY: `sddl` est une chaîne UTF-16 terminée par NUL ; les deux pointeurs de sortie
        // désignent des variables locales vivantes.
        let converted = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                1,
                &mut descriptor,
                &mut size,
            )
        };
        if converted == 0 || descriptor.is_null() {
            // SAFETY: aucun argument.
            let code = unsafe { GetLastError() };
            return Err(PlatformError::Failed(format!(
                "descripteur de sécurité refusé (erreur {code})"
            )));
        }

        let mut attributes = SecurityAttributes {
            n_length: size_of::<SecurityAttributes>() as u32,
            security_descriptor: descriptor,
            inherit_handle: 0,
        };
        let path = to_wide(&format!(r"\\.\pipe\{name}"));
        // SAFETY: `path` est terminée par NUL, `attributes` vit pendant l'appel et son
        // descripteur est valide.
        let handle = unsafe {
            CreateNamedPipeW(
                path.as_ptr(),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                1,
                64 * 1024,
                64 * 1024,
                0,
                &mut attributes,
            )
        };
        // SAFETY: le descripteur vient de la conversion SDDL et n'est plus utilisé.
        unsafe { LocalFree(descriptor) };

        if handle as isize == INVALID_HANDLE_VALUE {
            // SAFETY: aucun argument.
            let code = unsafe { GetLastError() };
            return Err(PlatformError::Failed(format!(
                "création du canal impossible (erreur {code})"
            )));
        }
        Ok(Self { handle })
    }

    /// Attend un client. Bloque jusqu'à sa connexion.
    pub fn accept(&self) -> Result<(), PlatformError> {
        // SAFETY: le handle est valide tant que `self` vit ; pas d'E/S superposée.
        let connected = unsafe { ConnectNamedPipe(self.handle, std::ptr::null_mut()) };
        if connected != 0 {
            return Ok(());
        }
        // SAFETY: aucun argument.
        let code = unsafe { GetLastError() };
        // Un client déjà connecté n'est pas une erreur : c'est une course bénigne entre
        // `CreateFile` et `ConnectNamedPipe`.
        if code == ERROR_PIPE_CONNECTED {
            return Ok(());
        }
        Err(PlatformError::Failed(format!(
            "connexion du client impossible (erreur {code})"
        )))
    }

    ///
    /// `max_frame` borne la charge acceptée **avant** toute allocation : une annonce démesurée
    /// est refusée sur la foi des quatre octets d'en-tête.
    pub fn read_frame(&self, max_frame: usize) -> Result<Vec<u8>, PlatformError> {
        let mut header = [0u8; 4];
        self.read_exact(&mut header)?;
        let declared = u32::from_le_bytes(header) as usize;
        if declared > max_frame {
            return Err(PlatformError::Failed(format!(
                "trame annoncée de {declared} octets, au-delà de la limite de {max_frame}"
            )));
        }
        let mut frame = vec![0u8; 4 + declared];
        frame[..4].copy_from_slice(&header);
        self.read_exact(&mut frame[4..])?;
        Ok(frame)
    }

    /// Écrit des octets tels quels : le cadrage appartient à obliniscor-proto.
    pub fn write_frame(&self, frame: &[u8]) -> Result<(), PlatformError> {
        self.write_all(frame)
    }

    /// Termine la connexion courante, en gardant l'instance pour le client suivant.
    pub fn disconnect(&self) {
        // SAFETY: handle valide ; l'échec n'apporte rien à l'appelant.
        unsafe { DisconnectNamedPipe(self.handle) };
    }

    fn read_exact(&self, buffer: &mut [u8]) -> Result<(), PlatformError> {
        let mut filled = 0usize;
        while filled < buffer.len() {
            let mut read = 0u32;
            let remaining = buffer.len() - filled;
            // SAFETY: le pointeur désigne `remaining` octets encore libres dans `buffer`.
            let ok = unsafe {
                ReadFile(
                    self.handle,
                    buffer[filled..].as_mut_ptr().cast(),
                    remaining as u32,
                    &mut read,
                    std::ptr::null_mut(),
                )
            };
            if ok == 0 {
                // SAFETY: aucun argument.
                let code = unsafe { GetLastError() };
                if code == ERROR_BROKEN_PIPE {
                    return Err(PlatformError::Denied(
                        "le client a fermé le canal".to_owned(),
                    ));
                }
                return Err(PlatformError::Failed(format!(
                    "lecture du canal impossible (erreur {code})"
                )));
            }
            if read == 0 {
                return Err(PlatformError::Failed("lecture vide du canal".to_owned()));
            }
            filled += read as usize;
        }
        Ok(())
    }

    fn write_all(&self, bytes: &[u8]) -> Result<(), PlatformError> {
        let mut written_total = 0usize;
        while written_total < bytes.len() {
            let mut written = 0u32;
            // SAFETY: le pointeur désigne `bytes.len() - written_total` octets lisibles.
            let ok = unsafe {
                WriteFile(
                    self.handle,
                    bytes[written_total..].as_ptr().cast(),
                    (bytes.len() - written_total) as u32,
                    &mut written,
                    std::ptr::null_mut(),
                )
            };
            if ok == 0 {
                // SAFETY: aucun argument.
                let code = unsafe { GetLastError() };
                return Err(PlatformError::Failed(format!(
                    "écriture dans le canal impossible (erreur {code})"
                )));
            }
            written_total += written as usize;
        }
        Ok(())
    }
}

impl Drop for PipeServer {
    fn drop(&mut self) {
        // SAFETY: le handle vient de `CreateNamedPipeW` et n'est plus utilisé après.
        unsafe { CloseHandle(self.handle) };
    }
}

/// Côté client : une connexion à un canal existant.
pub struct PipeClient {
    handle: Handle,
}

// SAFETY: le handle est possédé par la structure et un handle de fichier est utilisable depuis
// un autre fil.
unsafe impl Send for PipeClient {}

impl PipeClient {
    /// Se connecte au canal nommé, en attendant qu'une instance soit disponible.
    pub fn connect(name: &str, attempts: u32) -> Result<Self, PlatformError> {
        let path = to_wide(&format!(r"\\.\pipe\{name}"));
        let mut last = 0;
        for _ in 0..attempts.max(1) {
            // SAFETY: `path` est terminée par NUL ; les autres pointeurs sont nuls.
            let handle = unsafe {
                CreateFileW(
                    path.as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    0,
                    std::ptr::null_mut(),
                    OPEN_EXISTING,
                    0,
                    std::ptr::null_mut(),
                )
            };
            if handle as isize != INVALID_HANDLE_VALUE {
                return Ok(Self { handle });
            }
            // SAFETY: aucun argument.
            last = unsafe { GetLastError() };
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        Err(PlatformError::Failed(format!(
            "connexion au canal {name} impossible (erreur {last})"
        )))
    }

    /// Écrit des octets tels quels : le cadrage appartient à obliniscor-proto.
    pub fn write_frame(&self, frame: &[u8]) -> Result<(), PlatformError> {
        write_all(self.handle, frame)
    }

    /// Lit une trame complète.
    pub fn read_frame(&self, max_frame: usize) -> Result<Vec<u8>, PlatformError> {
        let mut header = [0u8; 4];
        read_exact(self.handle, &mut header)?;
        let declared = u32::from_le_bytes(header) as usize;
        if declared > max_frame {
            return Err(PlatformError::Failed(format!(
                "trame annoncée de {declared} octets, au-delà de la limite de {max_frame}"
            )));
        }
        let mut frame = vec![0u8; 4 + declared];
        frame[..4].copy_from_slice(&header);
        read_exact(self.handle, &mut frame[4..])?;
        Ok(frame)
    }
}

impl Drop for PipeClient {
    fn drop(&mut self) {
        // SAFETY: le handle vient de `CreateFileW` et n'est plus utilisé après.
        unsafe { CloseHandle(self.handle) };
    }
}

fn read_exact(handle: Handle, buffer: &mut [u8]) -> Result<(), PlatformError> {
    let mut filled = 0usize;
    while filled < buffer.len() {
        let mut read = 0u32;
        let remaining = buffer.len() - filled;
        // SAFETY: le pointeur désigne `remaining` octets libres dans `buffer`.
        let ok = unsafe {
            ReadFile(
                handle,
                buffer[filled..].as_mut_ptr().cast(),
                remaining as u32,
                &mut read,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 || read == 0 {
            // SAFETY: aucun argument.
            let code = unsafe { GetLastError() };
            return Err(PlatformError::Failed(format!(
                "lecture du canal impossible (erreur {code})"
            )));
        }
        filled += read as usize;
    }
    Ok(())
}

fn write_all(handle: Handle, bytes: &[u8]) -> Result<(), PlatformError> {
    let mut written_total = 0usize;
    while written_total < bytes.len() {
        let mut written = 0u32;
        // SAFETY: le pointeur désigne `bytes.len() - written_total` octets lisibles.
        let ok = unsafe {
            WriteFile(
                handle,
                bytes[written_total..].as_ptr().cast(),
                (bytes.len() - written_total) as u32,
                &mut written,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            // SAFETY: aucun argument.
            let code = unsafe { GetLastError() };
            return Err(PlatformError::Failed(format!(
                "écriture dans le canal impossible (erreur {code})"
            )));
        }
        written_total += written as usize;
    }
    Ok(())
}

fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
