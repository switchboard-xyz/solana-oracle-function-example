// Note: EtherPrices API requires a non-US IP address

use crate::*;

use switchboard_solana::get_ixn_discriminator;
use usdy_usd_oracle::{OracleDataBorsh, TradingSymbol, OracleDataWithTradingSymbol, RefreshOraclesParams};
use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Ticker {
    pub symbol: String, // BTCUSDT
    pub mean: I256,  // 0.00000000
    pub median: I256, // 0.00000000
    pub std: I256, // 0.00000000
}

#[derive(Clone, Debug)]
pub struct IndexData {
    pub symbol: String,
    pub data: Ticker,
}
impl TryInto<OracleDataBorsh> for IndexData {
    
    type Error = SbError;

    fn try_into(self) -> Result<OracleDataBorsh, Self::Error> {
        let oracle_timestamp: i64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| {
                SbError::CustomMessage("Invalid oracle_timestamp".to_string())
            })?
            .as_secs()
            .try_into()
            .map_err(|_| {
                SbError::CustomMessage("Invalid oracle_timestamp".to_string())
            })?;

            switchboard_solana::Result::Ok(OracleDataBorsh {
                oracle_timestamp,
                mean: self.data.mean.as_u64(),
                median: self.data.median.as_u64(),
                std: self.data.std.as_u64(),

            })
    }
}

pub struct EtherPrices {
    pub usdy_usd: IndexData,
}

impl EtherPrices {

    // Fetch data from the EtherPrices API
    pub async fn fetch(mean:  ethers::types::U256, median:  ethers::types::U256, std:  ethers::types::U256) -> std::result::Result<EtherPrices, SbError> {
        let symbols = ["USDYUSD"];
        let mean: I256 = mean.try_into().map_err(|_| {
            SbError::CustomMessage("Invalid mean".to_string())
        }).unwrap();
        let median: I256 = median.try_into().map_err(|_| {
            SbError::CustomMessage("Invalid median".to_string())
        }).unwrap();
        let std: I256 = std.try_into().map_err(|_| {
            SbError::CustomMessage("Invalid std".to_string())
        }).unwrap();
        Ok(EtherPrices {
            usdy_usd: {
                let symbol = symbols[0];
                
                IndexData {
                    symbol: symbol.to_string(),
                    data: Ticker {
                        symbol: symbol.to_string(),
                        mean,
                        median,
                        std,
                    
                    }
                }
            }
        })
    }

    pub fn to_ixns(&self, runner: &FunctionRunner) -> Vec<Instruction> {
        let rows: Vec<OracleDataWithTradingSymbol> = vec![
            OracleDataWithTradingSymbol {
                symbol: TradingSymbol::Usdy_usdc,
                data: self.usdy_usd.clone().try_into().map_err(|_| {
                    SbError::CustomMessage("Invalid oracle data".to_string())
                }).unwrap(),
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

