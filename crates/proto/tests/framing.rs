//! Tests du cadrage : ce qu'une trame acceptée doit rendre, et ce qu'elle doit faire refuser.
//!
//! Ils passent par l'API publique uniquement : c'est le contrat que le client et le service
//! partagent, et c'est lui qui doit résister à un pair hostile.

use obliniscor_proto::{
    HEADER_LEN, MAX_FRAME, Message, PROTOCOL_VERSION, ProtoError, decode, encode,
};

/// Décode en exigeant la réussite : un refus est un échec du test, avec sa cause en clair.
fn decode_ok(frame: &[u8]) -> Message {
    match decode::<Message>(frame) {
        Ok(message) => message,
        Err(error) => panic!("trame refusée : {error}"),
    }
}

/// Décode en exigeant le refus : une acceptation est un échec du test.
fn decode_err(frame: &[u8]) -> ProtoError {
    match decode::<Message>(frame) {
        Ok(message) => panic!("trame acceptée à tort : {message:?}"),
        Err(error) => error,
    }
}

/// Construit une trame à partir d'une charge arbitraire, pour éprouver des entrées que
/// `encode` ne produirait jamais.
fn build_frame(payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(HEADER_LEN + payload.len());
    frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    frame.extend_from_slice(payload);
    frame
}

/// Le préfixe seul d'une trame annonçant `declared` octets : c'est le strict minimum qu'un
/// décodeur lit avant de décider. Sert à éprouver le contrôle de taille sans jamais allouer ce
/// qu'il est censé interdire.
fn announced(declared: u32) -> Vec<u8> {
    declared.to_le_bytes().to_vec()
}

#[test]
fn every_variant_survives_a_round_trip() {
    let messages = [
        Message::GetStatus,
        Message::ListProcesses,
        Message::RunMemoryClean {
            zones: Vec::new(),
            dry_run: false,
        },
        Message::RunMemoryClean {
            zones: vec!["working-set".to_owned(), "standby".to_owned()],
            dry_run: true,
        },
        Message::Progress {
            done: 0,
            total: None,
        },
        Message::Progress {
            done: 512,
            total: Some(1024),
        },
    ];

    for message in messages {
        let frame = encode(&message).unwrap();
        assert_eq!(decode_ok(&frame), message, "aller-retour de {message:?}");
    }
}

#[test]
fn the_published_constants_are_pinned() {
    // Ces valeurs sont le contrat public du protocole : les changer rompt la compatibilité, donc
    // le test les fige explicitement (le cahier des charges impose 1 Mio, §8.2).
    assert_eq!(PROTOCOL_VERSION, 1);
    assert_eq!(MAX_FRAME, 1_048_576);
    assert_eq!(HEADER_LEN, 4);
}

#[test]
fn a_frame_starts_with_the_length_of_its_payload() {
    let frame = encode(&Message::GetStatus).unwrap();
    let declared = u32::from_le_bytes([frame[0], frame[1], frame[2], frame[3]]) as usize;

    assert_eq!(declared, frame.len() - HEADER_LEN);
}

#[test]
fn a_frame_announcing_two_mebibytes_is_refused_before_reading_anything() {
    // 2 Mio annoncés et zéro octet de charge : un décodeur qui allouerait d'après l'annonce
    // réserverait ces 2 Mio, et un attaquant en demanderait des gigaoctets.
    let frame = announced(2 * 1024 * 1024);
    let error = decode_err(&frame);

    assert!(
        matches!(
            &error,
            ProtoError::FrameTooLarge { declared, max }
                if *declared == 2 * 1024 * 1024 && *max == MAX_FRAME
        ),
        "refus inattendu : {error:?}"
    );

    // Le message nomme les deux tailles : c'est ce que lit l'utilisateur, et c'est ce qui
    // prouve que le refus vient du contrôle de taille et non d'une trame simplement courte.
    let text = error.to_string();
    assert!(text.contains("2097152"), "message inattendu : {text}");
    assert!(text.contains("1048576"), "message inattendu : {text}");
}

#[test]
fn one_byte_over_the_maximum_is_already_too_much() {
    let error = decode_err(&announced((MAX_FRAME as u32) + 1));

    assert!(
        matches!(
            &error,
            ProtoError::FrameTooLarge { declared, max }
                if *declared == MAX_FRAME + 1 && *max == MAX_FRAME
        ),
        "refus inattendu : {error:?}"
    );
}

