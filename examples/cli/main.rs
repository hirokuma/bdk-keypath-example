mod wrapper;

use std::path::Path;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use tracing::*;

use wrapper::WalletWrapper;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create wallet
    Create,
    /// Balance
    Balance,
    /// Get addresses.
    Addrs,
    /// Get new address.
    #[command(name = "newaddr")]
    NewAddr,
    /// Decode transaction hex string.
    Tx {
        /// hex string to decode
        tx_hex: String,
    },
    /// Create a spendable transaction.
    Spend {
        /// output address
        out_addr: String,
        /// amount sats
        amount: u64,
        /// feerate
        fee_rate: f64,
    },
    /// Create a spendable transaction signed by SINGLE+ANYONE_CAN_PAY
    #[command(name = "spend-single")]
    SpendSingle {
        /// output address
        out_addr: String,
        /// amount sats
        amount: u64,
        /// feerate
        fee_rate: f64,
    },
    /// Send raw transaction.
    #[command(name = "sendrawtx")]
    SendRawTx { tx_hex: String },
    /// Remove wallet files
    RemoveWalletFiles,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    tracing::info!("bdk_wallet example");

    let cli = Cli::parse();

    let config = btc_wallet::load_config(Path::new("./config.toml"))
        .inspect_err(|e| error!("load_config: {e}"))?;
    let passphrase = "SuperSecurePassword123!";
    let wallet_path = Path::new("sample-wallet.bdk");

    match cli.command {
        None => {
            // clap will show help if user asks, but when no subcommand provided, print help
            Cli::command().print_help()?;
            println!();
        }
        Some(Commands::Create) => {
            let _wallet = WalletWrapper::create_wallet(
                config.network.to_string().as_str(),
                config.electrum.server.as_str(),
                passphrase,
                wallet_path.into(),
            )
            .inspect_err(|e| error!("create: {e}"))?;
            println!("wallet created");
        }
        Some(Commands::Balance) => {
            let wallet = WalletWrapper::load_wallet(passphrase, wallet_path.into())
                .inspect_err(|e| error!("load: {e}"))?;
            let balance = wallet.balance()?;
            println!("balance: {}", balance);
        }
        Some(Commands::Addrs) => {
            todo!();
        }
        Some(Commands::NewAddr) => {
            let mut wallet = WalletWrapper::load_wallet(passphrase, wallet_path.into())
                .inspect_err(|e| error!("load: {e}"))?;
            let new_addr = wallet.new_address()?;
            println!("new address: {}", new_addr);
        }
        Some(Commands::Tx { tx_hex }) => {
            let tx = btc_wallet::parse_tx_hex(&tx_hex).inspect_err(|e| error!("to_hex: {e}"))?;
            println!("{:#?}", tx);
        }
        Some(Commands::Spend {
            out_addr,
            amount,
            fee_rate,
        }) => {
            let mut wallet = WalletWrapper::load_wallet(passphrase, wallet_path.into())
                .inspect_err(|e| error!("load: {e}"))?;
            let result = wallet
                .send_tx(&out_addr, amount, fee_rate)
                .inspect_err(|e| error!("create_tx: {e}"))?;
            println!("raw_tx: {}", result.tx);
            println!("txid: {}", result.txid);
        }
        Some(Commands::SpendSingle {
            out_addr,
            amount,
            fee_rate,
        }) => {
            let mut wallet = WalletWrapper::load_wallet(passphrase, wallet_path.into())
                .inspect_err(|e| error!("load: {e}"))?;
            let result = wallet
                .send_tx_single_anypay(&out_addr, amount, fee_rate)
                .inspect_err(|e| error!("create_tx: {e}"))?;
            println!("raw_tx: {}", result.tx);
            println!("txid: {}", result.txid);
        }
        Some(Commands::SendRawTx { tx_hex }) => {
            let wallet = WalletWrapper::load_wallet(passphrase, wallet_path.into())
                .inspect_err(|e| error!("load: {e}"))?;
            let txid = wallet
                .send_raw_tx(&tx_hex)
                .inspect_err(|e| error!("send_tx: {e}"))?;
            println!("txid: {}", txid);
        }
        Some(Commands::RemoveWalletFiles) => {
            std::fs::remove_file(wallet_path)?;
            println!("remove: {}", wallet_path.to_string_lossy());
        }
    }

    Ok(())
}
