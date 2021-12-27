use web3::transports::Http;
use web3::Web3;

#[derive(Clone, PartialEq)]
pub enum EthereumChain {
    AvalancheC,
    BinanceSmartChain,
    Ethereum,
    Moonriver,
    Moonbeam,
}

impl EthereumChain {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "avac" => Self::AvalancheC,
            "avax-c" => Self::AvalancheC,
            "avalanche-c" => Self::AvalancheC,
            "bsc" => Self::BinanceSmartChain,
            "binancesmartchain" => Self::BinanceSmartChain,
            "eth" => Self::Ethereum,
            "ethereum" => Self::Ethereum,
            "moonriver" => Self::Moonriver,
            "movr" => Self::Moonriver,
            "moonbeam" => Self::Moonbeam,
            "glmr" => Self::Moonbeam,
            _ => panic!("Invalid chain identifier!"),
        }
    }
    pub fn to_str(&self) -> &str {
        match *self {
            Self::AvalancheC => "Avalanche-C",
            Self::BinanceSmartChain => "BSC",
            Self::Ethereum => "Ethereum",
            Self::Moonriver => "Moonriver",
            Self::Moonbeam => "Moonbeam",
        }
    }
    pub fn to_trading_symbol(&self) -> String {
        match *self {
            Self::AvalancheC => "AVAX",
            Self::BinanceSmartChain => "BNB",
            Self::Ethereum => "ETH",
            Self::Moonriver => "MOVR",
            Self::Moonbeam => "GLMR",
        }
        .to_string()
    }
}

pub struct EthereumNode {
    pub chain: EthereumChain,
    pub web3: Web3<Http>,
}
