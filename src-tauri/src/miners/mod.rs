pub mod adapter;
pub mod lolminer;
pub mod plugin_sketch;
pub mod xmrig;

pub use adapter::{MinerAdapter, MinerKind, MinerRunState, MinerStats, StartRequest};
pub use lolminer::LolMinerAdapter;
pub use xmrig::XmrigAdapter;
