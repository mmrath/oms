use crate::order_list::OrderList;
use crate::model::{Instrument, OrderEvent, OrderFill, Side};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::option::Option::None;
use failure::Error;

#[derive(Debug)]
pub struct OrderBook {
    instrument: Instrument,
    last_traded_price: Option<u64>,
    order_list: OrderList,
    max_bid: Option<u64>,
    min_ask: Option<u64>,
    bids: BTreeMap<u64, Vec<usize>>,
    asks: BTreeMap<u64, Vec<usize>>,
}

impl OrderBook {
    pub fn new(instrument: Instrument) -> Self {
        Self {
            instrument,
            last_traded_price: None,
            order_list: OrderList::new(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            max_bid: None,
            min_ask: None,
        }
    }

    pub fn event(&mut self, event: OrderEvent) -> Result<Vec<OrderFill>, Error> {
        match event {
            OrderEvent::Market { id, side, qty } => self.market(id, side, qty),
            OrderEvent::Limit {
                id,
                side,
                qty,
                price,
            } => self.limit(id, side, qty, price),
            OrderEvent::Cancel { id } => {
                let _ = self.cancel(id);
                Ok(Vec::new())
            }
            _ => unimplemented!("Not implemented"),
        }
    }

    pub fn cancel(&mut self, order_id: u64) -> Result<(), Error> {
        let _ = self.order_list.delete(&order_id)?;
        Ok(())
    }

    fn market(&mut self, id: u64, side: Side, qty: u64) -> Result<Vec<OrderFill>, Error> {
        let mut fills: Vec<OrderFill> = Vec::new();
        let mut remaining_qty: u64 = qty;

        match side {
            Side::Bid => {
                while remaining_qty > 0 && !self.asks.is_empty() {
                    //Unwrap is safe as the list is not empty
                    let best_price = *self.min_ask().unwrap();
                    if let Entry::Occupied(mut entry) = self.asks.entry(best_price) {
                        let best_orders = entry.get_mut();

                        let new_fills = Self::process_order_list(
                            &mut self.order_list,
                            best_orders,
                            remaining_qty,
                            id,
                        )?;

                        if best_orders.is_empty() {
                            entry.remove();
                            self.min_ask = self.min_ask().cloned();
                        }
                        let filled_qty: u64 = new_fills.iter().map(|f| f.qty()).sum();
                        remaining_qty -= filled_qty;
                        fills.extend(new_fills);
                    } else {
                        panic!("Should not be reachable");
                    }
                }
                if remaining_qty > 0 {
                    info!(
                        "There are not enough sell orders to fulfill this order {:?}",
                        id
                    );
                }
            }
            Side::Ask => {
                while remaining_qty > 0 && !self.bids.is_empty() {
                    let best_price = *self.max_bid().unwrap();
                    if let Entry::Occupied(mut entry) = self.bids.entry(best_price) {
                        let best_orders = entry.get_mut();
                        let new_fills = Self::process_order_list(
                            &mut self.order_list,
                            best_orders,
                            remaining_qty,
                            id,
                        )?;
                        if best_orders.is_empty() {
                            entry.remove();
                            self.max_bid = self.max_bid().cloned();
                        }
                        let filled_qty: u64 = new_fills.iter().map(|fill| fill.qty()).sum();
                        remaining_qty -= filled_qty;
                        fills.extend(new_fills);
                    } else {
                        panic!("Should not be reachable");
                    }
                }

                if remaining_qty > 0 {
                    info!(
                        "There are not enough buy orders to fulfill this order {:?}",
                        id
                    );
                }
            }
        }

        Ok(fills)
    }

    fn limit(
        &mut self,
        id: u64,
        side: Side,
        qty: u64,
        price: u64,
    ) -> Result<Vec<OrderFill>, Error> {
        let mut fills: Vec<OrderFill> = Vec::new();
        let mut remaining_qty: u64 = qty;
        match side {
            Side::Bid => {
                trace!("In buy, Book: {:?} order: {:?}", self, id);
                //Safe unwrap as asks not empty
                while remaining_qty > 0 && !self.asks.is_empty()
                    && price >= *self.min_ask().unwrap()
                {
                    let best_price = *self.min_ask().unwrap();

                    if let Entry::Occupied(mut entry) = self.asks.entry(best_price) {
                        let best_orders = entry.get_mut();

                        let new_fills = Self::process_order_list(
                            &mut self.order_list,
                            best_orders,
                            remaining_qty,
                            id,
                        )?;
                        if best_orders.is_empty() {
                            entry.remove();
                            self.min_ask = self.min_ask().cloned();
                        }
                        let filled_qty: u64 = new_fills.iter().map(|f| f.qty()).sum();
                        remaining_qty -= filled_qty;
                        fills.extend(new_fills);
                    }
                }
                if remaining_qty > 0 {
                    let index: usize = self.order_list.insert(id, price, qty)?;
                    self.bids
                        .entry(price)
                        .or_insert_with(|| Vec::with_capacity(10))
                        .push(index);
                }
            }
            Side::Ask => {
                while remaining_qty > 0 && !self.bids.is_empty()
                    && price <= *self.max_bid().unwrap()
                {
                    let best_price = *self.max_bid().unwrap();

                    if let Entry::Occupied(mut entry) = self.bids.entry(best_price) {
                        let best_orders = entry.get_mut();

                        debug!(
                            "Remaining: {:?}, best bids: {:?}",
                            remaining_qty, best_orders
                        );

                        let new_fills = Self::process_order_list(
                            &mut self.order_list,
                            best_orders,
                            remaining_qty,
                            id,
                        )?;
                        if best_orders.is_empty() {
                            entry.remove();
                            self.max_bid = self.max_bid().cloned();
                        }
                        let filled_qty: u64 = new_fills.iter().map(|fill| fill.qty()).sum();
                        remaining_qty -= filled_qty;
                        fills.extend(new_fills);
                    }
                }

                if remaining_qty > 0 {
                    debug!("Remaining: {:?}", remaining_qty);
                    let index: usize = self.order_list.insert(id, price, qty)?;
                    self.asks
                        .entry(price)
                        .or_insert_with(|| Vec::with_capacity(10))
                        .push(index);
                }
            }
        }

        Ok(fills)
    }

    fn process_order_list(
        order_list: &mut OrderList,
        opposite_orders: &mut Vec<usize>,
        quantity_still_to_trade: u64,
        id: u64,
    ) -> Result<Vec<OrderFill>, Error> {
        /*
          Takes an OrderList (stack of orders at one price) and an incoming order and matches
          appropriate trades given the order's quantity.
          **/

        let mut fills: Vec<OrderFill> = Vec::new();
        let mut qty_to_fill = quantity_still_to_trade;
        let mut filled_index = None;

        debug!(
            "Process order list, OrderList: {:?} order: {:?}",
            opposite_orders, id
        );

        for (index, head_order_idx) in opposite_orders.iter_mut().enumerate() {
            if qty_to_fill == 0 {
                break;
            }
            let head_order = &mut order_list[*head_order_idx];
            let traded_price = head_order.price();
            let available_qty = head_order.qty();
            if available_qty == 0 {
                filled_index = Some(index);
                continue;
            }
            let traded_quantity: u64;

            debug!(
                "Quantity to trade: {:?} available_qty: {:?}",
                qty_to_fill, available_qty
            );

            if qty_to_fill >= available_qty {
                traded_quantity = available_qty;
                qty_to_fill -= available_qty;
                filled_index = Some(index);
            } else {
                traded_quantity = qty_to_fill;
                qty_to_fill = 0u64;
            }
            head_order.fill(traded_quantity);
            let fill: OrderFill;
            fill = OrderFill::new(id, head_order.id(), traded_price, traded_quantity);
            fills.push(fill);
        }
        debug!("Filled index {:?}", filled_index);
        if let Some(index) = filled_index {
            opposite_orders.drain(0..index + 1);
        }

        Ok(fills)
    }

    fn max_bid(&self) -> Option<&u64> {
        self.bids.keys().max()
    }

    fn min_ask(&self) -> Option<&u64> {
        self.asks.keys().min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn market_order_insertion_with_no_previous_order() {
        ::crate::core::test_setup();

        let mut ob: OrderBook = OrderBook::new(Instrument::new("AUDUSD"));

        let o1 = OrderEvent::market(1, Side::Bid, 100u64);
        let o2 = OrderEvent::market(1, Side::Ask, 100u64);

        let filled = ob.event(o1);
        assert_eq!(filled.unwrap().len(), 0, "Order is not filled");
        let filled = ob.event(o2);
        assert_eq!(filled.unwrap().len(), 0, "Order is not filled");
    }

    #[test]
    pub fn limit_order_insertion_with_no_previous_order() {
        ::crate::core::test_setup();

        let mut ob: OrderBook = OrderBook::new(Instrument::new("AUDUSD"));

        let o1 = OrderEvent::limit(1, Side::Bid, 10u64, 100u64);
        let o2 = OrderEvent::limit(2, Side::Ask, 10u64, 100u64);

        let filled = ob.event(o1);
        assert_eq!(filled.unwrap().len(), 0, "Order is not filled");
        //assert_eq!(ob.bids.len(), 1, "There should be one bid");
        //assert_eq!(ob.asks.len(), 0, "There should be zero asks");
        let filled = ob.event(o2);
        assert_eq!(filled.unwrap().len(), 1, "Order should be filled");
        //assert_eq!(ob.bids.len(), 0, "There should be zero bids");
        //assert_eq!(ob.asks.len(), 0, "There should be zero asks");
    }

    struct TestData {
        pub orders: Vec<OrderEvent>,
        pub cancels: Vec<u64>,
        pub orders2: Vec<OrderEvent>,
        pub expected: Vec<OrderFill>,
    }

    fn sell_1_101x100() -> OrderEvent {
        OrderEvent::limit(1, Side::Ask, 101u64, 100u64)
    }

    fn buy_2_101x100() -> OrderEvent {
        OrderEvent::limit(2, Side::Bid, 101u64, 100u64)
    }

    fn sell_3_101x50() -> OrderEvent {
        OrderEvent::limit(3, Side::Ask, 101u64, 50u64)
    }

    fn buy_4_101x50() -> OrderEvent {
        OrderEvent::limit(4, Side::Bid, 101u64, 50u64)
    }

    fn sell_5_101x25() -> OrderEvent {
        OrderEvent::limit(5, Side::Ask, 101u64, 25u64)
    }

    fn buy_6_101x25() -> OrderEvent {
        OrderEvent::limit(6, Side::Bid, 101u64, 25u64)
    }

    fn buy_7_101x25() -> OrderEvent {
        OrderEvent::limit(7, Side::Bid, 101u64, 25u64)
    }

    fn xa101x100() -> OrderFill {
        OrderFill::new(1, 2, 101u64, 100u64)
    }

    fn xa101x50() -> OrderFill {
        OrderFill::new(1, 1, 101u64, 50u64)
    }

    fn xb101x50() -> OrderFill {
        OrderFill::new(3, 4, 101u64, 50u64)
    }

    fn xa101x25() -> OrderFill {
        OrderFill::new(5, 6, 101u64, 25u64)
    }

    fn xb101x25x() -> OrderFill {
        OrderFill::new(7, 1, 101u64, 25u64)
    }

    #[test]
    pub fn test_ask() {
        run_test(TestData {
            orders: vec![sell_1_101x100()],
            cancels: vec![],
            orders2: vec![],
            expected: vec![],
        });
    }

    #[test]
    pub fn test_bid() {
        run_test(TestData {
            orders: vec![buy_2_101x100()],
            cancels: vec![],
            orders2: vec![],
            expected: vec![],
        })
    }

    #[test]
    fn test_execution() {
        run_test(TestData {
            orders: vec![sell_1_101x100(), buy_2_101x100()],
            cancels: vec![],
            orders2: vec![],
            expected: vec![OrderFill::new(2, 1, 101u64, 100u64)],
        });

        run_test(TestData {
            orders: vec![buy_2_101x100(), sell_1_101x100()],
            cancels: vec![],
            orders2: vec![],
            expected: vec![OrderFill::new(1, 2, 101u64, 100u64)],
        });
    }

    #[test]
    fn test_partial_fill1() {
        run_test(TestData {
            orders: vec![sell_1_101x100(), buy_4_101x50()],
            cancels: vec![],
            orders2: vec![],
            expected: vec![OrderFill::new(4, 1, 101u64, 50u64)],
        });

        run_test(TestData {
            orders: vec![buy_4_101x50(), sell_1_101x100()],
            cancels: vec![],
            orders2: vec![],
            expected: vec![OrderFill::new(1, 4, 101u64, 50u64)],
        });
    }

    #[test]
    fn test_incremental_over_fill1() {
        run_test(TestData {
            orders: vec![
                sell_1_101x100(),
                buy_6_101x25(),
                buy_6_101x25(),
                buy_6_101x25(),
                buy_6_101x25(),
                buy_6_101x25(),
            ],
            cancels: vec![],
            orders2: vec![],
            expected: vec![
                OrderFill::new(6, 1, 101u64, 25u64),
                OrderFill::new(6, 1, 101u64, 25u64),
                OrderFill::new(6, 1, 101u64, 25u64),
                OrderFill::new(6, 1, 101u64, 25u64),
            ],
        });
    }

    #[test]
    fn test_incremental_over_fill2() {
        run_test(TestData {
            orders: vec![
                buy_2_101x100(),
                sell_5_101x25(),
                sell_5_101x25(),
                sell_5_101x25(),
                sell_5_101x25(),
                sell_5_101x25(),
            ],
            cancels: vec![],
            orders2: vec![],
            expected: vec![
                OrderFill::new(5, 2, 101u64, 25u64),
                OrderFill::new(5, 2, 101u64, 25u64),
                OrderFill::new(5, 2, 101u64, 25u64),
                OrderFill::new(5, 2, 101u64, 25u64),
            ],
        });
    }

    #[test]
    fn test_queue_position() {
        run_test(TestData {
            orders: vec![buy_6_101x25(), buy_7_101x25(), sell_5_101x25()],
            cancels: vec![],
            orders2: vec![],
            expected: vec![OrderFill::new(5, 6, 101u64, 25u64)],
        });
    }

    #[test]
    fn test_cancel_simple() {
        run_test(TestData {
            orders: vec![buy_6_101x25()],
            cancels: vec![6],
            orders2: vec![sell_5_101x25()],
            expected: vec![],
        });
    }

    #[test]
    fn test_cancel_from_front_of_queue() {
        run_test(TestData {
            orders: vec![buy_6_101x25(), buy_7_101x25()],
            cancels: vec![6],
            orders2: vec![sell_5_101x25()],
            expected: vec![OrderFill::new(5, 7, 101u64, 25u64)],
        });
    }

    #[test]
    fn test_cancel_front_back_out_of_order_then_partial_execution() {
        run_test(TestData {
            orders: vec![
                buy_2_101x100(),
                buy_4_101x50(),
                buy_7_101x25(),
                buy_6_101x25(),
            ],
            cancels: vec![7, 2, 7],
            orders2: vec![sell_5_101x25()],
            expected: vec![OrderFill::new(5, 4, 101u64, 25)],
        });
    }

    fn run_test(mut data: TestData) {
        ::crate::core::test_setup();

        let mut ob: OrderBook = OrderBook::new(Instrument::new("AUDUSD"));
        let mut fills: Vec<OrderFill> = Vec::new();

        for index in 0..data.orders.len() {
            let ord = data.orders.remove(0);
            let mut new_fills = ob.event(ord).unwrap();
            fills.append(&mut new_fills);
        }

        for index in 0..data.cancels.len() {
            let ord_id = data.cancels.remove(0);
            ob.cancel(ord_id);
        }

        for index in 0..data.orders2.len() {
            let ord = data.orders2.remove(0);
            let mut new_fills = ob.event(ord).unwrap();
            fills.append(&mut new_fills);
        }

        assert_eq!(data.expected.len(), fills.len(), "testing fill length");

        for (actual, expected) in fills.iter().zip(data.expected.iter()) {
            assert_equal(actual, expected);
        }
    }

    fn assert_equal(actual: &OrderFill, expected: &OrderFill) {
        assert_eq!(actual.ord_id_1(), expected.ord_id_1());
        assert_eq!(actual.ord_id_2(), expected.ord_id_2());
        assert_eq!(actual.price(), expected.price());
        assert_eq!(actual.qty(), expected.qty());
    }
}
