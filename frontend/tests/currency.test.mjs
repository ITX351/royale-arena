import assert from 'node:assert/strict'
import test from 'node:test'
import { isValidSellPrice, isValidBalance, MAX_COINS } from '../src/utils/currency.ts'

test('sell prices require finite half-units within the UI and server limit', () => {
  for (const value of [0.5, 1, 1.5, 9999]) assert.equal(isValidSellPrice(value), true)
  for (const value of [undefined, null, '', '0.5', 0, -1, 0.6, 9999.5, 1e308, NaN, Infinity]) {
    assert.equal(isValidSellPrice(value), false, String(value))
  }
})

test('balances retain half-units and reject unsafe amounts', () => {
  for (const value of [0, 0.5, 100.5, MAX_COINS]) assert.equal(isValidBalance(value), true)
  for (const value of [-0.5, 0.6, MAX_COINS + 0.5, 1e308, NaN, Infinity]) {
    assert.equal(isValidBalance(value), false, String(value))
  }
})
