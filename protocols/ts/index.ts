// TypeScript type mirrors for Borsh deserializer integrations

export interface PriceLevel {
  price: number;
  size: number;
}

export interface OrderBook {
  symbol: string;
  timestamp: number;
  bids: PriceLevel[];
  asks: PriceLevel[];
}

export type OrderSide = "Buy" | "Sell";
export type OrderType = "Limit" | "Market";

export interface Trade {
  tradeId: string;
  symbol: string;
  price: number;
  size: number;
  side: OrderSide;
  orderType: OrderType;
  timestamp: number;
}

export type AlertSeverity = "Info" | "Warning" | "Critical";

export interface TantraAlert {
  alertId: string;
  source: string;
  severity: AlertSeverity;
  message: string;
  timestamp: number;
}
