import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { registerHooks } from 'node:module'
import test from 'node:test'

// Node >= 22.18: resolve extensionless imports in the existing TypeScript sources.
registerHooks({
  resolve(specifier, context, nextResolve) {
    if (context.parentURL?.endsWith('.ts') && specifier.startsWith('./') && !specifier.endsWith('.ts')) {
      return nextResolve(`${specifier}.ts`, context)
    }
    return nextResolve(specifier, context)
  },
})

const { GameRuleParser } = await import('../src/utils/gameRuleParser.ts')
const { DEFAULT_RULES_CONFIG } = await import('../src/constants/defaultRulesConfig.ts')
const parser = new GameRuleParser()
const validate = (rules) => parser.validate(rules)

test('default and full-feature rules remain valid', () => {
  for (const rules of [DEFAULT_RULES_CONFIG, JSON.parse(readFileSync(new URL('../public/docs/full-feature-rules-template.json', import.meta.url), 'utf8'))]) {
    const result = validate(rules)
    assert.equal(result.isValid, true, result.errors.join('\n'))
  }
})

test('optional caps may be omitted', () => {
  const rules = structuredClone(DEFAULT_RULES_CONFIG)
  for (const field of ['max_life_cap', 'max_strength_cap', 'max_backpack_items_cap']) delete rules.player[field]
  assert.equal(validate(rules).isValid, true)
})

for (const field of ['max_life_cap', 'max_strength_cap', 'max_backpack_items_cap']) {
  const backpack = field === 'max_backpack_items_cap'
  const accepted = backpack ? [0, 2147483648, Number.MAX_SAFE_INTEGER] : [-2147483648, 0, 2147483647]
  const rejected = [1.5, NaN, Infinity, -Infinity, null, '100', ...(backpack ? [-1, Number.MAX_SAFE_INTEGER + 1] : [-2147483649, 2147483648])]
  for (const value of accepted) {
    test(`${field} accepts ${value}`, () => {
      const rules = structuredClone(DEFAULT_RULES_CONFIG)
      rules.player[field] = value
      const result = validate(rules)
      assert.equal(result.isValid, true, result.errors.join('\n'))
    })
  }
  for (const value of rejected) {
    test(`${field} rejects ${String(value)} (${typeof value})`, () => {
      const rules = structuredClone(DEFAULT_RULES_CONFIG)
      rules.player[field] = value
      assert.equal(validate(rules).isValid, false)
    })
  }
}

for (const value of [-2147483648, -1, 1, 2147483647, 0, 1.5, -2147483649, 2147483648, NaN, Infinity, null, '1']) {
  test(`permanent buff validates effect_value ${String(value)} (${typeof value})`, () => {
    const rules = structuredClone(DEFAULT_RULES_CONFIG)
    rules.items_config.items.permanent_buffs[0].properties.effect_value = value
    assert.equal(validate(rules).isValid, [-2147483648, -1, 1, 2147483647].includes(value))
  })
}
