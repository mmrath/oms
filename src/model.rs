use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Default)]
pub struct IdGen {
    seq: AtomicUsize, // Does not need to be atomic as of now.
}

impl IdGen {
    pub fn new() -> Self {
        Self {
            seq: AtomicUsize::new(1),
        }
    }

    pub fn next(&self) -> u64 {
        self.seq.fetch_add(1, Ordering::SeqCst) as u64
    }
}

lazy_static! {
    pub static ref ORDER_ID_GEN : IdGen = IdGen::new();
}

lazy_static! {
    pub static ref ORDER_FILL_ID_GEN : IdGen = IdGen::new();
}

#[cfg(test)]
pub fn test_setup() {
    use env_logger;

    let _ = env_logger::try_init();
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Instrument {
    symbol: String,
}

impl Instrument {
    pub fn new(sym: &str) -> Self {
        Self {
            symbol: String::from(sym),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Deserialize)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Copy, Clone)]
pub enum OrderEvent {
    Market {
        id: u64,
        side: Side,
        qty: u64,
    },
    Limit {
        id: u64,
        side: Side,
        price: u64,
        qty: u64,
    },
    Cancel {
        id: u64,
    },
    Replace {
        id: u64,
        side: Side,
        price: u64,
        qty: u64,
    },
}

impl OrderEvent {
    pub fn market(id: u64, side: Side, qty: u64) -> Self {
        OrderEvent::Market { id, side, qty }
    }

    pub fn limit(id: u64, side: Side, price: u64, qty: u64) -> Self {
        OrderEvent::Limit {
            id,
            side,
            price,
            qty,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OrderFill {
    id: u64,
    ord_id_1: u64,
    ord_id_2: u64,
    price: u64,
    qty: u64,
}

impl OrderFill {
    pub fn new(ord_id_1: u64, ord_id_2: u64, price: u64, qty: u64) -> Self {
        Self {
            id: (&*ORDER_FILL_ID_GEN).next(),
            ord_id_1,
            ord_id_2,
            price,
            qty,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn ord_id_1(&self) -> u64 {
        self.ord_id_1
    }

    pub fn ord_id_2(&self) -> u64 {
        self.ord_id_2
    }

    pub fn price(&self) -> u64 {
        self.price
    }

    pub fn qty(&self) -> u64 {
        self.qty
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct OrderInfo {
    // A persistent id - from DB
    crate id: u64,
    crate price: u64,
    crate qty: u64,
}

impl OrderInfo {
    pub(crate) fn new(id: u64, price: u64, qty: u64) -> Self {
        Self { id, price, qty }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn price(&self) -> u64 {
        self.price
    }
    pub fn qty(&self) -> u64 {
        self.qty
    }
    pub fn fill(&mut self, fill_qty: u64) {
        self.qty -= fill_qty;
    }
}
