use crate::futures::future::join_all;
use rust_decimal::Decimal;
use std::boxed::Box;
use std::future::Future;
use std::pin::Pin;
pub use switchboard_solana::prelude::*;
pub mod etherprices;

pub use etherprices::*;
use std::str::FromStr;
use switchboard_solana::switchboard_function;
use switchboard_utils;
use switchboard_utils::FromPrimitive;
use switchboard_utils::SbError;
use switchboard_utils::ToPrimitive;
use tokio;

use ethers::types::I256;

use ethers_contract_derive::abigen;

declare_id!("2LuPhyrumCFRXjeDuYp1bLNYp7EbzUraZcvrzN9ZBUkN");

pub const PROGRAM_SEED: &[u8] = b"USDY_USDC_ORACLE_V2";

pub const ORACLE_SEED: &[u8] = b"ORACLE_USDY_SEED_V2";
//
#[account(zero_copy(unsafe))]
pub struct MyProgramState {
    pub bump: u8,
    pub authority: Pubkey,
    pub switchboard_function: Pubkey,
    pub btc_price: f64,
}
async fn get_uniswap_price(
    ether_transport: ethers::providers::Provider<ethers::providers::Http>,
    factory_addr: ethers::types::H160,
    usd_h160: ethers::types::H160,
    usdy_h160: ethers::types::H160,
) -> Result<Decimal, SbError> {
    abigen!(Factory, "./src/factory.json");
    let factory_contract = Factory::new(factory_addr, ether_transport.clone().into());

    let pool = factory_contract
        .get_pool(usd_h160, usdy_h160, 500)
        .call()
        .await;

    println!("pool: {:?}", &pool);

    abigen!(Pool, "./src/pool.json");
    let pool_contract = Pool::new(pool.unwrap(), ether_transport.into());
    let slot0 = pool_contract.slot_0().call().await.unwrap();
    //    sqrtPriceX96 = sqrt(price) * 2 ** 96

    let sqrt_price_x96: ethers::types::U256 = slot0.0;
    let price: ethers::types::U256 = (sqrt_price_x96 * sqrt_price_x96) >> (96 * 2);

    let inverse_price: f64 = 0.000001 / (price.as_u128() as f64);
    let inverse_price = 1.0 / inverse_price;
    let inverse_price = 1_000_000_000_000_000_000.0 / inverse_price * 1_000_000_000_000_000_000.0;
    println!("Uniswap price: {:?}", &inverse_price);
    Ok(Decimal::from_f64(inverse_price).unwrap())
}
// future
async fn get_ondo_price(
    ether_transport: ethers::providers::Provider<ethers::providers::Http>,
) -> Result<Decimal, SbError> {
    abigen!(Ondo, "./src/ondo.json");
    let ondo = Ondo::new(
        ethers::types::H160::from_str("0xa0219aa5b31e65bc920b5b6dfb8edf0988121de0").unwrap(),
        ether_transport.clone().into(),
    );

    let price = ondo.get_price().call().await.unwrap();
    let price: u128 = price.as_u128();
    println!("Ondo price: {:?}", price);
    // return Pin<Box<dyn Future<Output = Result<Decimal, SbError>>>> {
    Ok(Decimal::from_u128(price).unwrap())
}
#[switchboard_function]
pub async fn etherprices_oracle_function(
    runner: FunctionRunner,
    _params: Vec<u8>,
) -> Result<Vec<Instruction>, SbFunctionError> {
    msg!("etherprices_oracle_function");

    let mantle_transport =
        ethers::providers::Provider::try_from("https://mantle.publicnode.com").unwrap();
    let ether_transport =
        ethers::providers::Provider::try_from("https://ethereum.publicnode.com").unwrap();
    let agni_factory =
        ethers::types::H160::from_str("0x25780dc8Fc3cfBD75F33bFDAB65e969b603b2035").unwrap();

    let fusion_factory =
        ethers::types::H160::from_str("0x530d2766D1988CC1c000C8b7d00334c14B69AD71").unwrap();
    let usdy_h160 =
        ethers::types::H160::from_str("0x5bE26527e817998A7206475496fDE1E68957c5A6").unwrap();
    let usd_h160 =
        ethers::types::H160::from_str("0x09Bc4E0D864854c6aFB6eB9A9cdF58aC190D0dF9").unwrap();

    let v: Vec<Pin<Box<dyn Future<Output = Result<Decimal, SbError>> + Send>>> = vec![
        Box::pin(get_uniswap_price(
            mantle_transport.clone(),
            agni_factory,
            usd_h160,
            usdy_h160,
        )),
        Box::pin(get_uniswap_price(
            mantle_transport.clone(),
            fusion_factory,
            usd_h160,
            usdy_h160,
        )),
        Box::pin(get_ondo_price(ether_transport.clone())),
    ];
    let usdy_decimals: Vec<Decimal> = join_all(v).await.into_iter().map(|x| x.unwrap()).collect();
    let first_two = usdy_decimals[0..2].to_vec();
    let ondo_price = usdy_decimals.last().unwrap() / Decimal::from(1_000_000_000 as u64);
    let usdy_decimals_divided_by_e18 = first_two
        .into_iter()
        .map(|x| x / Decimal::from(1_000_000_000_000_000_000 as u64))
        .collect::<Vec<Decimal>>();
    let usdy_e18s_f64s = usdy_decimals_divided_by_e18
        .into_iter()
        .map(|x| f64::from_str(&x.to_string()).unwrap())
        .collect::<Vec<f64>>();
    let usdy_mean = statistical::mean(&usdy_e18s_f64s);

    
    let usdy_mean = Decimal::from_f64(usdy_mean).unwrap()
        * Decimal::from(1_000_000_000 as u64 );

    
    println!("usdy_mean: {:?}", usdy_mean);

    println!("USDY Mean: {}", usdy_mean);
    msg!("sending transaction");
    println!("Ondo price: {:?}", ondo_price);
    // Finally, emit the signed quote and partially signed transaction to the functionRunner oracle
    // The functionRunner oracle will use the last outputted word to stdout as the serialized result. This is what gets executed on-chain.
    let etherprices = EtherPrices::fetch(
        // implement error handling and map_err
        ethers::types::U256::from(ToPrimitive::to_u128(&ondo_price).unwrap()),
        ethers::types::U256::from(ToPrimitive::to_u128(&usdy_mean).unwrap()),
        
    )
    .await
    .unwrap();
    println!("1");
    let ixs: Vec<Instruction> = etherprices.to_ixns(&runner);
    Ok(ixs)
}

#[sb_error]
pub enum Error {
    InvalidResult,
}