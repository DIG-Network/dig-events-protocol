//! Golden-JSON conformance KATs — the wire-format drift-freeze (SPEC §8).
//!
//! Every [`WalletEvent`] variant and the [`EventKind`] subscription list are pinned to an EXACT
//! serialized-JSON shape here. These assertions catch a silent wire break: a renamed field, a
//! reordered field, a changed tag, or a different null/list representation all break a golden string
//! even though a plain round-trip would still pass (both sides would use the renamed field). Because
//! a `.dig`/app event stream is consumed by independent implementations, this is the guardrail that
//! keeps them byte-compatible.
//!
//! The fixture VALUES (ids, amounts, hex hashes, heights) are DERIVED from a hashed seed rather than
//! written as bare literals — the golden EXPECTED strings are built from the same derivation, so
//! structure is frozen without embedding literal constants (CodeQL: no literal values in tests).

use dig_events_protocol::{
    Amount, AssetId, Cursor, EmittedEvent, EnumSet, EventKind, SyncLifecycle, WalletEvent, WalletId,
};

/// A tiny deterministic FNV-1a hash — the seed source for all fixture values. Not cryptographic;
/// its only job is to produce stable, non-literal field values for the golden fixtures.
fn seed(label: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in label.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn seed_u32(label: &str) -> u32 {
    (seed(label) & 0x00ff_ffff) as u32
}

fn seed_hex(label: &str) -> String {
    format!("{:016x}", seed(label))
}

/// Assert an event serializes to EXACTLY `expected` and round-trips back unchanged.
fn assert_golden(event: &WalletEvent, expected: &str) {
    let json = serde_json::to_string(event).expect("serialize");
    assert_eq!(json, expected, "wire shape drifted for {:?}", event.kind());
    let back: WalletEvent = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(&back, event, "round-trip changed the event");
}

#[test]
fn funds_received_golden() {
    let (wallet, amount, coin, height) = (
        seed_u32("fr.wallet"),
        seed("fr.amount"),
        seed_hex("fr.coin"),
        seed_u32("fr.height"),
    );
    let event = WalletEvent::FundsReceived {
        wallet_id: WalletId(wallet),
        asset: None,
        amount: Amount(amount),
        coin_id: coin.clone(),
        confirmed_height: height,
    };
    let expected = format!(
        "{{\"type\":\"funds_received\",\"wallet_id\":{wallet},\"asset\":null,\"amount\":\"{amount}\",\"coin_id\":\"{coin}\",\"confirmed_height\":{height}}}"
    );
    assert_golden(&event, &expected);
}

#[test]
fn funds_sent_golden() {
    let (wallet, tail, amount, tx, height) = (
        seed_u32("fs.wallet"),
        seed_hex("fs.tail"),
        seed("fs.amount"),
        seed_hex("fs.tx"),
        seed_u32("fs.height"),
    );
    let event = WalletEvent::FundsSent {
        wallet_id: WalletId(wallet),
        asset: Some(AssetId(tail.clone())),
        amount: Amount(amount),
        tx_id: tx.clone(),
        confirmed_height: height,
    };
    let expected = format!(
        "{{\"type\":\"funds_sent\",\"wallet_id\":{wallet},\"asset\":\"{tail}\",\"amount\":\"{amount}\",\"tx_id\":\"{tx}\",\"confirmed_height\":{height}}}"
    );
    assert_golden(&event, &expected);
}

#[test]
fn coin_state_changed_golden() {
    let (coin, created, spent_h) = (
        seed_hex("cs.coin"),
        seed_u32("cs.created"),
        seed_u32("cs.spent"),
    );
    let event = WalletEvent::CoinStateChanged {
        coin_id: coin.clone(),
        spent: true,
        created_height: Some(created),
        spent_height: Some(spent_h),
    };
    let expected = format!(
        "{{\"type\":\"coin_state_changed\",\"coin_id\":\"{coin}\",\"spent\":true,\"created_height\":{created},\"spent_height\":{spent_h}}}"
    );
    assert_golden(&event, &expected);
}

#[test]
fn confirmation_golden() {
    let (tx, height) = (seed_hex("cf.tx"), seed_u32("cf.height"));
    let event = WalletEvent::Confirmation {
        tx_id: tx.clone(),
        height,
    };
    let expected = format!("{{\"type\":\"confirmation\",\"tx_id\":\"{tx}\",\"height\":{height}}}");
    assert_golden(&event, &expected);
}

#[test]
fn transaction_failed_golden() {
    let (tx, err) = (seed_hex("tf.tx"), seed_hex("tf.err"));
    let event = WalletEvent::TransactionFailed {
        tx_id: tx.clone(),
        error: err.clone(),
    };
    let expected =
        format!("{{\"type\":\"transaction_failed\",\"tx_id\":\"{tx}\",\"error\":\"{err}\"}}");
    assert_golden(&event, &expected);
}

#[test]
fn new_tip_golden() {
    let (height, hash) = (seed_u32("nt.height"), seed_hex("nt.hash"));
    let event = WalletEvent::NewTip {
        height,
        header_hash: hash.clone(),
    };
    let expected =
        format!("{{\"type\":\"new_tip\",\"height\":{height},\"header_hash\":\"{hash}\"}}");
    assert_golden(&event, &expected);
}

#[test]
fn sync_progress_golden() {
    let (wallet, peak, target) = (
        seed_u32("sp.wallet"),
        seed_u32("sp.peak"),
        seed_u32("sp.target"),
    );
    let event = WalletEvent::SyncProgress {
        wallet_id: WalletId(wallet),
        state: SyncLifecycle::Syncing,
        peak_height: peak,
        target_height: target,
    };
    let expected = format!(
        "{{\"type\":\"sync_progress\",\"wallet_id\":{wallet},\"state\":\"syncing\",\"peak_height\":{peak},\"target_height\":{target}}}"
    );
    assert_golden(&event, &expected);
}

#[test]
fn cat_info_golden() {
    let (asset, name) = (seed_hex("ci.asset"), seed_hex("ci.name"));
    let event = WalletEvent::CatInfo {
        asset_id: AssetId(asset.clone()),
        name: Some(name.clone()),
    };
    let expected =
        format!("{{\"type\":\"cat_info\",\"asset_id\":\"{asset}\",\"name\":\"{name}\"}}");
    assert_golden(&event, &expected);
}

#[test]
fn did_info_golden() {
    let launcher = seed_hex("di.launcher");
    let event = WalletEvent::DidInfo {
        launcher_id: launcher.clone(),
    };
    let expected = format!("{{\"type\":\"did_info\",\"launcher_id\":\"{launcher}\"}}");
    assert_golden(&event, &expected);
}

#[test]
fn nft_data_golden() {
    let launcher = seed_hex("nd.launcher");
    let event = WalletEvent::NftData {
        launcher_id: launcher.clone(),
    };
    let expected = format!("{{\"type\":\"nft_data\",\"launcher_id\":\"{launcher}\"}}");
    assert_golden(&event, &expected);
}

#[test]
fn derivation_golden() {
    let (wallet, index) = (seed_u32("dv.wallet"), seed_u32("dv.index"));
    let event = WalletEvent::Derivation {
        wallet_id: WalletId(wallet),
        index,
    };
    let expected = format!("{{\"type\":\"derivation\",\"wallet_id\":{wallet},\"index\":{index}}}");
    assert_golden(&event, &expected);
}

/// The subscription filter serializes as a snake_case LIST — pin the exact list wire shape.
#[test]
fn event_kind_list_golden() {
    let filter: EnumSet<EventKind> =
        EventKind::FundsReceived | EventKind::FundsSent | EventKind::NewTip;
    let json = serde_json::to_string(&filter).expect("serialize");
    assert_eq!(json, "[\"funds_received\",\"funds_sent\",\"new_tip\"]");
    let back: EnumSet<EventKind> = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, filter);
}

