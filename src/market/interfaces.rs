use std::cmp::min;
use serde::{ Deserialize, Serialize };
use crate::utils::construct_druid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTrade {
    pub bid_id: String,
    pub ask_id: String,
    pub quantity: u64,
    pub price: f64,
    pub created_at: String,
    pub druid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub asset_address: String,
    pub price: f64,
    pub quantity: u64,
    pub is_bid: bool,
    pub created_at: String,
    pub druid: String,
    pub desired_asset_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
    pub pending_trades: Vec<PendingTrade>,
}

/// Finds the index for an order to be inserted at based on the price
///
/// ### Arguments
///
/// * `prices` - A list of current orders
/// * `price` - The price of the order to be inserted
pub fn find_index_for_order(prices: &mut Vec<Order>, price: &f64) -> usize {
    let mut left = 0;
    let mut right = prices.len() - 1;

    while left <= right {
        let mid = (left + right) / 2;
        let mid_price = &prices[mid].price;

        if mid_price == price {
            return mid;
        } else if mid_price < price {
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }

    left
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: Vec::new(),
            asks: Vec::new(),
            pending_trades: Vec::new(),
        }
    }

    /// Adds an order to the order book
    ///
    /// ### Arguments
    ///
    /// * `order` - The order to be added
    pub fn add_order(&mut self, order: Order) {
        if order.is_bid {
            self.match_bid(&mut order.clone());
        } else {
            self.insert_order_in_list(order, false);
        }
    }

    /// Matches a bid with the lowest ask, if possible. If not possible,
    /// the bid is added to the order book
    ///
    /// ### Arguments
    ///
    /// * `bid` - The bid to be matched
    pub fn match_bid(&mut self, bid: &mut Order) {
        let mut ask_idx = 0;

        while ask_idx < self.asks.len() {
            let ask = &self.asks[ask_idx];

            if ask.price <= bid.price {
                let quantity = min(ask.quantity, bid.quantity);
                let pending_trade = PendingTrade {
                    bid_id: bid.id.clone(),
                    ask_id: ask.id.clone(),
                    quantity,
                    price: ask.price,
                    created_at: String::from(""),
                    druid: construct_druid(),
                };

                self.pending_trades.push(pending_trade);
                self.asks[ask_idx].quantity -= quantity;
                bid.quantity -= quantity;

                self.clean_up_empty_orders(&Some(ask_idx), &None);
                ask_idx += 1;
            } else {
                self.insert_order_in_list(bid.clone(), true);
                break;
            }
        }
    }

    /// Inserts an order into the order book at the correct index
    /// 
    /// ### Arguments
    /// 
    /// * `order` - The order to be inserted
    /// * `is_bid` - Whether the order is a bid or not
    fn insert_order_in_list(&mut self, order: Order, is_bid: bool) {
        let order_list = if is_bid { &mut self.bids } else { &mut self.asks };
        let search_idx = find_index_for_order(order_list, &order.price);
        let idx = if order_list[search_idx].price > order.price && is_bid {
            search_idx + 1
        } else {
            search_idx - 1
        };

        order_list.insert(idx, order);
    }

    /// Removes orders from the order book if they have no quantity left
    ///
    /// ### Arguments
    ///
    /// * `ask_idx` - The index of the ask to be removed
    /// * `bid_idx` - The index of the bid to be removed
    fn clean_up_empty_orders(&mut self, ask_idx: &Option<usize>, bid_idx: &Option<usize>) {
        if ask_idx.is_some() && self.asks[ask_idx.unwrap()].quantity == 0 {
            let idx = ask_idx.unwrap();
            self.asks.remove(idx);
        }

        if bid_idx.is_some() && self.bids[bid_idx.unwrap()].quantity == 0 {
            self.bids.remove(bid_idx.unwrap());
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub address: String,
    pub name: String,
    pub symbol: Option<String>,
    pub total_supply: u64,
    pub highest_bid: Option<String>,
    pub lowest_ask: Option<String>,
}
