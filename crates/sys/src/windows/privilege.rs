//! Activation temporaire de privilèges — la première vraie entrée d'`unsafe` du projet.
//!
//! Le principe tient en une phrase : **un privilège activé est rendu quoi qu'il arrive**. Il
//! n'y a donc pas de `release()` à appeler, seulement un objet qui tombe — à la fin du bloc,
//! sur un `?`, sur une panique. Un appelant ne peut pas oublier de le rendre.
//!
//! La séquence vient de la documentation Microsoft (`AdjustTokenPrivileges`) ; seul le
//! comportement s'observe, aucun code de projet tiers n'est recopié (ADR-001).
//!
//! **Sous Linux et macOS, ce module n'a pas d'équivalent** : il n'existe pas de privilège
//! activable à la demande, mais un droit du système de fichiers ou une capacité, obtenu au
//! lancement. C'est l'ADR-009 en pratique : la fonction est déclarée indisponible, pas
//! simulée.

use obliniscor_core::Privilege;
use obliniscor_platform::{EnabledPrivileges, PlatformError, Privileges};
use std::ffi::c_void;

type Handle = *mut c_void;

const TOKEN_ADJUST_PRIVILEGES: u32 = 0x0020;
const TOKEN_QUERY: u32 = 0x0008;
const SE_PRIVILEGE_ENABLED: u32 = 0x0002;
/// `ERROR_NOT_ALL_ASSIGNED` : le jeton ne porte pas le privilège demandé.
const ERROR_NOT_ALL_ASSIGNED: u32 = 1300;

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Luid {
    low: u32,
    high: i32,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct LuidAndAttributes {
    luid: Luid,
    attributes: u32,
}

#[repr(C)]
struct TokenPrivileges {
    count: u32,
    privileges: [LuidAndAttributes; 1],
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn OpenProcessToken(process: Handle, access: u32, token: *mut Handle) -> i32;
    fn LookupPrivilegeValueW(system: *const u16, name: *const u16, luid: *mut Luid) -> i32;
    fn AdjustTokenPrivileges(
        token: Handle,
        disable_all: i32,
        new_state: *const TokenPrivileges,
        buffer_length: u32,
        previous: *mut c_void,
        return_length: *mut u32,
    ) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> Handle;
    fn CloseHandle(handle: Handle) -> i32;
    fn GetLastError() -> u32;
}

/// Les privilèges activés par cet objet, rendus à sa chute.
pub struct TokenPrivilegesGuard {
    token: Handle,
    enabled: Vec<Privilege>,
}

impl EnabledPrivileges for TokenPrivilegesGuard {
    fn enabled(&self) -> &[Privilege] {
        &self.enabled
    }
}

impl Drop for TokenPrivilegesGuard {
    fn drop(&mut self) {
        // Rendre chaque privilège, puis fermer le jeton. L'échec de la restitution ne peut pas
        // être signalé (on est dans `Drop`, et `Drop` ne rend rien) : il est ignoré pour ne pas
        // transformer une fin de portée en panique. La session se termine avec le processus.
        for privilege in std::mem::take(&mut self.enabled) {
            if let Some((state, _)) = lookup(privilege) {
                let token_state = state_for(state, 0);
                // SAFETY: `token_state` désigne une structure de la taille annoncée, et
                // `self.token` est un jeton valide jusqu'à `CloseHandle` ci-dessous.
                unsafe {
                    AdjustTokenPrivileges(
                        self.token,
                        0,
                        &token_state,
                        0,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    );
                }
            }
        }
        // SAFETY: `self.token` vient d'`OpenProcessToken` et n'est plus utilisé.
        unsafe { CloseHandle(self.token) };
    }
}

/// Implémentation réelle de la capacité, sans état.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemPrivileges;

impl Privileges for SystemPrivileges {
    fn acquire(
        &self,
        privileges: &[Privilege],
    ) -> Result<Box<dyn EnabledPrivileges>, PlatformError> {
        if privileges.is_empty() {
            return Ok(Box::new(TokenPrivilegesGuard {
                token: std::ptr::null_mut(),
                enabled: Vec::new(),
            }));
        }

        let mut token: Handle = std::ptr::null_mut();
        // SAFETY: `token` est un pointeur valide vers un handle, rempli par l'appel.
        let opened = unsafe {
            OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                &mut token,
            )
        };
        if opened == 0 {
            // SAFETY: aucun argument.
            let code = unsafe { GetLastError() };
            return Err(PlatformError::Failed(format!(
                "OpenProcessToken a échoué (erreur {code})"
            )));
        }

        let mut enabled = Vec::with_capacity(privileges.len());
        for privilege in privileges {
            let Some((state, _)) = lookup(*privilege) else {
                disable_all(token, &enabled);
                // SAFETY: jeton valide, plus utilisé ensuite.
                unsafe { CloseHandle(token) };
                return Err(PlatformError::Failed(format!(
                    "{} est inconnu du système",
                    privilege.as_str()
                )));
            };
            let token_state = state_for(state, SE_PRIVILEGE_ENABLED);

            // SAFETY: structure de la taille annoncée, jeton valide.
            let adjusted = unsafe {
                AdjustTokenPrivileges(
                    token,
                    0,
                    &token_state,
                    0,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            };
            // SAFETY: aucun argument.
            let code = unsafe { GetLastError() };
            // `AdjustTokenPrivileges` rend `TRUE` même quand il n'a rien accordé : le seul
            // moyen de le savoir est `ERROR_NOT_ALL_ASSIGNED`. C'est le piège de cet appel.
            if adjusted == 0 || code == ERROR_NOT_ALL_ASSIGNED {
                disable_all(token, &enabled);
                // SAFETY: jeton valide, plus utilisé ensuite.
                unsafe { CloseHandle(token) };
                return Err(PlatformError::MissingPrivilege(*privilege));
            }
            enabled.push(*privilege);
        }

        Ok(Box::new(TokenPrivilegesGuard { token, enabled }))
    }
}

/// Rend un privilège déjà activé, sur un jeton encore ouvert.
fn disable_all(token: Handle, enabled: &[Privilege]) {
    for privilege in enabled {
        if let Some((state, _)) = lookup(*privilege) {
            let token_state = state_for(state, 0);
            // SAFETY: structure de la taille annoncée, jeton valide pendant l'appel.
            unsafe {
                AdjustTokenPrivileges(
                    token,
                    0,
                    &token_state,
                    0,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
            }
        }
    }
}

/// Traduit un nom de privilège en LUID local.
fn lookup(privilege: Privilege) -> Option<(Luid, ())> {
    let name = to_wide(privilege.as_str());
    let mut luid = Luid::default();
    // SAFETY: `name` est une chaîne UTF-16 terminée par NUL ; `luid` reçoit le résultat.
    let ok = unsafe { LookupPrivilegeValueW(std::ptr::null(), name.as_ptr(), &mut luid) };
    (ok != 0).then_some((luid, ()))
}

fn state_for(luid: Luid, attributes: u32) -> TokenPrivileges {
    TokenPrivileges {
        count: 1,
        privileges: [LuidAndAttributes { luid, attributes }],
    }
}

fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