#[test]
fn an_oversized_message_is_refused_instead_of_producing_an_unusable_frame() {
    // Une charge au-delà de `MAX_FRAME` doit être signalée à l'émetteur : renvoyer une trame
    // vide ferait envoyer au service une trame invalide, refusée plus tard par le lecteur.
    let message = Message::RunMemoryClean {
        zones: vec!["x".repeat(MAX_FRAME + 1)],
        dry_run: true,
    };

    let error = match encode(&message) {
        Ok(frame) => panic!("trame produite à tort : {} octet(s)", frame.len()),
        Err(error) => error,
    };

    assert!(
        matches!(
            &error,
            ProtoError::FrameTooLarge { declared, max }
                if *declared > MAX_FRAME && *max == MAX_FRAME
        ),
        "refus inattendu : {error:?}"
    );
}

#[test]
fn a_frame_of_exactly_the_maximum_size_is_accepted() {
    // Une trame de `MAX_FRAME` octets de charge : la limite est inclusive. On part d'un message
    // minimal dont on mesure la charge, puis on complète sa zone jusqu'à tomber pile dessus.
    let empty = Message::RunMemoryClean {
        zones: vec![String::new()],
        dry_run: false,
    };
    let base = encode(&empty).unwrap().len() - HEADER_LEN;
    let padding = MAX_FRAME - base;
    let message = Message::RunMemoryClean {
        zones: vec!["a".repeat(padding)],
        dry_run: false,
    };

    let frame = encode(&message).unwrap();
    assert_eq!(frame.len(), HEADER_LEN + MAX_FRAME);
    assert_eq!(
        u32::from_le_bytes([frame[0], frame[1], frame[2], frame[3]]) as usize,
        MAX_FRAME
    );
    assert_eq!(decode_ok(&frame), message);
}

#[test]
fn a_truncated_frame_is_refused() {
    let full = encode(&Message::Progress {
        done: 1,
        total: Some(4),
    })
    .unwrap();

    // Un octet de charge manque.
    let error = decode_err(&full[..full.len() - 1]);
    assert!(
        matches!(&error, ProtoError::TruncatedPayload { .. }),
        "refus inattendu : {error:?}"
    );

    // La longueur annonce plus que ce qui suit.
    let mut lying = announced(10);
    lying.extend_from_slice(b"{}");
    assert!(matches!(
        decode_err(&lying),
        ProtoError::TruncatedPayload {
            declared: 10,
            received: 2
        }
    ));

    // Moins que le préfixe : même la longueur manque.
    assert!(matches!(
        decode_err(&full[..HEADER_LEN - 1]),
        ProtoError::TruncatedHeader { received } if received == HEADER_LEN - 1
    ));
}

#[test]
fn an_unknown_field_is_tolerated_when_reading() {
    // Un pair plus récent peut ajouter `reason` : le refuser casserait la compatibilité
    // mineure promise par le cahier des charges.
    let payload = br#"{"RunMemoryClean":{"zones":["standby"],"dry_run":true,"reason":"pression"}}"#;

    assert_eq!(
        decode_ok(&build_frame(payload)),
        Message::RunMemoryClean {
            zones: vec!["standby".to_owned()],
            dry_run: true,
        }
    );

    // Même tolérance sur un autre message, avec un champ inconnu en plus d'un champ absent.
    let payload = br#"{"Progress":{"done":3,"phase":"nettoyage"}}"#;
    assert_eq!(
        decode_ok(&build_frame(payload)),
        Message::Progress {
            done: 3,
            total: None,
        }
    );
}

#[test]
fn a_missing_field_takes_its_default() {
    // L'autre sens de la compatibilité : un pair plus ancien n'envoie pas `dry_run`.
    let payload = br#"{"RunMemoryClean":{"zones":["standby"]}}"#;

    assert_eq!(
        decode_ok(&build_frame(payload)),
        Message::RunMemoryClean {
            zones: vec!["standby".to_owned()],
            dry_run: false,
        }
    );
}

#[test]
fn bytes_after_the_announced_payload_are_refused() {
    // Un flux désaligné ferait lire la trame suivante de travers : on refuse plutôt que de
    // sauter les octets en trop.
    let mut frame = encode(&Message::GetStatus).unwrap();
    frame.push(b' ');

    assert!(matches!(
        decode_err(&frame),
        ProtoError::TrailingBytes { extra: 1 }
    ));
}

#[test]
fn a_payload_that_is_not_a_protocol_message_is_refused() {
    assert!(matches!(
        decode_err(&build_frame(b"pas du json")),
        ProtoError::InvalidJson { .. }
    ));

    // JSON valide, mais pas un message connu du protocole.
    assert!(matches!(
        decode_err(&build_frame(br#"{"Unknown":{}}"#)),
        ProtoError::InvalidJson { .. }
    ));
}