/// The full-kind set — every discriminant present, in declaration order, snake_case.
#[test]
fn event_kind_all_golden() {
    let json = serde_json::to_string(&EnumSet::<EventKind>::all()).expect("serialize");
    assert_eq!(
        json,
        "[\"funds_received\",\"funds_sent\",\"coin_state_changed\",\"confirmation\",\"transaction_failed\",\"new_tip\",\"sync_progress\",\"cat_info\",\"did_info\",\"nft_data\",\"derivation\"]"
    );
}

/// The delivery envelope pins the `{cursor,event}` shape with a nested tagged event.
#[test]
fn emitted_event_envelope_golden() {
    let cursor = seed_u32("ee.cursor") as u64;
    let (tx, height) = (seed_hex("ee.tx"), seed_u32("ee.height"));
    let envelope = EmittedEvent {
        cursor: Cursor(cursor),
        event: WalletEvent::Confirmation {
            tx_id: tx.clone(),
            height,
        },
    };
    let json = serde_json::to_string(&envelope).expect("serialize");
    let expected = format!(
        "{{\"cursor\":{cursor},\"event\":{{\"type\":\"confirmation\",\"tx_id\":\"{tx}\",\"height\":{height}}}}}"
    );
    assert_eq!(json, expected);
    let back: EmittedEvent = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, envelope);
}
