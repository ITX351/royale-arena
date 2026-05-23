import type { Item, Player } from '@/types/gameStateTypes'

// Returns the highest vote bonus from the provided items
function getMaxItemVotes(items: Array<Item | null | undefined>): number {
  let maxVotes = 0
  for (const item of items) {
    if (!item) {
      continue
    }
    const votes = item.item_type?.properties?.votes
    if (typeof votes === 'number' && votes > maxVotes) {
      maxVotes = votes
    }
  }
  return maxVotes
}

export const calculatePlayerVotes = (player: Player): number => {
  const equippedItems = [player.equipped_weapon, player.equipped_armor]
  const bagItems = player.inventory ?? []
  const maxVotes = getMaxItemVotes([...equippedItems, ...bagItems])
  return 1 + maxVotes
}
