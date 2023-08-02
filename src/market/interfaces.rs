use crate::utils::{construct_druid, construct_formatted_date};
use serde::{Deserialize, Serialize};
use std::cmp::min;

/// A pending trade within the orderbook that will need to be resolved on chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTrade {
    pub bid_id: String,
    pub ask_id: String,
    pub quantity: u64,
    pub price: f64,
    pub created_at: String,
    pub druid: String,
}

/// Order to be placed within an orderbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub asset_address: String,
    pub price: f64,
    pub quantity: u64,
    pub is_bid: bool,
    pub created_at: String,
    pub desired_asset_address: Option<String>,
}

/// Orderbook of bid and ask orders for a given asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
    pub pending_trades: Vec<PendingTrade>,
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
            self.match_ask(&mut order.clone());
        }
    }

    pub fn match_ask(&mut self, ask: &mut Order) {
        let mut bid_idx = 0;

        // Loop is required to match an ask with multiple bids
        while bid_idx < self.bids.len() {
            let bid = &self.bids[bid_idx];

            if bid.price >= ask.price {
                self.construct_pending_trade(ask, bid_idx, bid.quantity);
                self.clean_up_empty_orders(&None, &Some(bid_idx));
                bid_idx += 1;
            } else {
                self.insert_order_in_list(ask.clone(), false);
                break;
            }
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

        // Loop is required to match a bid with multiple asks
        while ask_idx < self.asks.len() {
            let ask = &self.asks[ask_idx];

            if ask.price <= bid.price {
                self.construct_pending_trade(bid, ask_idx, ask.quantity);
                self.clean_up_empty_orders(&Some(ask_idx), &None);
                ask_idx += 1;
            } else {
                self.insert_order_in_list(bid.clone(), true);
                break;
            }
        }
    }

    /// Constructs a pending trade and adds it to the pending trades list
    ///
    /// ### Argeumnts
    ///
    /// * `order` - The order to be matched
    /// * `list_idx` - The index of the existing order to be matched
    /// * `list_quantity` - The quantity of the existing order to be matched
    fn construct_pending_trade(&mut self, order: &mut Order, list_idx: usize, list_quantity: u64) {
        let match_list = if order.is_bid {
            &mut self.asks
        } else {
            &mut self.bids
        };
        let quantity = min(order.quantity, list_quantity);
        let pending_trade = PendingTrade {
            bid_id: order.id.clone(),
            ask_id: match_list[list_idx].id.clone(),
            quantity,
            price: match_list[list_idx].price,
            created_at: construct_formatted_date(),
            druid: construct_druid(),
        };

        self.pending_trades.push(pending_trade);
        order.quantity -= quantity;
        match_list[list_idx].quantity -= quantity;
    }

    /// Inserts an order into the order book at the correct index
    ///
    /// ### Arguments
    ///
    /// * `order` - The order to be inserted
    /// * `is_bid` - Whether the order is a bid or not
    fn insert_order_in_list(&mut self, order: Order, is_bid: bool) {
        let order_list = if is_bid {
            &mut self.bids
        } else {
            &mut self.asks
        };
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
