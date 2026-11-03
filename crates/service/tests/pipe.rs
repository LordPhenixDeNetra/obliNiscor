//! Aller-retour complet sur un vrai canal nommé : client, trame, service, réponse.
//!
//! Ces tests sont Windows seulement : le canal nommé est le transport de Windows. Sous Linux et
//! macOS, l'équivalent sera une socket Unix (P6b), et le test aura la même forme.
//!
//! Le serveur vit dans le fil de test et le client dans un fil séparé : c'est le moyen de
//! garantir que l'instance reste ouverte pendant que le client lit. Sans cela, le serveur
//! déconnecte avant la lecture et le client reçoit `ERROR_PIPE_NOT_CONNECTED` (233) — une course
//! que ce test a rencontrée avant d'être corrigée.

#![cfg(windows)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use obliniscor_core::MemorySnapshot;
use obliniscor_platform::fakes::{FakeMemory, FakePrivileges, FakeProcesses};
use obliniscor_proto as proto;
use obliniscor_proto::{MAX_FRAME, Message};
use obliniscor_service::Service;
use obliniscor_sys::windows::pipe::{PipeClient, PipeServer};

/// Nom unique par test : deux tests ne doivent pas se disputer le même canal.
fn pipe_name(suffix: &str) -> String {
    format!("obliniscor-t-{suffix}-{}", std::process::id())
}

fn service() -> Service<FakeMemory, FakePrivileges, FakeProcesses> {
    Service::new(
        FakeMemory::new(MemorySnapshot {
            total: 32_000,
            available: 12_000,
            free: Some(8_000),
        }),
        FakePrivileges::default(),
        FakeProcesses::new(),
    )
}

#[test]
fn a_status_request_makes_the_round_trip_over_the_pipe() {
    let name = pipe_name("status");
    let server = PipeServer::create(&name).expect("création du canal");

    // Le client part en premier : il réessaie jusqu'à ce qu'une instance l'accepte.
    let client_name = name.clone();
    let client = std::thread::spawn(move || {
        let client = PipeClient::connect(&client_name, 40).expect("connexion du client");
        let request = proto::encode(&Message::GetStatus).expect("encodage");
        client.write_frame(&request).expect("envoi");
        let response = client.read_frame(MAX_FRAME).expect("réception");
        proto::decode::<proto::Reply>(&response).expect("décodage de la réponse")
    });

    server.accept().expect("acceptation du client");
    obliniscor_service::pipe_server::serve_one(&server, &service()).expect("traitement");

    match client.join().expect("fin du client") {
        proto::Reply::Status {
            total,
            available,
            free,
        } => {
            assert_eq!(total, 32_000);
            assert_eq!(available, 12_000);
            assert_eq!(free, Some(8_000));
        }
        other => panic!("réponse inattendue : {other:?}"),
    }
    // La déconnexion vient après la lecture du client : le serveur ne lui coupe pas la parole.
    server.disconnect();
}

#[test]
fn an_unreadable_frame_is_answered_with_a_protocol_error() {
    let name = pipe_name("garbage");
    let server = PipeServer::create(&name).expect("création");

    let client_name = name.clone();
    let client = std::thread::spawn(move || {
        let client = PipeClient::connect(&client_name, 40).expect("connexion");
        // Trame bien formée, mais dont la charge n'est pas un message du protocole.
        let payload = b"pas du JSON";
        let mut frame = Vec::with_capacity(proto::HEADER_LEN + payload.len());
        frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        frame.extend_from_slice(payload);
        client.write_frame(&frame).expect("envoi");
        let response = client.read_frame(MAX_FRAME).expect("réception");
        proto::decode::<proto::Reply>(&response).expect("décodage")
    });

    server.accept().expect("acceptation");
    obliniscor_service::pipe_server::serve_one(&server, &service()).expect("traitement");

    match client.join().expect("fin du client") {
        proto::Reply::Failed { code } => assert_eq!(code, "E_PROTOCOL"),
        other => panic!("réponse inattendue : {other:?}"),
    }
    server.disconnect();
}
