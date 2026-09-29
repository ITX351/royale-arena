export const MAX_COINS = Number.MAX_SAFE_INTEGER / 2
export const MAX_SELL_PRICE = 9999

export const isValidBalance = (value: unknown): value is number =>
  typeof value === 'number' && Number.isFinite(value) &&
  value >= 0 && value <= MAX_COINS && Number.isInteger(value * 2)

export const isValidSellPrice = (price: unknown): price is number =>
  typeof price === 'number' && Number.isFinite(price) &&
  price >= 0.5 && price <= MAX_SELL_PRICE && Number.isInteger(price * 2)
