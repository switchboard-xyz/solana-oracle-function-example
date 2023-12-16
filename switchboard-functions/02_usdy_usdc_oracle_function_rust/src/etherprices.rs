// Note: EtherPrices API requires a non-US IP address

use crate::*;

use switchboard_solana::get_ixn_discriminator;
use usdy_usd_oracle::{OracleDataBorsh, TradingSymbol, OracleDataWithTradingSymbol, RefreshOraclesParams};
use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Ticker {
    pub symbol: String, // BTCUSDT
    pub ondo_price: u128,
    pub traded_price: u128
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
                ondo_price: self.data.ondo_price as u64,
                traded_price: self.data.traded_price as u64,

            })
    }
}

pub struct EtherPrices {
    pub usdy_usd: IndexData,
}

impl EtherPrices {

    // Fetch data from the EtherPrices API
    pub async fn fetch(ondo_price:  ethers::types::U256, traded_price:  ethers::types::U256) -> std::result::Result<EtherPrices, SbError> {
        let symbols = ["USDYUSD"];
        let ondo_price = ondo_price.as_u128();
        let traded_price = traded_price.as_u128();
        println!("ondo_price: {:?}", ondo_price);
        println!("traded_price: {:?}", traded_price);

        Ok(EtherPrices {
            usdy_usd: {
                let symbol = symbols[0];
                println!("symbol: {:?}", symbol);
                IndexData {
                    symbol: symbol.to_string(),
                    data: Ticker {
                        symbol: symbol.to_string(),
                        ondo_price: ondo_price,
                        traded_price: traded_price,
                    
                    }
                }
            }
        })
    }

    pub fn to_ixns(&self, runner: &FunctionRunner) -> Vec<Instruction> {
        println!("to_ixns");
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
        println!("2");
        let params = RefreshOraclesParams { rows };

        let (program_state_pubkey, _state_bump) =
            Pubkey::find_program_address(&[b"USDY_USDC_ORACLE_V2"], &usdy_usd_oracle::ID);
        println!("program_state_pubkey: {:?}", program_state_pubkey);

        let (oracle_pubkey, _oracle_bump) =
            Pubkey::find_program_address(&[b"ORACLE_USDY_SEED_V2"], &usdy_usd_oracle::ID);
        println!("oracle_pubkey: {:?}", oracle_pubkey);
        let (ondo_price_feed, _) =
            Pubkey::find_program_address(&[b"ORACLE_USDY_SEED_V2",
            runner.function.as_ref(),
            b"ondo_price_feed"],
            &usdy_usd_oracle::ID);
        let (ondo_traded_feed, _) =
            Pubkey::find_program_address(&[b"ORACLE_USDY_SEED_V2",
            runner.function.as_ref(),
            b"ondo_traded_feed"],
            &usdy_usd_oracle::ID);
        println!("oracle_pubkey: {:?}", oracle_pubkey);
        
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
                //ondo_price_feed
                AccountMeta {
                    pubkey: ondo_price_feed,
                    is_signer: false,
                    is_writable: true,
                },
                //ondo_traded_feed
                AccountMeta {
                    pubkey: ondo_traded_feed,
                    is_signer: false,
                    is_writable: true,
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

