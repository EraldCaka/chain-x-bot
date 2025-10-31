<h3 align="center">Chain X Bot</h3>


<p align="center">
  A Rust-based crypto trading bot using Backpack API and Binance ticker integration.
</p>

---

## About

**Trade Bot** is an automated trading bot that interacts with the Backpack exchange API to place market and limit orders. It can also fetch live cryptocurrency prices from Binance to make informed trading decisions.

**Status:**  Under Development

---

## Features

- Place **Market** and **Limit** orders via Backpack API
- Monitor **live crypto prices** from Binance
- Batch order execution support
- Logging of executed orders and API responses
- RSI indicator support  `soon`
- MACD indicator support `soon`
- Stochastic indicator support `soon`

---
<p align="center">
  <img src="public/trades.png" alt="trades"/>
</p>

---

## Configuration

Create a `.env` file in the root of the project and add your Backpack API credentials and endpoints:

```env
API_KEY=your_backpack_api_key
API_SECRET=your_backpack_api_secret
BACKPACK_REST_URL=https://api.backpack.exchange/api/v1
BACKPACK_WS_URL=wss://ws.backpack.exchange
```