// Note: EtherPrices API requires a non-US IP address

use crate::*;

use switchboard_solana::get_ixn_discriminator;
use usdy_usd_oracle::{OracleDataBorsh, TradingSymbol, OracleDataWithTradingSymbol, RefreshOraclesParams};
use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Ticker {
    pub symbol: String, // BTCUSDT
    pub price: I256,  // 0.00000000
}

#[derive(Clone, Debug)]
pub struct IndexData {
    pub symbol: String,
    pub data: Ticker,
}
impl Into<OracleDataBorsh> for IndexData {
    fn into(self) -> OracleDataBorsh {
        let oracle_timestamp: i64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .try_into()
            .unwrap_or_default();

        OracleDataBorsh {
            oracle_timestamp,
            price: self.data.price.as_u64()
        }
    }
}

pub struct EtherPrices {
    pub usdy_usd: IndexData,
}

impl EtherPrices {

    // Fetch data from the EtherPrices API
    pub async fn fetch(price:  ethers::types::U256) -> std::result::Result<EtherPrices, SbError> {
        let symbols = ["USDYUSD"];
        let price: I256 = price.try_into().unwrap_or_default();

        Ok(EtherPrices {
            usdy_usd: {
                let symbol = symbols[0];
                
                IndexData {
                    symbol: symbol.to_string(),
                    data: Ticker {
                        symbol: symbol.to_string(),
                        price: price.try_into().unwrap_or_default(),
                    
                    }
                }
            }
        })
    }

    pub fn to_ixns(&self, runner: &FunctionRunner) -> Vec<Instruction> {
        let rows: Vec<OracleDataWithTradingSymbol> = vec![
            OracleDataWithTradingSymbol {
                symbol: TradingSymbol::Usdy_usdc,
                data: self.usdy_usd.clone().into(),
            }
            // OracleDataWithTradingSymbol {
            // symbol: TradingSymbol::Sol,
            // data: self.sol_usdt.clone().into(),
            // },
            // OracleDataWithTradingSymbol {
            // symbol: TradingSymbol::Doge,
            // data: self.doge_usdt.clone().into(),
            // },
        ];
        println!("{}, {}", self.usdy_usd.data.price, TradingSymbol::Usdy_usdc as u8);

        let params = RefreshOraclesParams { rows };

        let (program_state_pubkey, _state_bump) =
            Pubkey::find_program_address(&[b"USDY_USDC_ORACLE"], &usdy_usd_oracle::ID);

        let (oracle_pubkey, _oracle_bump) =
            Pubkey::find_program_address(&[b"ORACLE_USDY_SEED"], &usdy_usd_oracle::ID);

        let ixn = Instruction {
            program_id: usdy_usd_oracle::ID,
            accounts: vec![
                AccountMeta {
                    pubkey: program_state_pubkey,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: oracle_pubkey,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: runner.function,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: runner.signer,
                    is_signer: true,
                    is_writable: false,
                },
            ],
            data: [
                get_ixn_discriminator("refresh_oracles").to_vec(),
                params.try_to_vec().unwrap(),
            ]
            .concat(),
        };
        vec![ixn]
    }
}

