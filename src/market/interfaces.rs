use crate::utils::{ construct_druid, construct_formatted_date };
use serde::{ Deserialize, Serialize };
use std::{cmp::min, collections::HashSet};

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
    pub present_orders: HashSet<String>,
    pub pending_trades: Vec<PendingTrade>,
}

impl OrderBook {
    /// Creates a new orderbook
    pub fn new() -> Self {
        OrderBook {
            bids: Vec::new(),
            asks: Vec::new(),
            present_orders: HashSet::new(),
            pending_trades: Vec::new(),
        }
    }

    /// Checks if an order is present in the orderbook
    pub fn is_order_present(&self, order_id: &str) -> bool {
        self.present_orders.contains(order_id)
    }

    /// Adds an order to the orderbook, matching it with any existing orders
    ///
    /// ### Arguments
    ///
    /// * `current_order` - The order to be added
    pub fn add_order(&mut self, current_order: &mut Order) {
        let mut list_idx = 0;
        let mut match_list = if current_order.is_bid { &mut self.asks } else { &mut self.bids };

        // Loop is required to match a bid with multiple asks
        while list_idx < match_list.len() {
            let match_order = &match_list[list_idx];

            if
                (current_order.is_bid && match_order.price <= current_order.price) ||
                (!current_order.is_bid && match_order.price >= current_order.price)
            {
                let ask_idx = if current_order.is_bid { Some(list_idx) } else { None };
                let bid_idx = if current_order.is_bid { None } else { Some(list_idx) };

                self.construct_pending_trade(current_order, list_idx, match_order.quantity);
                self.clean_up_empty_orders(&ask_idx, &bid_idx);
                list_idx += 1;
            } else {
                self.insert_order_in_list(current_order.clone(), true);
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
        let match_list = if order.is_bid { &mut self.asks } else { &mut self.bids };
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
        let order_list = if is_bid { &mut self.bids } else { &mut self.asks };
        let search_idx = find_index_for_order(order_list, &order.price);
        let idx = if order_list[search_idx].price > order.price && is_bid {
            search_idx + 1
        } else {
            search_idx - 1
        };

        order_list.insert(idx, order);

        // Add the order to the present orders list
        self.present_orders.insert(order.id.clone());
    }

    /// Removes orders from the order book if they have no quantity left
    ///
    /// ### Arguments
    ///
    /// * `ask_idx` - The index of the ask to be removed
    /// * `bid_idx` - The index of the bid to be removed
    fn clean_up_empty_orders(&mut self, ask_idx: &Option<usize>, bid_idx: &Option<usize>) {
        if ask_idx.is_some() && self.asks[ask_idx.unwrap()].quantity == 0 {
            self.present_orders.remove(&self.asks[ask_idx.unwrap()].id);
            self.asks.remove(ask_idx.unwrap());
        }

        if bid_idx.is_some() && self.bids[bid_idx.unwrap()].quantity == 0 {
            self.present_orders.remove(&self.bids[bid_idx.unwrap()].id);
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
