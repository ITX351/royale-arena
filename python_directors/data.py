from constants import *
import random

globals = {
    'weather': 1.0 # 搜索到人物能看到是谁的概率
}

def alive_roles():
    res = [v for k, v in roles.items() if v['life'] > 0 and v['life'] <= 100]
    res.sort(key=lambda x: x['order'])
    for role in res:
        role['vote'] = 1
        for item in role['hands']:
            role['vote'] += count_vote(item)
    return res

def count_vote(item):
    try:
        return items[item][2]
    except:
        return 0
    return 0

def enabled_places():
    res = [v for k, v in places.items() if v['able']]
    res.sort(key=lambda x: x['order'])
    return res

def all_items():
    return list(items.keys()) + ITEM_PERFECT_CURE + ITEM_PREMIUM_CURE + ITEM_CURE + \
        ITEM_BANDAGE + ITEM_ACTIVE + ITEM_WATER + ITEM_TEA + list(ITEM_UPGRADE_MAP.keys())

# order: 显示顺序，exists: 当地散落的道具，able: 是否启用，safe: 是否安全
places = {}
if DIRECTIONS_ENABLED:
    for i in range(0, len(PLACES) * 4):
        places[PLACES[i // 4] + DIRECTIONS[i % 4]] = { 'order': i + 1, 'exists': [], 'able': True, 'safe': PLACES[i // 4] in SAFE_PLACES }
else:
    for i in range(0, len(PLACES)):
        places[PLACES[i]] = { 'order': i + 1, 'exists': [], 'able': True, 'safe': PLACES[i] in SAFE_PLACES }

# order: 显示顺序，able: 能否行动，life: 生命，strength: 体力，location: 位置，hands: 手中道具，things: 背包道具，rest: 是否满足静养条件
roles = {
    '导演': { 'order': -1, 'life': 1000, 'strength': 100, 'hands': [], 'things': [], 'location': '主席台', 'able': False, 'rest': 2 },
    '观众': { 'order': 0, 'life': 1000, 'strength': 100, 'hands': [], 'things': [], 'location': '看台', 'able': False, 'rest': 2 }
}
for i in range(0, len(ROLES)):
    roles[ROLES[i]] = { 'order': i + 1, 'life': 100, 'strength': 100, 'hands': [], 'things': [], 'location': '', 'able': True, 'rest': 2 }

# [类别，剩余次数，白天票数]
items = {}
for i in ITEM_HOT_AOE:
    items[i] = [1, ITEM_HOT_AOE_TIMES, 0]
for i in ITEM_HOT_VITAL:
    items[i] = [2, ITEM_HOT_VITAL_TIMES, 0]
for i in ITEM_HOT:
    items[i] = [3, ITEM_HOT_TIMES, 0]
for i in ITEM_COLD:
    items[i] = [4, ITEM_COLD_TIMES, 1]
for i in ITEM_PROTECT + ITEM_ENSURE:
    items[i] = [5, 100, 2]
for i in ITEM_LOCATOR:
    items[i] = [6, 1, 3]
for i in ITEM_TELESCOPE:
    items[i] = [6, 2, 3]
for i in ITEM_LOCK + ITEM_SHOW:
    items[i] = [7, 1, 3]
for i in ITEM_BOMB + ITEM_KILL:
    items[i] = [8, 1, 0]
for k, v in places.items():
    v['name'] = k
# 是否已经出现在场过，避免升级产生重复道具
item_generated = set()

# 随机初始道具
initial_items = ITEM_COLD * 3 + list(ITEM_UPGRADE_MAP.keys()) * 3
for i in initial_items:
    places[random.choice(list(places.keys()))]['exists'].append(i)

for k, v in roles.items():
    v['name'] = k
    v['injured'] = v.get('injured', 0)
    v['deliver'] = v.get('deliver', 0)
    if v['able'] and len(v['location']) > 0:
        try:
            places[v['location']]['exists'].remove(k)
        except:
            pass
        places[v['location']]['exists'].append(k)
    if v['life'] <= 0:
        v['hands'] = []
        v['things'] = []
    v['ts'] = 0
