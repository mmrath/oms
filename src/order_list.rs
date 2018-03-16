use std::collections::HashMap;
use failure::Error;
use std::ops::{Index, IndexMut};

use crate::model::OrderInfo;

#[derive(Debug)]
pub(crate) struct OrderList {
    //Keeps list of orders
    orders: Vec<OrderInfo>,
    free: Vec<usize>,
    //Keeps a map for easy deletion
    order_map: HashMap<u64, usize>,
}

impl OrderList {
    pub fn new() -> Self {
        let max_size: usize = 100_000;
        let mut list = Self {
            orders: Vec::with_capacity(max_size),
            free: Vec::with_capacity(max_size),
            order_map: HashMap::with_capacity(max_size),
        };

        //Preallocate
        for i in 0..max_size {
            list.orders.push(OrderInfo::new(0, 0, 0));
            list.free.push(i);
        }
        list
    }

    pub fn insert(&mut self, id: u64, price: u64, qty: u64) -> Result<usize, Error> {
        //set size to zero

        if self.free.is_empty() {
            self.orders.push(OrderInfo::new(id, price, qty));
            let index = self.orders.len() - 1;
            self.order_map.insert(id, index);
            Ok(index)
        } else {
            let index = self.free.pop().unwrap(); // Safe
            let ord = &mut self.orders[index];
            ord.id = id;
            ord.qty = qty;
            ord.price = price;
            self.order_map.insert(id, index);
            Ok(index)
        }
    }

    pub fn delete(&mut self, id: &u64) -> Result<bool, Error> {
        //set size to zero
        if let Some(idx) = self.order_map.remove(id) {
            if let Some(mut ord) = self.orders.get_mut(idx) {
                //let result = ord.clone();
                self.free.push(idx);
                ord.qty = 0;
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl Index<usize> for OrderList {
    type Output = OrderInfo;

    #[inline]
    fn index(&self, index: usize) -> &OrderInfo {
        &self.orders[index]
    }
}

impl IndexMut<usize> for OrderList {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut OrderInfo {
        &mut (self.orders[index])
    }
}
