//! État persistant d'obliNiscor : réglages, historique et règles.
//!
//! Ce crate est le seul qui **écrit** sur le disque de l'utilisateur, et il le fait par des
//! bibliothèques de la suite (`lpdnsoft-settings` pour les réglages, `lpdnsoft-atomic` pour
//! les écritures tout-ou-rien) plutôt qu'avec ses propres fichiers temporaires : un fichier de
//! réglages à moitié écrit est un bug qu'on ne voit qu'au prochain démarrage.
//!
//! Rien ici ne parle au système d'exploitation : pas de processus, pas de privilège, pas de
//! volume. Le stockage est une affaire de fichiers, et c'est tout (ADR-004).

pub mod settings;

pub use settings::{FILE_NAME, ObliNiscorSettings};
