use std::path::PathBuf;
use wallet_utils::encdec;

use btc_wallet::{self, BtcWallet, Config};

pub fn create_wallet(
    config: Config,
    passphrase: &str,
    wallet_path: PathBuf,
) -> anyhow::Result<BtcWallet> {
    let (wallet, xprv) = BtcWallet::create(config, &wallet_path)?;
    store_xprv(passphrase, &wallet_path, &xprv)?;
    Ok(wallet)
}

pub fn load_wallet(
    config: Config,
    passphrase: &str,
    wallet_path: PathBuf,
) -> anyhow::Result<BtcWallet> {
    let xprv = load_xprv(passphrase, &wallet_path)?;
    Ok(BtcWallet::load(config, &xprv, &wallet_path)?)
}

fn store_xprv(passphrase: &str, wallet_path: &PathBuf, xprv: &str) -> anyhow::Result<()> {
    let v = encdec::encode_private_key(xprv, passphrase)?;

    let conn = rusqlite::Connection::open(wallet_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS wallet_key(
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            private_key BLOB NOT NULL)
        ",
        (),
    )?;
    conn.execute("INSERT INTO wallet_key (private_key) VALUES (?1)", (v,))?;
    Ok(())
}

fn load_xprv(passphrase: &str, wallet_path: &PathBuf) -> anyhow::Result<String> {
    let conn = rusqlite::Connection::open(wallet_path)?;
    let xprv: Vec<u8> =
        conn.query_row("SELECT private_key FROM wallet_key LIMIT 1", (), |row| {
            row.get(0)
        })?;
    let xprv = encdec::decode_private_key(&xprv, passphrase)?;
    Ok(xprv)
}
