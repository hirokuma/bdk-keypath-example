// flutter_rust_bridgeで使うことを想定

use std::{path::PathBuf, str::FromStr};
use wallet_utils::encdec;

use btc_wallet::{self, BtcWallet};

pub struct WalletWrapper(BtcWallet);

pub struct SendResult {
    pub tx: String,
    pub txid: String,
}

impl WalletWrapper {
    pub fn create_wallet(
        network: &str,
        electrum_server: &str,
        passphrase: &str,
        wallet_path: PathBuf,
    ) -> anyhow::Result<Self> {
        let electrum_config = btc_wallet::ElectrumConfig {
            enabled: true,
            server: electrum_server.into(),
            batch_size: 30,
            gap_limit: 20,
        };
        let config = btc_wallet::Config {
            network: btc_wallet::Network::from_str(network).map_err(anyhow::Error::msg)?,
            electrum: electrum_config,
            backend: btc_wallet::Backend::Electrum,
        };
        let (wallet, xprv) = BtcWallet::create(config, &wallet_path)?;
        store_xprv(passphrase, &wallet_path, &xprv, network, electrum_server)?;
        Ok(WalletWrapper(wallet))
    }

    pub fn load_wallet(passphrase: &str, wallet_path: PathBuf) -> anyhow::Result<Self> {
        let (xprv, config) = load_xprv(passphrase, &wallet_path)?;
        let wallet = BtcWallet::load(config, &xprv, &wallet_path)?;
        Ok(WalletWrapper(wallet))
    }

    pub fn balance(&self) -> anyhow::Result<u64> {
        let balance = self.0.balance();
        Ok(balance.confirmed.to_sat())
    }

    pub fn new_address(&mut self) -> anyhow::Result<String> {
        let address = self.0.new_address()?;
        Ok(address.to_string())
    }

    pub fn send_tx(
        &mut self,
        out_addr: &str,
        amount: u64,
        fee_rate: f64,
    ) -> anyhow::Result<SendResult> {
        let address = self.0.parse_address(out_addr)?;
        let tx = self.0.create_tx(&address, amount, fee_rate)?;
        let txid = self.0.send_tx(&tx)?;
        Ok(SendResult {
            tx: btc_wallet::to_tx_hex(&tx),
            txid: txid.to_string(),
        })
    }

    pub fn send_tx_single_anypay(
        &mut self,
        out_addr: &str,
        amount: u64,
        fee_rate: f64,
    ) -> anyhow::Result<SendResult> {
        let address = self.0.parse_address(out_addr)?;
        let tx = self.0.create_tx_single_anypay(&address, amount, fee_rate)?;
        let txid = self.0.send_tx(&tx)?;
        Ok(SendResult {
            tx: btc_wallet::to_tx_hex(&tx),
            txid: txid.to_string(),
        })
    }

    pub fn send_raw_tx(&self, tx_hex: &str) -> anyhow::Result<String> {
        let tx = btc_wallet::parse_tx_hex(tx_hex)?;
        let txid = self.0.send_tx(&tx)?;
        Ok(txid.to_string())
    }
}

fn store_xprv(
    passphrase: &str,
    wallet_path: &PathBuf,
    xprv: &str,
    network: &str,
    electrum_server: &str,
) -> anyhow::Result<()> {
    let v = encdec::encode_private_key(xprv, passphrase)?;

    let conn = rusqlite::Connection::open(wallet_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS wallet_key(
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            private_key BLOB NOT NULL,
            network TEXT NOT NULL,
            electrum_server TEXT NOT NULL)
        ",
        (),
    )?;
    conn.execute(
        "INSERT INTO wallet_key (private_key, network, electrum_server) VALUES (?1, ?2, ?3)",
        (v, network, electrum_server),
    )?;
    Ok(())
}

fn load_xprv(
    passphrase: &str,
    wallet_path: &PathBuf,
) -> anyhow::Result<(String, btc_wallet::Config)> {
    let conn = rusqlite::Connection::open(wallet_path)?;
    let result: (Vec<u8>, String, String) = conn.query_row(
        "SELECT private_key, network, electrum_server FROM wallet_key LIMIT 1",
        (),
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    let xprv = encdec::decode_private_key(&result.0, passphrase)?;
    let config = btc_wallet::Config {
        network: btc_wallet::Network::from_str(&result.1).map_err(anyhow::Error::msg)?,
        electrum: btc_wallet::ElectrumConfig {
            enabled: true,
            server: result.2,
            batch_size: 30,
            gap_limit: 20,
        },
        backend: btc_wallet::Backend::Electrum,
    };
    Ok((xprv, config))
}
